#[must_use]
pub fn xp_for_level(level: i64) -> i64 {
    let level = level.max(1);
    100 * level * level
}

#[must_use]
pub fn add_xp(current_level: i64, current_xp: i64, xp_to_next: i64, reward: i64) -> (i64, i64, i64) {
    let mut level = current_level.max(1);
    let mut xp = current_xp + reward;
    let mut to_next = if xp_to_next > 0 { xp_to_next } else { xp_for_level(level) };

    while xp >= to_next {
        xp -= to_next;
        level += 1;
        to_next = xp_for_level(level);
    }

    (level, xp, to_next)
}

#[must_use]
pub fn level_up_reward_gold(levels: i64) -> f64 {
    50.0 * levels.max(0) as f64
}

#[must_use]
pub fn level_up_reward_gems(levels: i64) -> f64 {
    5.0 * levels.max(0) as f64
}
