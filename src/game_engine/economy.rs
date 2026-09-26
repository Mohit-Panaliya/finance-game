use chrono::{DateTime, Utc};

#[must_use]
pub fn production_per_hour(production_rate: f64, level: i64) -> f64 {
    production_rate * (level.max(1) as f64)
}

#[must_use]
pub fn accrue_production(
    stored_resource: f64,
    production_rate: f64,
    level: i64,
    last_collected_at: Option<&str>,
    now: &DateTime<Utc>,
) -> (f64, String) {
    let rate = production_per_hour(production_rate, level);
    let accrued = match last_collected_at.and_then(|s| DateTime::parse_from_rfc3339(s).ok()) {
        Some(prev) => {
            let prev = prev.with_timezone(&Utc);
            let hours = (now.signed_duration_since(prev).num_seconds().max(0) as f64) / 3600.0;
            rate * hours
        }
        None => 0.0,
    };
    (stored_resource + accrued, now.to_rfc3339())
}

#[must_use]
pub fn upgrade_cost(building_type: &str, level: i64) -> f64 {
    let base_cost = match building_type {
        "town_hall" => 100.0,
        "gold_mine" => 50.0,
        "gold_storage" => 50.0,
        "gem_mine" => 100.0,
        "gem_storage" => 100.0,
        "barracks" => 200.0,
        "army_camp" => 150.0,
        "laboratory" => 300.0,
        "spell_factory" => 250.0,
        "workshop" => 300.0,
        "clan_castle" => 400.0,
        "cannon" => 80.0,
        "archer_tower" => 90.0,
        "mortar" => 120.0,
        "air_defense" => 150.0,
        "wizard_tower" => 180.0,
        "inferno_tower" => 300.0,
        "eagle_artillery" => 500.0,
        "scattershot" => 400.0,
        "walls" => 30.0,
        _ => 50.0,
    };
    let lvl = level.max(1);
    base_cost * 1.7f64.powi((lvl - 1) as i32)
}

#[must_use]
pub fn upgrade_duration_secs(building_type: &str, level: i64) -> i64 {
    let base_duration = match building_type {
        "town_hall" => 3600,
        "gold_mine" => 1800,
        "gold_storage" => 1800,
        "gem_mine" => 3600,
        "gem_storage" => 3600,
        "barracks" => 7200,
        "army_camp" => 5400,
        "laboratory" => 10800,
        "spell_factory" => 9000,
        "workshop" => 10800,
        "clan_castle" => 14400,
        "cannon" => 2700,
        "archer_tower" => 3000,
        "mortar" => 3600,
        "air_defense" => 4500,
        "wizard_tower" => 5400,
        "inferno_tower" => 7200,
        "eagle_artillery" => 14400,
        "scattershot" => 10800,
        "walls" => 600,
        _ => 1800,
    };
    base_duration * level.max(1)
}

#[must_use]
pub fn parse_ts(s: Option<&str>) -> Option<DateTime<Utc>> {
    s.and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}
