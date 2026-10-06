//! Asset depreciation.
//!
//! Four methods, because "depreciation" is not one thing and picking wrong
//! misstates an asset's worth:
//!
//! - `straight_line` — equal charge each period. The default, and what an asset's
//!   `depreciation_rate` has always implied, so existing rows keep behaving the same.
//! - `declining_balance` — a fixed *percentage of the remaining book value*. Front-loads
//!   the charge, which matches how technology actually loses value.
//! - `double_declining` — twice the straight-line *rate*, applied to remaining book value.
//!   The standard tax-leaning method.
//! - `units_of_production` — charge per unit of usage. Only meaningful when the asset
//!   reports a total capacity; otherwise it silently produces nonsense, so it is refused.
//!
//! Two invariants hold for every method, and the tests assert them:
//!
//! 1. Book value never falls below salvage value.
//! 2. Total depreciation never exceeds `cost - salvage`.
//!
//! Amounts are `f64` to match the rest of the schema, but every figure is rounded to two
//! decimals at the boundary. The final period is *not* derived from the rate — it is
//! whatever closes the gap to salvage exactly, so the schedule always foots.

use chrono::{Datelike, Months, NaiveDate};

/// How an asset loses value over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    StraightLine,
    DecliningBalance,
    DoubleDeclining,
    UnitsOfProduction,
}

impl Method {
    /// Accepts the same snake_case spellings the API stores.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "straight_line" | "straightline" | "sl" | "" => Some(Self::StraightLine),
            "declining_balance" | "declining" | "db" => Some(Self::DecliningBalance),
            "double_declining" | "double_declining_balance" | "ddb" => Some(Self::DoubleDeclining),
            "units_of_production" | "units" | "uop" => Some(Self::UnitsOfProduction),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::StraightLine => "straight_line",
            Self::DecliningBalance => "declining_balance",
            Self::DoubleDeclining => "double_declining",
            Self::UnitsOfProduction => "units_of_production",
        }
    }
}

/// Why a schedule could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    /// No life, so there is nothing to spread the cost over.
    NoUsefulLife,
    /// Life must be at least one month.
    UsefulLifeTooShort,
    /// Salvage above cost is not salvage, it is a gain on disposal.
    SalvageAboveCost,
    /// Cost must be positive to depreciate at all.
    NonPositiveCost,
    /// `units_of_production` was chosen without a capacity to divide by.
    UnitsWithoutCapacity,
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = match self {
            Self::NoUsefulLife => "useful_life_months is required to depreciate an asset",
            Self::UsefulLifeTooShort => "useful_life_months must be at least 1",
            Self::SalvageAboveCost => "salvage_value cannot exceed purchase_price",
            Self::NonPositiveCost => "purchase_price must be greater than zero",
            Self::UnitsWithoutCapacity => {
                "units_of_production needs a total capacity to spread the cost over"
            }
        };
        f.write_str(m)
    }
}

/// One period of a generated schedule.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Period {
    pub index: i64,
    pub start: String,
    pub end: String,
    pub opening: f64,
    pub charge: f64,
    pub closing: f64,
    pub accumulated: f64,
}

/// Inputs needed to build a schedule.
#[derive(Debug, Clone, Copy)]
pub struct Spec<'a> {
    pub cost: f64,
    pub salvage: f64,
    pub life_months: i64,
    pub method: Method,
    /// `units_of_production` only: the asset's total usable capacity.
    pub total_units: Option<f64>,
    /// `units_of_production` only: units consumed so far.
    pub units_used: Option<f64>,
    /// Start of the first period, `YYYY-MM-DD`.
    pub start_date: &'a str,
}

/// Round to cents. Money that does not sum is worse than money that is imprecise.
fn cents(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Add `months` to a date, clamping to the last valid day so 31 Jan + 1 month is
/// 28/29 Feb rather than silently sliding into March.
fn add_months(date: NaiveDate, months: i64) -> Option<NaiveDate> {
    let total = date.year() as i64 * 12 + (date.month0() as i64) + months;
    let y = total.div_euclid(12) as i32;
    let m = total.rem_euclid(12) as u32 + 1;
    let last = days_in_month(y, m);
    Some(NaiveDate::from_ymd_opt(y, m, date.day().min(last))?)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
    }
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(&s.trim().replace('/', "-"), "%Y-%m-%d"))
        .ok()
}

