//! Career save schema v2 — per-level personal bests and recent clears.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rating::grade_from_rating;

pub const SAVE_VERSION: u32 = 2;
pub const LEVELS_PER_BIOME: u64 = 12;

#[derive(Serialize, Deserialize, Resource, Clone, Debug)]
pub struct SaveData {
    pub version: u32,
    pub master_seed: u64,
    pub player_name: String,
    pub highest_galaxy: u64,
    pub highest_biome: u64,
    pub highest_level: u64,
    pub career_high_score: u64,
    pub total_bricks_destroyed: u64,
    pub total_levels_cleared: u64,
    pub level_results: HashMap<String, LevelPersonalBest>,
    pub recent_clears: Vec<RecentClear>,
    pub achievements: HashSet<String>,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            master_seed: 42,
            player_name: "Pilot".into(),
            highest_galaxy: 0,
            highest_biome: 0,
            highest_level: 0,
            career_high_score: 0,
            total_bricks_destroyed: 0,
            total_levels_cleared: 0,
            level_results: HashMap::new(),
            recent_clears: Vec::new(),
            achievements: HashSet::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LevelPersonalBest {
    pub best_rating: u8,
    pub best_score: u64,
    pub best_max_combo: u32,
    pub best_time_secs: f32,
    pub attempts: u32,
    pub clears: u32,
    pub last_played_unix: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RecentClear {
    pub key: String,
    pub galaxy: u64,
    pub biome: u64,
    pub level: u64,
    pub rating: u8,
    pub grade: String,
    pub is_pb: bool,
}

pub fn level_key(galaxy: u64, biome: u64, level: u64) -> String {
    format!("g{galaxy}_b{biome}_l{level}")
}

/// Whether the address is at or behind the campaign frontier (unlocked).
pub fn is_level_unlocked(
    galaxy: u64,
    biome: u64,
    level: u64,
    highest_galaxy: u64,
    highest_biome: u64,
    highest_level: u64,
) -> bool {
    if galaxy < highest_galaxy {
        return true;
    }
    if galaxy > highest_galaxy {
        return false;
    }
    if biome < highest_biome {
        return true;
    }
    if biome > highest_biome {
        return false;
    }
    level <= highest_level
}

/// Average best rating over cleared levels (0 if none).
pub fn mastery_percent(save: &SaveData) -> f32 {
    let cleared: Vec<u8> = save
        .level_results
        .values()
        .filter(|pb| pb.clears > 0)
        .map(|pb| pb.best_rating)
        .collect();
    if cleared.is_empty() {
        0.0
    } else {
        let sum: u32 = cleared.iter().map(|&r| r as u32).sum();
        sum as f32 / cleared.len() as f32
    }
}

pub fn s_rank_count(save: &SaveData) -> u32 {
    save.level_results
        .values()
        .filter(|pb| pb.clears > 0 && pb.best_rating >= 90)
        .count() as u32
}

/// Merge a successful clear into save data. Returns whether this was a PB.
pub fn apply_level_clear(
    save: &mut SaveData,
    galaxy: u64,
    biome: u64,
    level: u64,
    rating: u8,
    level_score: u64,
    max_combo: u32,
    time_secs: f32,
    run_score: u64,
    bricks_destroyed: u64,
    now_unix: u64,
) -> bool {
    let key = level_key(galaxy, biome, level);
    let grade = grade_from_rating(rating);

    let entry = save
        .level_results
        .entry(key.clone())
        .or_insert(LevelPersonalBest {
            best_rating: 0,
            best_score: 0,
            best_max_combo: 0,
            best_time_secs: f32::MAX,
            attempts: 0,
            clears: 0,
            last_played_unix: 0,
        });

    entry.attempts = entry.attempts.saturating_add(1);
    entry.clears = entry.clears.saturating_add(1);
    entry.last_played_unix = now_unix;

    let is_pb = rating > entry.best_rating
        || (rating == entry.best_rating && level_score > entry.best_score);

    if is_pb {
        entry.best_rating = rating;
        entry.best_score = level_score;
    }
    if max_combo > entry.best_max_combo {
        entry.best_max_combo = max_combo;
    }
    if time_secs < entry.best_time_secs {
        entry.best_time_secs = time_secs;
    }

    advance_frontier(save, galaxy, biome, level);

    save.total_levels_cleared = save.total_levels_cleared.saturating_add(1);
    save.total_bricks_destroyed = save.total_bricks_destroyed.saturating_add(bricks_destroyed);
    if run_score > save.career_high_score {
        save.career_high_score = run_score;
    }

    if rating >= 90 {
        save.achievements.insert("first_s".into());
    }
    if rating >= 97 {
        save.achievements.insert("first_ss".into());
    }
    save.achievements.insert("first_clear".into());

    save.recent_clears.insert(
        0,
        RecentClear {
            key,
            galaxy,
            biome,
            level,
            rating,
            grade: grade.as_str().to_string(),
            is_pb,
        },
    );
    save.recent_clears.truncate(10);

    is_pb
}

fn advance_frontier(save: &mut SaveData, galaxy: u64, biome: u64, cleared_level: u64) {
    let (ug, ub, ul) = if cleared_level + 1 < LEVELS_PER_BIOME {
        (galaxy, biome, cleared_level + 1)
    } else {
        (galaxy, biome + 1, 0)
    };

    let better = ug > save.highest_galaxy
        || (ug == save.highest_galaxy && ub > save.highest_biome)
        || (ug == save.highest_galaxy && ub == save.highest_biome && ul > save.highest_level);

    if better {
        save.highest_galaxy = ug;
        save.highest_biome = ub;
        save.highest_level = ul;
    }
}

/// Record an attempt that did not clear (game over mid-level).
pub fn apply_level_attempt(
    save: &mut SaveData,
    galaxy: u64,
    biome: u64,
    level: u64,
    now_unix: u64,
) {
    let key = level_key(galaxy, biome, level);
    let entry = save.level_results.entry(key).or_insert(LevelPersonalBest {
        best_rating: 0,
        best_score: 0,
        best_max_combo: 0,
        best_time_secs: f32::MAX,
        attempts: 0,
        clears: 0,
        last_played_unix: 0,
    });
    entry.attempts = entry.attempts.saturating_add(1);
    entry.last_played_unix = now_unix;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_key_format() {
        assert_eq!(level_key(1, 2, 3), "g1_b2_l3");
    }

    #[test]
    fn pb_updates_on_higher_rating() {
        let mut save = SaveData::default();
        let pb1 = apply_level_clear(&mut save, 0, 0, 0, 70, 500, 10, 40.0, 500, 20, 1);
        assert!(pb1);
        assert_eq!(save.level_results["g0_b0_l0"].best_rating, 70);

        let pb2 = apply_level_clear(&mut save, 0, 0, 0, 85, 400, 12, 35.0, 900, 20, 2);
        assert!(pb2);
        assert_eq!(save.level_results["g0_b0_l0"].best_rating, 85);
        assert_eq!(save.level_results["g0_b0_l0"].best_score, 400);
    }

    #[test]
    fn lower_rating_is_not_pb() {
        let mut save = SaveData::default();
        apply_level_clear(&mut save, 0, 0, 0, 90, 1000, 20, 20.0, 1000, 10, 1);
        let pb = apply_level_clear(&mut save, 0, 0, 0, 80, 2000, 15, 30.0, 2000, 10, 2);
        assert!(!pb);
        assert_eq!(save.level_results["g0_b0_l0"].best_rating, 90);
        assert_eq!(save.level_results["g0_b0_l0"].best_score, 1000);
        assert_eq!(save.level_results["g0_b0_l0"].clears, 2);
    }

    #[test]
    fn same_rating_higher_score_is_pb() {
        let mut save = SaveData::default();
        apply_level_clear(&mut save, 0, 0, 0, 80, 500, 10, 40.0, 500, 5, 1);
        let pb = apply_level_clear(&mut save, 0, 0, 0, 80, 800, 10, 40.0, 800, 5, 2);
        assert!(pb);
        assert_eq!(save.level_results["g0_b0_l0"].best_score, 800);
    }

    #[test]
    fn frontier_advances_on_clear() {
        let mut save = SaveData::default();
        assert_eq!(save.highest_level, 0);
        apply_level_clear(&mut save, 0, 0, 0, 60, 100, 5, 50.0, 100, 10, 1);
        assert_eq!(save.highest_galaxy, 0);
        assert_eq!(save.highest_biome, 0);
        assert_eq!(save.highest_level, 1);
    }

    #[test]
    fn unlocked_helpers() {
        assert!(is_level_unlocked(0, 0, 0, 0, 0, 0));
        assert!(is_level_unlocked(0, 0, 1, 0, 0, 2));
        assert!(!is_level_unlocked(0, 0, 3, 0, 0, 2));
        assert!(!is_level_unlocked(1, 0, 0, 0, 0, 5));
    }

    #[test]
    fn mastery_and_recent() {
        let mut save = SaveData::default();
        apply_level_clear(&mut save, 0, 0, 0, 80, 100, 5, 40.0, 100, 5, 1);
        apply_level_clear(&mut save, 0, 0, 1, 60, 100, 5, 40.0, 200, 5, 2);
        let m = mastery_percent(&save);
        assert!((m - 70.0).abs() < 0.01, "mastery={m}");
        assert_eq!(save.recent_clears.len(), 2);
        assert_eq!(save.recent_clears[0].level, 1);
    }
}
