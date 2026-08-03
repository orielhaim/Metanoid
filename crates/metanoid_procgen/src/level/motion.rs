//! The brick movement engine — pure spec.
//!
//! Every moving brick follows a *path* (a smooth, closed curve in the grid
//! plane) swept by a scalar `s` (0..1 per loop). A *speed profile* controls how
//! `s` advances over time, so bricks can cruise at constant speed early on and
//! later sweep through paths with fast and slow sections, mid-path bursts and
//! winding shapes. The game layer turns this spec into world space, clamps it
//! so bricks never overlap and keeps the motion perfectly smooth.

use rand::prelude::*;

/// The geometric shape of a brick's path. All shapes are closed loops (periodic
/// in `s`), so motion never has to reverse direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathShape {
    /// Pure horizontal back-and-forth along a line segment.
    Sideways,
    /// Horizontal travel with a gentle vertical bob.
    Wave,
    /// Elliptical loop (superellipse).
    Ellipse,
    /// Sharp diamond loop.
    Diamond,
    /// Figure-eight.
    Figure8,
    /// Lissajous winding (e.g. 2:3, 3:5 ratios).
    Lissajous,
}

/// How the path parameter `s` advances — the "gears" of the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpeedShape {
    /// Constant speed around the whole loop.
    Uniform,
    /// Speed oscillates smoothly along the path (fast & slow sections).
    Pulse,
    /// Slows down through the middle of the path, fast at the extremes.
    SlowMiddle,
    /// A burst of speed around the middle of the path.
    Burst,
}

/// Full motion specification for one brick.
#[derive(Debug, Clone, Copy)]
pub struct BrickMotion {
    pub shape: PathShape,
    pub speed_shape: SpeedShape,
    /// Path-parameter advance per second (1.0 = one full loop).
    pub base_speed: f32,
    /// Horizontal sweeps per loop.
    pub freq_x: f32,
    /// Vertical sweeps per loop.
    pub freq_y: f32,
    /// Superellipse exponent (1.0 = diamond, 2.0 = ellipse).
    pub superellipse_k: f32,
    /// Initial path offset 0..1.
    pub phase: f32,
    /// Vertical amplitude in grid cells (bounded by the free vertical runway).
    pub amp_y_cells: f32,
    /// Lissajous phase shift (radians).
    pub phase_shift: f32,
    /// Speed-oscillation count per loop.
    pub speed_waves: f32,
    /// Speed-oscillation amplitude 0..1 (0 = uniform).
    pub speed_amp: f32,
}

impl Default for BrickMotion {
    fn default() -> Self {
        Self {
            shape: PathShape::Sideways,
            speed_shape: SpeedShape::Uniform,
            base_speed: 0.4,
            freq_x: 1.0,
            freq_y: 0.0,
            superellipse_k: 1.0,
            phase: 0.0,
            amp_y_cells: 0.0,
            phase_shift: 0.0,
            speed_waves: 2.0,
            speed_amp: 0.0,
        }
    }
}

fn tier_of(progress: f32) -> u8 {
    if progress < 0.32 {
        0
    } else if progress < 0.66 {
        1
    } else {
        2
    }
}