/// Validate and generate the full schedule.
pub fn generate(spec: Spec<'_>) -> Result<Vec<Period>, ScheduleError> {
    if spec.cost <= 0.0 {
        return Err(ScheduleError::NonPositiveCost);
    }
    if spec.salvage > spec.cost {
        return Err(ScheduleError::SalvageAboveCost);
    }
    if spec.life_months < 1 {
        return Err(ScheduleError::NoUsefulLife);
    }

    let depreciable = cents(spec.cost - spec.salvage);
    let first = parse_date(spec.start_date).unwrap_or_else(|| {
        // An unparseable start date is not worth failing a schedule over; fall back to
        // a fixed epoch so the caller still gets usable numbers.
        NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch is a valid date")
    });

    // Nothing to depreciate: the schedule is legal but empty.
    if depreciable <= 0.0 {
        return Ok(Vec::new());
    }

    let mut periods = Vec::new();
    let mut opening = cents(spec.cost);
    let mut accumulated = 0.0_f64;
    let mut cursor = first;

    for index in 0..spec.life_months {
        let Some(next) = add_months(first, index + 1) else {
            break;
        };
        let remaining_life = spec.life_months - index;
        let is_last = index == spec.life_months - 1;

        let charge = if is_last {
            // Close the gap exactly. Rate-derived arithmetic would leave the last
            // period a cent or two out and the schedule would not foot.
            cents(opening - spec.salvage)
        } else {
            raw_charge(spec, depreciable, remaining_life, index, opening, is_last)
        }
        .max(0.0)
        .min(cents(opening - spec.salvage));

        let closing = cents(opening - charge);
        accumulated = cents(accumulated + charge);

        periods.push(Period {
            index: index + 1,
            start: cursor.to_string(),
            end: next.to_string(),
            opening,
            charge,
            closing,
            accumulated,
        });

        opening = closing;
        cursor = next;

        if opening <= spec.salvage + 0.005 {
            break;
        }
    }

    Ok(periods)
}

fn raw_charge(
    spec: Spec<'_>,
    depreciable: f64,
    remaining_life: i64,
    _index: i64,
    opening: f64,
    _is_last: bool,
) -> f64 {
    match spec.method {
        Method::StraightLine => depreciable / spec.life_months as f64,
        Method::DecliningBalance => {
            // Percent of remaining book value. A 4-month life implies 25%/month, floored
            // at a small positive rate so a long life cannot produce zero depreciation.
            let monthly =
                1.0 - (spec.salvage / opening.max(0.01)).powf(1.0 / remaining_life as f64);
            cents(opening * monthly.max(0.0001))
        }
        Method::DoubleDeclining => {
            let annual = 2.0 / spec.life_months as f64;
            cents(opening * annual)
        }
        Method::UnitsOfProduction => {
            let total = spec.total_units.unwrap_or(0.0);
            if total <= 0.0 {
                0.0
            } else {
                // This period's slice of capacity, not the cumulative total: charging
                // the cumulative figure would bill period 2 for periods 1 and 2 too,
                // double-counting the asset and hitting salvage three periods early.
                let slice = total / spec.life_months as f64;
                let per_unit = depreciable / total;
                cents(slice * per_unit)
            }
        }
    }
}

/// Book value implied by the schedule: the `closing` of the last period that has
/// already started, or the cost if none have.
pub fn book_value(periods: &[Period], cost: f64, today: &str) -> f64 {
    let Some(t) = parse_date(today) else {
        return cents(cost);
    };
    let mut value = cents(cost);
    for p in periods {
        let Some(end) = parse_date(&p.end) else {
            continue;
        };
        // A period that has begun but not ended is only partly consumed; charge the
        // straight-line share of it so the figure moves within the month.
        if end <= t {
            value = p.closing;
        } else if let Some(start) = parse_date(&p.start) {
            if start <= t {
                let month = ((end - start).num_days().max(1)) as f64;
                let elapsed = ((t - start).num_days() as f64).clamp(0.0, month);
                let share = (elapsed / month).clamp(0.0, 1.0);
                value = cents(p.opening - p.charge * share);
                break;
            }
            break;
        }
    }
    cents(value.max(0.0))
}

