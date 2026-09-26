use serde::{Deserialize, Serialize};

use super::troops;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleTroop {
    pub id: String,
    pub troop_type: String,
    pub attack: i64,
    pub hp: i64,
    pub count: i64,
    pub level: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleBuilding {
    pub id: String,
    pub building_type: String,
    pub name: String,
    pub hp: i64,
    pub max_hp: i64,
    pub level: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleOutcome {
    pub won: bool,
    pub stars: i64,
    pub destruction_percent: f64,
    pub gold_reward: f64,
    pub gem_reward: f64,
    pub xp_reward: i64,
    pub trophy_change: i64,
    pub enemy_troops: Vec<BattleTroop>,
    pub enemy_buildings: Vec<BattleBuilding>,
}

#[must_use]
pub fn total_defense_hp(buildings: &[BattleBuilding]) -> i64 {
    buildings.iter().map(|b| b.hp.max(0)).sum()
}

#[must_use]
pub fn total_attack_power(troops_out: &[BattleTroop]) -> i64 {
    troops_out
        .iter()
        .map(|t| troops::attack_at_level(t.attack, t.level) * t.count.max(0))
        .sum()
}

#[must_use]
pub fn stars_for(destruction_percent: f64) -> i64 {
    if destruction_percent >= 100.0 {
        3
    } else if destruction_percent >= 66.0 {
        2
    } else if destruction_percent >= 33.0 {
        1
    } else {
        0
    }
}

#[must_use]
pub fn destruction_percent(attack_power: i64, defense_hp: i64) -> f64 {
    if defense_hp <= 0 {
        return 100.0;
    }
    let ratio = (attack_power as f64) / (defense_hp as f64);
    (ratio * 100.0).clamp(0.0, 100.0)
}

#[must_use]
pub fn simulate(
    _battle_type: &str,
    troops_out: &[BattleTroop],
    buildings: &[BattleBuilding],
) -> BattleOutcome {
    let attack_power = total_attack_power(troops_out);
    let defense_hp = total_defense_hp(buildings);
    let destruction = destruction_percent(attack_power, defense_hp);
    let stars = stars_for(destruction);
    let won = stars >= 1;

    let loot_pct = destruction / 100.0;
    let gold_reward = 100.0 * loot_pct * (1.0 + stars as f64 * 0.5);
    let gem_reward = (loot_pct * 10.0 + stars as f64 * 2.0).round();
    let xp_reward = (10.0 * loot_pct + 5.0 * stars as f64).round() as i64;

    let trophy_change = if won {
        (stars * 2) + (destruction / 25.0).round() as i64
    } else {
        -3
    };

    BattleOutcome {
        won,
        stars,
        destruction_percent: destruction,
        gold_reward,
        gem_reward,
        xp_reward,
        trophy_change,
        enemy_troops: troops_out.to_vec(),
        enemy_buildings: buildings.to_vec(),
    }
}
