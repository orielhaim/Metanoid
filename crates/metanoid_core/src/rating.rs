//! 0–100 level performance rating with transparent breakdown.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Stats collected during a single level attempt.
#[derive(Resource, Debug, Clone)]
pub struct LevelRunStats {
    pub breakable_bricks: u32,
    pub bricks_destroyed: u32,
    pub max_combo: u32,
    pub lives_remaining: i32,
    pub starting_lives: i32,
    /// Ball deaths / life losses this level.
    pub deaths: u32,
    pub elapsed_secs: f32,
    pub is_boss: bool,
    /// Points earned during this level (run score delta).
    pub run_score_delta: u64,
    pub score_at_level_start: u64,
    pub biome_difficulty: f32,
}

impl Default for LevelRunStats {
    fn default() -> Self {
        Self {
            breakable_bricks: 0,
            bricks_destroyed: 0,
            max_combo: 0,
            lives_remaining: 3,
            starting_lives: 3,
            deaths: 0,
            elapsed_secs: 0.0,
            is_boss: false,
            run_score_delta: 0,
            score_at_level_start: 0,
            biome_difficulty: 0.3,
        }
    }
}

impl LevelRunStats {
    pub fn begin_level(
        &mut self,
        breakable_bricks: u32,
        starting_lives: i32,
        score_at_start: u64,
        is_boss: bool,
        biome_difficulty: f32,
    ) {
        *self = Self {
            breakable_bricks,
            bricks_destroyed: 0,
            max_combo: 0,
            lives_remaining: starting_lives,
            starting_lives,
            deaths: 0,
            elapsed_secs: 0.0,
            is_boss,
            run_score_delta: 0,
            score_at_level_start: score_at_start,
            biome_difficulty,
        };
    }
}

/// Targets derived from level content for fair scoring.
#[derive(Debug, Clone, Copy)]
pub struct RatingTargets {
    pub par_time_secs: f32,
    pub target_max_combo: u32,
    pub starting_lives: i32,
}