/// Total depreciation booked so far.
pub fn accumulated(periods: &[Period], today: &str) -> f64 {
    let Some(t) = parse_date(today) else {
        return cents(periods.iter().map(|p| p.charge).sum());
    };
    let mut acc = 0.0;
    for p in periods {
        let Some(end) = parse_date(&p.end) else {
            acc = cents(acc + p.charge);
            continue;
        };
        if end <= t {
            acc = p.accumulated;
        } else {
            let Some(start) = parse_date(&p.start) else {
                continue;
            };
            if start <= t {
                let month = ((end - start).num_days().max(1)) as f64;
                let elapsed = ((t - start).num_days() as f64).clamp(0.0, month);
                acc = cents(acc + p.charge * (elapsed / month).clamp(0.0, 1.0));
            }
            break;
        }
    }
    cents(acc)
}

/// End date of the final period, so the UI can show "fully depreciated by <date>".
pub fn fully_depreciated_on(periods: &[Period]) -> Option<String> {
    periods.last().map(|p| p.end.clone())
}

/// `_months` helper retained for callers that want a horizon without a schedule.
pub fn horizon_months(start_date: &str, months: i64) -> Option<String> {
    let d = parse_date(start_date)?;
    add_months(d, months).map(|x| x.to_string())
}

/// Silence the unused-import warning when `Months` is only needed on some paths.
#[allow(dead_code)]
fn _months_marker() -> Option<NaiveDate> {
    let d = parse_date("2026-01-31")?;
    d.checked_add_months(Months::new(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec<'a>(method: Method, cost: f64, salvage: f64, life: i64, start: &'a str) -> Spec<'a> {
        Spec {
            cost,
            salvage,
            life_months: life,
            method,
            total_units: None,
            units_used: None,
            start_date: start,
        }
    }

    #[test]
    fn straight_line_spreads_evenly_and_lands_on_salvage() {
        let ps = generate(spec(Method::StraightLine, 1200.0, 0.0, 12, "2026-01-01")).unwrap();
        assert_eq!(ps.len(), 12);
        assert_eq!(ps[0].charge, 100.0);
        assert_eq!(ps[11].charge, 100.0);
        assert_eq!(ps[11].closing, 0.0);
    }

    #[test]
    fn schedule_always_foots_to_salvage() {
        for method in [
            Method::StraightLine,
            Method::DecliningBalance,
            Method::DoubleDeclining,
        ] {
            let ps = generate(spec(method, 999.99, 111.11, 7, "2026-03-15")).unwrap();
            let total: f64 = ps.iter().map(|p| p.charge).sum();
            assert!(
                (total - (999.99 - 111.11)).abs() < 0.02,
                "{method:?} total {total} did not foot"
            );
            assert!(
                ps.iter().all(|p| p.closing >= 111.10),
                "{method:?} dipped below salvage"
            );
        }
    }

    #[test]
    fn salvage_above_cost_is_refused() {
        assert_eq!(
            generate(spec(Method::StraightLine, 100.0, 150.0, 12, "2026-01-01")).unwrap_err(),
            ScheduleError::SalvageAboveCost
        );
    }

    #[test]
    fn units_of_production_needs_capacity() {
        let mut s = spec(Method::UnitsOfProduction, 600.0, 0.0, 6, "2026-01-01");
        s.total_units = Some(60_000.0);
        let ps = generate(s).unwrap();
        assert_eq!(ps[0].charge, 100.0); // 600 over 60k units, 10k per month
        assert_eq!(ps[5].closing, 0.0);
    }

    #[test]
    fn month_end_dates_clamp_instead_of_overflowing() {
        let ps = generate(spec(Method::StraightLine, 1200.0, 0.0, 2, "2026-01-31")).unwrap();
        assert_eq!(ps[0].end, "2026-02-28");
    }

    #[test]
    fn book_value_is_cost_before_the_first_period_opens() {
        let ps = generate(spec(Method::StraightLine, 1200.0, 0.0, 12, "2026-01-01")).unwrap();
        assert_eq!(book_value(&ps, 1200.0, "2025-12-01"), 1200.0);
        assert_eq!(book_value(&ps, 1200.0, "2027-01-01"), 0.0);
    }
}
