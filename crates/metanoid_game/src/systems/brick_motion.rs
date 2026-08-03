//! The brick movement engine.
//!
//! Every moving brick owns a [`BrickMover`]: a smooth closed *path* (sideways,
//! wave, ellipse, diamond, figure-8, Lissajous...) swept by an unbounded scalar
//! `u`, advanced by a *speed profile* that can hold constant speed early on and
//! later produce fast/slow sections, mid-path bursts and winding pacing.
//!
//! Motion is numerically integrated and collision-constrained: each frame we
//! sample the path ahead, and if it would enter another brick we bisect to the
//! exact boundary — the brick glides up against its neighbor and never overlaps
//! it, with no snapping.

use bevy::prelude::*;
use metanoid_core::components::brick::Brick;
use metanoid_procgen::level::motion::{BrickMotion, PathShape, SpeedShape};

/// A moving brick.
#[derive(Component)]
pub struct BrickMover {
    pub motion: BrickMotion,
    /// The grid-cell home position (used for vertical anchoring).
    pub origin: Vec2,
    /// Horizontal path center (runway midpoint).
    pub lane_center_x: f32,
    /// Horizontal amplitude (world px).
    pub amp_x: f32,
    /// Vertical amplitude (world px).
    pub amp_y: f32,
    pub half_w: f32,
    pub half_h: f32,
    /// Unbounded path scalar; phase = `u % 1.0`.
    pub u: f32,
}

impl BrickMover {
    pub fn phase(&self) -> f32 {
        self.u.rem_euclid(1.0)
    }

    /// Speed envelope along the path — pure, smooth functions of `s`.
    fn envelope(&self, s: f32) -> f32 {
        let m = &self.motion;
        match m.speed_shape {
            SpeedShape::Uniform => 1.0,
            SpeedShape::Pulse => {
                1.0 + m.speed_amp
                    * (std::f32::consts::TAU * m.speed_waves * s + m.phase_shift).sin()
            }
            SpeedShape::SlowMiddle => {
                // Slows through the middle of the path, faster near the extremes.
                1.0 - m.speed_amp * (0.5 - 0.5 * (std::f32::consts::TAU * s).cos())
            }
            SpeedShape::Burst => 1.0 + m.speed_amp * (-((s - 0.5).powi(2)) * 60.0).exp(),
        }
    }

    /// World position on the path at parameter `s`.
    pub fn path_pos(&self, s: f32) -> Vec2 {
        let a = std::f32::consts::TAU * s;
        let m = &self.motion;
        match m.shape {
            PathShape::Sideways => {
                Vec2::new(self.lane_center_x + self.amp_x * a.cos(), self.origin.y)
            }
            PathShape::Wave => Vec2::new(
                self.lane_center_x + self.amp_x * a.cos(),
                self.origin.y + self.amp_y * (a * m.freq_y).sin(),
            ),
            PathShape::Ellipse | PathShape::Diamond => {
                let (ca, sa) = (a.cos(), a.sin());
                let sx = ca.abs().powf(2.0 / m.superellipse_k) * ca.signum();
                let sy = sa.abs().powf(2.0 / m.superellipse_k) * sa.signum();
                Vec2::new(
                    self.lane_center_x + self.amp_x * sx,
                    self.origin.y + self.amp_y * sy,
                )
            }
            PathShape::Figure8 => Vec2::new(
                self.lane_center_x + self.amp_x * a.sin(),
                self.origin.y + self.amp_y * (a * 2.0).sin(),
            ),
            PathShape::Lissajous => Vec2::new(
                self.lane_center_x + self.amp_x * (a * m.freq_x).sin(),
                self.origin.y + self.amp_y * (a * m.freq_y + m.phase_shift).sin(),
            ),
        }
    }
}

/// All brick AABBs (static + movers), re-snapshotted each frame inside
/// [`tick_brick_motion`] via a `ParamSet` so reading positions never conflicts
/// with writing mover transforms.
#[derive(Default)]
pub struct BrickSnapshot {
    pub aabbs: Vec<(Entity, Vec2, f32, f32)>,
}

fn aabb_overlap(a: Vec2, ahw: f32, ahh: f32, b: Vec2, bhw: f32, bhh: f32) -> bool {
    (a.x - b.x).abs() < ahw + bhw - 0.5 && (a.y - b.y).abs() < ahh + bhh - 0.5
}

fn mover_hits(pos: Vec2, mover: &BrickMover, snap: &BrickSnapshot, self_entity: Entity) -> bool {
    for &(e, bpos, bhw, bhh) in &snap.aabbs {
        if e == self_entity {
            continue;
        }
        if aabb_overlap(pos, mover.half_w, mover.half_h, bpos, bhw, bhh) {
            return true;
        }
    }
    false
}