impl RatingTargets {
    pub fn from_level(breakable_bricks: u32, biome_difficulty: f32, starting_lives: i32) -> Self {
        let bricks = breakable_bricks.max(1) as f32;
        let par_time_secs = bricks * 1.15 * (1.0 + 0.15 * biome_difficulty.clamp(0.0, 1.0));
        let target_max_combo = (bricks * 0.35).clamp(8.0, 50.0) as u32;
        Self {
            par_time_secs,
            target_max_combo,
            starting_lives,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Grade {
    #[default]
    D,
    C,
    B,
    A,
    S,
    Ss,
}

impl Grade {
    pub fn as_str(self) -> &'static str {
        match self {
            Grade::D => "D",
            Grade::C => "C",
            Grade::B => "B",
            Grade::A => "A",
            Grade::S => "S",
            Grade::Ss => "SS",
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn grade_from_rating(rating: u8) -> Grade {
    match rating {
        0..=39 => Grade::D,
        40..=59 => Grade::C,
        60..=74 => Grade::B,
        75..=89 => Grade::A,
        90..=96 => Grade::S,
        _ => Grade::Ss,
    }
}

/// Transparent weighted breakdown (parts are pre-weight contributions 0..weight).
#[derive(Debug, Clone)]
pub struct RatingBreakdown {
    pub total: u8,
    pub grade: Grade,
    pub clear_part: f32,
    pub lives_part: f32,
    pub combo_part: f32,
    pub speed_part: f32,
    pub clean_part: f32,
    pub clean_bonus: f32,
}

/// Compute 0–100 rating from run stats. Call only on successful clear.
pub fn compute_rating(stats: &LevelRunStats, targets: &RatingTargets) -> RatingBreakdown {
    let clear_base = 1.0_f32;
    let clear_part = 35.0 * clear_base;

    let lives_denom = targets.starting_lives.max(1) as f32;
    let lives_factor = (stats.lives_remaining.max(0) as f32 / lives_denom).clamp(0.0, 1.0);
    let lives_part = 20.0 * lives_factor;

    let combo_factor = if targets.target_max_combo == 0 {
        0.0
    } else {
        (stats.max_combo as f32 / targets.target_max_combo as f32)
            .clamp(0.0, 1.15)
            .min(1.0)
    };
    let combo_part = 20.0 * combo_factor;

    let speed_factor = speed_factor(stats.elapsed_secs, targets.par_time_secs);
    let speed_part = 15.0 * speed_factor;

    let clean_factor = 1.0 - (stats.deaths as f32 / 3.0).min(1.0);
    let clean_part = 10.0 * clean_factor;

    let clean_bonus = if stats.deaths == 0 { 5.0 } else { 0.0 };

    let raw = clear_part + lives_part + combo_part + speed_part + clean_part + clean_bonus;
    let total = raw.round().clamp(0.0, 100.0) as u8;

    RatingBreakdown {
        total,
        grade: grade_from_rating(total),
        clear_part,
        lives_part,
        combo_part,
        speed_part,
        clean_part,
        clean_bonus,
    }
}

fn speed_factor(elapsed: f32, par: f32) -> f32 {
    if par <= 0.0 {
        return 1.0;
    }
    if elapsed <= par {
        1.0
    } else if elapsed >= 2.5 * par {
        0.0
    } else {
        // Linear from 1.0 at par to 0.0 at 2.5*par
        1.0 - (elapsed - par) / (1.5 * par)
    }
}

/// Live estimate using current stats (same formula).
pub fn estimate_rating(stats: &LevelRunStats) -> u8 {
    let targets = RatingTargets::from_level(
        stats.breakable_bricks,
        stats.biome_difficulty,
        stats.starting_lives,
    );
    compute_rating(stats, &targets).total
}

/// Result stored after a clear for UI and save merge.
#[derive(Resource, Debug, Clone, Default)]
pub struct LastRatingResult {
    pub galaxy: u64,
    pub biome: u64,
    pub level: u64,
    pub rating: u8,
    pub grade: Grade,
    pub breakdown: Option<RatingBreakdown>,
    pub level_score: u64,
    pub is_pb: bool,
    pub is_boss: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_stats() -> LevelRunStats {
        LevelRunStats {
            breakable_bricks: 40,
            bricks_destroyed: 40,
            max_combo: 20,
            lives_remaining: 3,
            starting_lives: 3,
            deaths: 0,
            elapsed_secs: 30.0,
            is_boss: false,
            run_score_delta: 1000,
            score_at_level_start: 0,
            biome_difficulty: 0.3,
        }
    }

    #[test]
    fn near_perfect_clear_scores_high() {
        let stats = base_stats();
        let targets = RatingTargets::from_level(40, 0.3, 3);
        // Under par, full lives, strong combo, zero deaths
        let mut s = stats;
        s.elapsed_secs = targets.par_time_secs * 0.5;
        s.max_combo = targets.target_max_combo;
        let r = compute_rating(&s, &targets);
        assert!(
            r.total >= 90,
            "near-perfect should be high, got {}",
            r.total
        );
        assert!(matches!(r.grade, Grade::S | Grade::Ss));
        assert_eq!(r.clean_bonus, 5.0);
    }

    #[test]
    fn poor_clear_scores_lower_than_good() {
        let targets = RatingTargets::from_level(40, 0.3, 3);
        let mut good = base_stats();
        good.elapsed_secs = targets.par_time_secs * 0.8;
        good.max_combo = targets.target_max_combo;
        good.deaths = 0;
        good.lives_remaining = 3;

        let mut poor = base_stats();
        poor.elapsed_secs = targets.par_time_secs * 2.4;
        poor.max_combo = 2;
        poor.deaths = 3;
        poor.lives_remaining = 0;

        let good_r = compute_rating(&good, &targets);
        let poor_r = compute_rating(&poor, &targets);
        assert!(
            good_r.total > poor_r.total,
            "good {} should beat poor {}",
            good_r.total,
            poor_r.total
        );
        assert!(
            poor_r.total < 70,
            "poor clear should not be elite, got {}",
            poor_r.total
        );
        assert_eq!(poor_r.clean_bonus, 0.0);
    }

    #[test]
    fn rating_clamps_to_0_100() {
        let targets = RatingTargets {
            par_time_secs: 10.0,
            target_max_combo: 10,
            starting_lives: 3,
        };
        let mut stats = base_stats();
        stats.lives_remaining = 100;
        stats.max_combo = 999;
        stats.elapsed_secs = 0.1;
        stats.deaths = 0;
        let r = compute_rating(&stats, &targets);
        assert!(r.total <= 100);
    }

    #[test]
    fn grade_bands_match_spec() {
        assert_eq!(grade_from_rating(0), Grade::D);
        assert_eq!(grade_from_rating(39), Grade::D);
        assert_eq!(grade_from_rating(40), Grade::C);
        assert_eq!(grade_from_rating(60), Grade::B);
        assert_eq!(grade_from_rating(75), Grade::A);
        assert_eq!(grade_from_rating(90), Grade::S);
        assert_eq!(grade_from_rating(97), Grade::Ss);
        assert_eq!(grade_from_rating(100), Grade::Ss);
    }

    #[test]
    fn speed_factor_edges() {
        assert!((speed_factor(5.0, 10.0) - 1.0).abs() < 1e-5);
        assert!((speed_factor(10.0, 10.0) - 1.0).abs() < 1e-5);
        assert!((speed_factor(25.0, 10.0) - 0.0).abs() < 1e-5);
        let mid = speed_factor(17.5, 10.0);
        assert!(mid > 0.0 && mid < 1.0, "mid={mid}");
    }

    #[test]
    fn targets_scale_with_brick_count() {
        let small = RatingTargets::from_level(10, 0.3, 3);
        let large = RatingTargets::from_level(80, 0.3, 3);
        assert!(large.par_time_secs > small.par_time_secs);
        assert!(large.target_max_combo >= small.target_max_combo);
    }
}
