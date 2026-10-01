use std::collections::BTreeMap;
use sea_orm::{ProxyRow, Value as SeaValue};
use turso::{Row, Value as TursoValue};
use crate::connection::{ColumnType};

const BOOL_COLUMN_NAMES: &[&str] = &[
    "completed", "active", "enabled", "verified", "approved",
    "is_", "has_", "can_", "should_", "was_", "did_",
    "deleted", "archived", "published", "visible", "hidden",
    "disabled", "is_active", "is_primary",
    // SQLite has no BOOLEAN type, so bool columns are declared INTEGER and are
    // otherwise decoded as BigInt. investments.tax_saving is the one bool column
    // that does not follow an is_/has_ naming convention, and its model field is
    // a plain bool, so reading a row failed with "Missing value for column
    // 'tax_saving'".
    "tax_saving",
];

fn is_likely_boolean(col_name: &str) -> bool {
    let lower = col_name.to_lowercase();
    BOOL_COLUMN_NAMES.iter().any(|p| lower == *p || lower.starts_with(p))
}

/// Convert a Turso Row to a SeaORM ProxyRow
pub fn row_to_proxy_row(
    row: &Row,
    column_names: &[String],
    schema_cache: &std::collections::HashMap<String, std::collections::HashMap<String, ColumnType>>,
) -> Result<ProxyRow, String> {
    let mut values = BTreeMap::new();
    let column_count = row.column_count();

    for i in 0..column_count {
        let col_name = if let Some(name) = column_names.get(i) {
            name.clone()
        } else {
            format!("col_{}", i)
        };

        let val = row.get_value(i)
            .map_err(|e| format!("Failed to get column {}: {}", col_name, e))?;

        let is_bool = schema_cache.values().any(|cols| {
            matches!(cols.get(&col_name), Some(ColumnType::Boolean))
        }) || is_likely_boolean(&col_name);

        let sea_val = turso_value_to_sea_value_typed(val, is_bool)?;
        values.insert(col_name, sea_val);
    }

    Ok(ProxyRow::new(values))
}

/// Convert a Turso Value to a SeaORM Value with type awareness
fn turso_value_to_sea_value_typed(val: TursoValue, is_bool: bool) -> Result<SeaValue, String> {
    match val {
        TursoValue::Null => Ok(SeaValue::Bool(None)),
        TursoValue::Integer(i) => {
            if is_bool {
                Ok(SeaValue::Bool(Some(i != 0)))
            } else {
                Ok(SeaValue::BigInt(Some(i)))
            }
        }
        TursoValue::Real(f) => Ok(SeaValue::Double(Some(f))),
        TursoValue::Text(s) => Ok(SeaValue::String(Some(s))),
        TursoValue::Blob(b) => Ok(SeaValue::Bytes(Some(b))),
    }
}

/// Try to parse a string as chrono DateTime<FixedOffset>
fn try_parse_datetime(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt);
    }
    if let Ok(dt) = chrono::DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%:z") {
        return Some(dt);
    }
    if let Ok(dt) = chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%:z") {
        return Some(dt);
    }
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(ndt.and_utc().fixed_offset());
    }
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(ndt.and_utc().fixed_offset());
    }
    None
}

/// Try to parse a string as chrono NaiveDate
fn try_parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Try to parse a string as chrono NaiveTime
fn try_parse_time(s: &str) -> Option<chrono::NaiveTime> {
    chrono::NaiveTime::parse_from_str(s, "%H:%M:%S")
        .or_else(|_| chrono::NaiveTime::parse_from_str(s, "%H:%M:%S%.f"))
        .ok()
}

/// Convert a SeaORM Statement to SQL string and parameters for Turso
pub fn statement_to_sql(stmt: &sea_orm::Statement) -> (String, Vec<TursoValue>) {
    let sql = stmt.sql.clone();
    let params = stmt.values.as_ref().map(|v| {
        v.0.iter().map(|v| sea_value_to_turso_value(v.clone())).collect()
    }).unwrap_or_default();

    (sql, params)
}