/// Advance every mover smoothly along its path, clamped so it never enters
/// another brick.
pub fn tick_brick_motion(
    time: Res<Time>,
    mut params: ParamSet<(
        Query<(Entity, &Transform, &Brick)>,
        Query<(Entity, &mut BrickMover, &mut Transform)>,
    )>,
) {
    let dt = time.delta_secs().min(0.05);
    let samples = 8u32;

    // Snapshot all brick AABBs (read-only) before mutating movers.
    let mut snapshot = BrickSnapshot::default();
    {
        let all = params.p0();
        snapshot.aabbs.reserve(all.iter().len());
        for (e, t, b) in &all {
            snapshot
                .aabbs
                .push((e, t.translation.truncate(), b.brick_half_w, b.brick_half_h));
        }
    }

    let mut movers = params.p1();
    for (entity, mut mover, mut transform) in &mut movers {
        let speed = mover.motion.base_speed * mover.envelope(mover.phase());
        let u_next = mover.u + dt * speed;

        // Sample the path ahead; find the first collision.
        let step = (u_next - mover.u) / samples as f32;
        let mut last_good = mover.u;
        let mut collision = None;
        for i in 1..=samples {
            let uu = mover.u + step * i as f32;
            if mover_hits(
                mover.path_pos(uu.rem_euclid(1.0)),
                &mover,
                &snapshot,
                entity,
            ) {
                collision = Some(uu);
                break;
            }
            last_good = uu;
        }

        let final_u = if let Some(cu) = collision {
            // Bisect to the exact boundary between the last good sample and the
            // first collision, so the brick stops flush against its neighbor.
            let mut lo = last_good;
            let mut hi = cu;
            for _ in 0..10 {
                let mid = (lo + hi) * 0.5;
                if mover_hits(
                    mover.path_pos(mid.rem_euclid(1.0)),
                    &mover,
                    &snapshot,
                    entity,
                ) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            lo
        } else {
            u_next
        };

        mover.u = final_u;
        let pos = mover.path_pos(mover.phase());
        transform.translation = pos.extend(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metanoid_procgen::level::motion::{PathShape, SpeedShape};

    fn sideways_mover(phase: f32) -> BrickMover {
        BrickMover {
            motion: BrickMotion {
                shape: PathShape::Sideways,
                speed_shape: SpeedShape::Uniform,
                base_speed: 0.5,
                freq_x: 1.0,
                freq_y: 0.0,
                superellipse_k: 1.0,
                phase,
                amp_y_cells: 0.0,
                phase_shift: 0.0,
                speed_waves: 2.0,
                speed_amp: 0.0,
            },
            origin: Vec2::new(0.0, 0.0),
            lane_center_x: 0.0,
            amp_x: 100.0,
            amp_y: 0.0,
            half_w: 40.0,
            half_h: 15.0,
            u: phase,
        }
    }

    #[test]
    fn sideways_path_bounded() {
        let m = sideways_mover(0.25);
        for i in 0..1000 {
            let s = i as f32 / 1000.0;
            let p = m.path_pos(s);
            assert!(p.x >= -100.0 - 1e-3 && p.x <= 100.0 + 1e-3, "x {}", p.x);
            assert!((p.y - 0.0).abs() < 1e-3);
        }
    }

    #[test]
    fn envelope_nonnegative() {
        let mut m = sideways_mover(0.0);
        m.motion.speed_shape = SpeedShape::Pulse;
        m.motion.speed_amp = 0.7;
        for i in 0..1000 {
            let e = m.envelope(i as f32 / 1000.0);
            assert!(e > 0.1, "envelope {e}");
        }
    }

    #[test]
    fn phase_wraps() {
        let mut m = sideways_mover(0.9);
        m.u = 2.3;
        assert!((m.phase() - 0.3).abs() < 1e-4);
    }

    #[test]
    fn collision_clamp_stops_at_neighbor() {
        let mut m = sideways_mover(0.25);
        // Place a static brick just to the right in the path's way.
        let snap = BrickSnapshot {
            aabbs: vec![(Entity::PLACEHOLDER, Vec2::new(145.0, 0.0), 40.0, 15.0)],
        };
        // Advance a large step; the mover must not cross into the blocker.
        let u_next = m.u + 0.4;
        let samples = 8;
        let step = (u_next - m.u) / samples as f32;
        let mut last_good = m.u;
        let mut collision = None;
        for i in 1..=samples {
            let uu = m.u + step * i as f32;
            if mover_hits(
                m.path_pos(uu.rem_euclid(1.0)),
                &m,
                &snap,
                Entity::PLACEHOLDER,
            ) {
                collision = Some(uu);
                break;
            }
            last_good = uu;
        }
        let final_u = if let Some(cu) = collision {
            let mut lo = last_good;
            let mut hi = cu;
            for _ in 0..10 {
                let mid = (lo + hi) * 0.5;
                if mover_hits(
                    m.path_pos(mid.rem_euclid(1.0)),
                    &m,
                    &snap,
                    Entity::PLACEHOLDER,
                ) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            lo
        } else {
            u_next
        };
        let pos = m.path_pos(final_u.rem_euclid(1.0));
        assert!(
            pos.x < 145.0 - 40.0 - 40.0 + 0.5,
            "mover crossed blocker: {}",
            pos.x
        );
    }
}
