use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct TroopCatalog {
    pub troop_type: &'static str,
    pub name: &'static str,
    pub base_attack: i64,
    pub base_hp: i64,
    pub train_cost: f64,
    pub train_time_secs: i64,
}

pub const CATALOG: &[TroopCatalog] = &[
    TroopCatalog {
        troop_type: "raider",
        name: "Raider",
        base_attack: 10,
        base_hp: 20,
        train_cost: 10.0,
        train_time_secs: 5,
    },
    TroopCatalog {
        troop_type: "archer",
        name: "Archer",
        base_attack: 8,
        base_hp: 12,
        train_cost: 8.0,
        train_time_secs: 4,
    },
    TroopCatalog {
        troop_type: "brute",
        name: "Brute",
        base_attack: 15,
        base_hp: 40,
        train_cost: 25.0,
        train_time_secs: 8,
    },
];

#[must_use]
pub fn find(troop_type: &str) -> Option<&'static TroopCatalog> {
    CATALOG.iter().find(|t| t.troop_type == troop_type)
}

#[must_use]
pub fn attack_at_level(base_attack: i64, level: i64) -> i64 {
    base_attack * level.max(1)
}

#[must_use]
pub fn hp_at_level(base_hp: i64, level: i64) -> i64 {
    base_hp * level.max(1)
}

#[must_use]
pub fn train_duration_secs(train_time_secs: i64, count: i64) -> i64 {
    train_time_secs.max(1) * count.max(1)
}

#[must_use]
pub fn training_complete_at(now: &DateTime<Utc>, train_time_secs: i64, count: i64) -> String {
    let secs = train_duration_secs(train_time_secs, count);
    (*now + chrono::Duration::seconds(secs)).to_rfc3339()
}

#[must_use]
pub fn is_training_complete(training_finishes_at: Option<&str>, now: &DateTime<Utc>) -> bool {
    match training_finishes_at
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
    {
        Some(dt) => now.signed_duration_since(dt.with_timezone(&Utc)).num_seconds() >= 0,
        None => false,
    }
}

#[must_use]
pub fn train_cost(troop_type: &str) -> f64 {
    find(troop_type).map(|t| t.train_cost).unwrap_or(10.0)
}

#[must_use]
pub fn train_time(troop_type: &str) -> i64 {
    find(troop_type).map(|t| t.train_time_secs).unwrap_or(5)
}

#[must_use]
pub fn base_attack(troop_type: &str) -> i64 {
    find(troop_type).map(|t| t.base_attack).unwrap_or(10)
}

#[must_use]
pub fn base_hp(troop_type: &str) -> i64 {
    find(troop_type).map(|t| t.base_hp).unwrap_or(20)
}