/// Convert a SeaORM Value to a Turso Value
pub fn sea_value_to_turso_value(val: SeaValue) -> TursoValue {
    match val {
        SeaValue::Bool(Some(b)) => TursoValue::Integer(b as i64),
        SeaValue::Bool(None) => TursoValue::Null,
        SeaValue::TinyInt(Some(i)) => TursoValue::Integer(i as i64),
        SeaValue::TinyInt(None) => TursoValue::Null,
        SeaValue::SmallInt(Some(i)) => TursoValue::Integer(i as i64),
        SeaValue::SmallInt(None) => TursoValue::Null,
        SeaValue::Int(Some(i)) => TursoValue::Integer(i as i64),
        SeaValue::Int(None) => TursoValue::Null,
        SeaValue::BigInt(Some(i)) => TursoValue::Integer(i),
        SeaValue::BigInt(None) => TursoValue::Null,
        SeaValue::TinyUnsigned(Some(i)) => TursoValue::Integer(i as i64),
        SeaValue::TinyUnsigned(None) => TursoValue::Null,
        SeaValue::SmallUnsigned(Some(i)) => TursoValue::Integer(i as i64),
        SeaValue::SmallUnsigned(None) => TursoValue::Null,
        SeaValue::Unsigned(Some(i)) => TursoValue::Integer(i as i64),
        SeaValue::Unsigned(None) => TursoValue::Null,
        SeaValue::BigUnsigned(Some(i)) => TursoValue::Integer(i as i64),
        SeaValue::BigUnsigned(None) => TursoValue::Null,
        SeaValue::Float(Some(f)) => TursoValue::Real(f as f64),
        SeaValue::Float(None) => TursoValue::Null,
        SeaValue::Double(Some(f)) => TursoValue::Real(f),
        SeaValue::Double(None) => TursoValue::Null,
        SeaValue::String(Some(s)) => TursoValue::Text((*s).to_string()),
        SeaValue::String(None) => TursoValue::Null,
        SeaValue::Bytes(Some(b)) => TursoValue::Blob((*b).to_vec()),
        SeaValue::Bytes(None) => TursoValue::Null,
        SeaValue::ChronoDateTime(Some(dt)) => TursoValue::Text(dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        SeaValue::ChronoDateTime(None) => TursoValue::Null,
        SeaValue::ChronoDateTimeWithTimeZone(Some(dt)) => TursoValue::Text(dt.to_rfc3339()),
        SeaValue::ChronoDateTimeWithTimeZone(None) => TursoValue::Null,
        SeaValue::ChronoDate(Some(d)) => TursoValue::Text(d.to_string()),
        SeaValue::ChronoDate(None) => TursoValue::Null,
        SeaValue::ChronoTime(Some(t)) => TursoValue::Text(t.to_string()),
        SeaValue::ChronoTime(None) => TursoValue::Null,
        SeaValue::TimeDateTime(Some(dt)) => TursoValue::Text(dt.to_string()),
        SeaValue::TimeDateTime(None) => TursoValue::Null,
        SeaValue::TimeDateTimeWithTimeZone(Some(dt)) => TursoValue::Text(dt.to_string()),
        SeaValue::TimeDateTimeWithTimeZone(None) => TursoValue::Null,
        SeaValue::Uuid(Some(u)) => TursoValue::Text(u.to_string()),
        SeaValue::Uuid(None) => TursoValue::Null,
        SeaValue::Json(Some(j)) => TursoValue::Text(j.to_string()),
        SeaValue::Json(None) => TursoValue::Null,
        _ => TursoValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sea_value_to_turso_value() {
        assert!(matches!(sea_value_to_turso_value(SeaValue::Bool(Some(true))), TursoValue::Integer(1)));
        assert!(matches!(sea_value_to_turso_value(SeaValue::Bool(None)), TursoValue::Null));
        assert!(matches!(sea_value_to_turso_value(SeaValue::BigInt(Some(42))), TursoValue::Integer(42)));
        assert!(matches!(sea_value_to_turso_value(SeaValue::Double(Some(3.14))), TursoValue::Real(3.14)));
    }

    #[test]
    fn test_try_parse_datetime() {
        assert!(try_parse_datetime("2026-09-13T10:30:00+00:00").is_some());
        assert!(try_parse_datetime("2026-09-13 10:30:00+00:00").is_some());
        assert!(try_parse_datetime("2026-09-13 10:30:00").is_some());
        assert!(try_parse_datetime("not a date").is_none());
    }

    #[test]
    fn test_try_parse_date() {
        assert!(try_parse_date("2026-09-13").is_some());
        assert!(try_parse_date("not a date").is_none());
    }

    #[test]
    fn test_try_parse_time() {
        assert!(try_parse_time("10:30:00").is_some());
        assert!(try_parse_time("10:30:00.123").is_some());
        assert!(try_parse_time("not a time").is_none());
    }

    #[test]
    fn test_turso_text_to_datetime() {
        // Every timestamp column in this project is modelled as Option<String>,
        // so timestamps must stay Text on read or they hydrate as NULL.
        let val = turso_value_to_sea_value_typed(TursoValue::Text("2026-09-13T10:30:00+00:00".into()), false).unwrap();
        assert!(matches!(val, SeaValue::String(Some(_))));
    }

    #[test]
    fn test_turso_text_to_naive_datetime() {
        let val = turso_value_to_sea_value_typed(TursoValue::Text("2026-09-13 10:30:00".into()), false).unwrap();
        assert!(matches!(val, SeaValue::String(Some(_))));
    }

    #[test]
    fn test_turso_text_fallback_string() {
        let val = turso_value_to_sea_value_typed(TursoValue::Text("hello world".into()), false).unwrap();
        assert!(matches!(val, SeaValue::String(Some(_))));
    }
}