/// Generate a deterministic motion spec for one brick.
///
/// Difficulty tiers (within a biome):
/// - Tier 0: straight sideways, constant slow speed.
/// - Tier 1: sideways / wave / ellipse, gentle speed pulses.
/// - Tier 2: winding shapes (figure-8, Lissajous, diamonds), bursts and
///   variable pacing, faster overall.
///
/// `free_up` / `free_down` are the empty cells above/below the brick's column;
/// they bound vertical amplitude so paths only wander where there is room.
pub fn generate_motion(
    rng: &mut impl Rng,
    progress: f32,
    chaos: f32,
    free_up: usize,
    free_down: usize,
) -> BrickMotion {
    let tier = tier_of(progress);

    let shape = match tier {
        0 => PathShape::Sideways,
        1 => match rng.random_range(0..3) {
            0 => PathShape::Sideways,
            1 => PathShape::Wave,
            _ => PathShape::Ellipse,
        },
        _ => match rng.random_range(0..4) {
            0 => PathShape::Ellipse,
            1 => PathShape::Diamond,
            2 => PathShape::Figure8,
            _ => PathShape::Lissajous,
        },
    };

    let speed_shape = match tier {
        0 => SpeedShape::Uniform,
        1 => match rng.random_range(0..3) {
            0 => SpeedShape::Uniform,
            1 => SpeedShape::Pulse,
            _ => SpeedShape::SlowMiddle,
        },
        _ => match rng.random_range(0..4) {
            0 => SpeedShape::Pulse,
            1 => SpeedShape::Pulse,
            2 => SpeedShape::SlowMiddle,
            _ => SpeedShape::Burst,
        },
    };

    let base_speed = match tier {
        0 => 0.28 + rng.random::<f32>() * 0.2,
        1 => 0.42 + rng.random::<f32>() * 0.3,
        _ => 0.62 + rng.random::<f32>() * 0.5,
    } * (1.0 + chaos * 0.35);

    // Vertical amplitude is bounded by the free runway above/below.
    let max_y = free_up.min(free_down) as f32;
    let amp_y_cells = match shape {
        PathShape::Sideways => 0.0,
        _ if max_y >= 1.0 => 0.35 + rng.random::<f32>() * (0.35 * max_y).min(1.2),
        _ => 0.0,
    };

    // Without vertical room, winding shapes degrade to horizontal motion.
    let shape = if amp_y_cells < 0.2
        && matches!(
            shape,
            PathShape::Lissajous | PathShape::Diamond | PathShape::Figure8
        ) {
        PathShape::Sideways
    } else {
        shape
    };

    let (freq_x, freq_y) = match shape {
        PathShape::Sideways => (1.0, 0.0),
        PathShape::Wave => (1.0, 2.0 + rng.random_range(0..2) as f32),
        PathShape::Ellipse | PathShape::Diamond => (1.0, 1.0),
        PathShape::Figure8 => (1.0, 2.0),
        PathShape::Lissajous => {
            let fx = if rng.random::<f32>() < 0.5 { 2.0 } else { 3.0 };
            let fy = [3.0, 4.0, 5.0][rng.random_range(0..3)];
            (fx, fy)
        }
    };

    let speed_amp = match tier {
        0 => 0.0,
        1 => 0.3 + rng.random::<f32>() * 0.25,
        _ => 0.4 + rng.random::<f32>() * 0.35,
    };
    let speed_waves = 1.0 + rng.random::<f32>() * 2.5;

    BrickMotion {
        shape,
        speed_shape,
        base_speed,
        freq_x,
        freq_y,
        superellipse_k: if shape == PathShape::Diamond {
            1.0
        } else {
            2.0
        },
        phase: rng.random::<f32>(),
        amp_y_cells,
        phase_shift: rng.random::<f32>() * std::f32::consts::TAU,
        speed_waves,
        speed_amp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn early_biomes_are_sideways_and_uniform() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        for _ in 0..200 {
            let m = generate_motion(&mut rng, 0.05, 0.3, 0, 0);
            assert_eq!(m.shape, PathShape::Sideways, "early shape");
            assert_eq!(m.speed_shape, SpeedShape::Uniform, "early speed");
            assert_eq!(m.amp_y_cells, 0.0);
        }
    }

    #[test]
    fn late_biomes_bring_variety() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let mut shapes = std::collections::HashSet::new();
        let mut speeds = std::collections::HashSet::new();
        for _ in 0..400 {
            let m = generate_motion(&mut rng, 0.95, 0.7, 3, 3);
            shapes.insert(m.shape);
            speeds.insert(m.speed_shape);
            assert!(m.base_speed > 0.6, "late speed {}", m.base_speed);
            assert!(m.amp_y_cells > 0.0, "late should have vertical room");
        }
        assert!(shapes.len() >= 3, "expected varied shapes, got {shapes:?}");
        assert!(speeds.len() >= 3, "expected varied speed profiles");
    }

    #[test]
    fn speed_amp_bounded() {
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        for _ in 0..500 {
            let m = generate_motion(&mut rng, 0.9, 1.0, 5, 5);
            assert!((0.0..=0.75).contains(&m.speed_amp));
            assert!(m.speed_waves >= 1.0);
            assert!((0.0..1.0).contains(&m.phase));
        }
    }

    #[test]
    fn deterministic() {
        let mut a = ChaCha8Rng::seed_from_u64(7);
        let mut b = ChaCha8Rng::seed_from_u64(7);
        let ma = generate_motion(&mut a, 0.5, 0.4, 2, 2);
        let mb = generate_motion(&mut b, 0.5, 0.4, 2, 2);
        assert_eq!(format!("{ma:?}"), format!("{mb:?}"));
    }
}
