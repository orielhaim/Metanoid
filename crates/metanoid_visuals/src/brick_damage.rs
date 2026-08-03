//! Dynamic brick damage: every impact of the ball carves a unique crack pattern
//! into the brick at the exact point of contact. Repeated hits build up a
//! distinct patchwork of cracks, so no two bricks ever look the same.

use bevy::prelude::*;

/// A single crack: a polyline in the brick's local UV space (0..1).
#[derive(Debug, Clone)]
pub struct Crack {
    pub points: Vec<Vec2>,
    pub width: f32,
    /// 0..1 — how deep / dark this crack is (scales with impact severity).
    pub depth: f32,
}

/// Deterministic pseudo-random from a seed.
fn rand01(seed: &mut u64) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    ((*seed % 10000) as f32) / 10000.0
}

pub fn add_impact(cracks: &mut Vec<Crack>, local_uv: Vec2, mut seed: u64, severity: f32) {
    let sev = severity.clamp(0.0, 1.0);
    let count = 2 + ((rand01(&mut seed) * 2.0 + sev * 3.0) as usize).min(3); // 2..5
    let base_angle = rand01(&mut seed) * std::f32::consts::TAU;

    for i in 0..count {
        let angle = base_angle
            + (i as f32 / count as f32) * std::f32::consts::TAU
            + (rand01(&mut seed) - 0.5) * 0.6;
        let len = 0.05 + sev * 0.30 + rand01(&mut seed) * 0.08;
        let segments = 3 + (rand01(&mut seed) * 3.0) as usize;
        let step = len / segments as f32;

        let mut points = vec![local_uv];
        let mut pos = local_uv;
        let mut ang = angle;
        for _ in 0..segments {
            ang += (rand01(&mut seed) - 0.5) * 1.3;
            pos += Vec2::new(ang.cos(), ang.sin()) * step;
            // Keep cracks inside the brick face with a small margin.
            let p = pos.clamp(Vec2::splat(0.03), Vec2::splat(0.97));
            points.push(p);
            pos = p;
            if (p - local_uv).length() > 0.55 {
                break;
            }
        }

        cracks.push(Crack {
            points,
            width: 0.7 + sev * 2.0 + rand01(&mut seed) * 0.8,
            depth: 0.25 + sev * 0.6,
        });
    }
}

fn pixel_idx(x: i32, y: i32, w: u32, h: u32) -> Option<usize> {
    if x < 0 || y < 0 || x as u32 >= w || y as u32 >= h {
        return None;
    }
    Some(((y as u32 * w + x as u32) * 4) as usize)
}

/// Darken an RGBA8 (sRGB) pixel buffer around a point.
fn darken_pixel(buf: &mut [u8], idx: usize, amount: f32) {
    buf[idx] = (buf[idx] as f32 * (1.0 - amount)) as u8;
    buf[idx + 1] = (buf[idx + 1] as f32 * (1.0 - amount)) as u8;
    buf[idx + 2] = (buf[idx + 2] as f32 * (1.0 - amount)) as u8;
}

/// Tint a pixel toward the glow color (for "molten" cracks on lava/explosive).
fn tint_pixel(buf: &mut [u8], idx: usize, glow: &Srgba, amount: f32) {
    buf[idx] = (buf[idx] as f32 * (1.0 - amount) + glow.red * 255.0 * amount) as u8;
    buf[idx + 1] = (buf[idx + 1] as f32 * (1.0 - amount) + glow.green * 255.0 * amount) as u8;
    buf[idx + 2] = (buf[idx + 2] as f32 * (1.0 - amount) + glow.blue * 255.0 * amount) as u8;
}

/// Bake all accumulated cracks into a base RGBA8 tile, returning new pixels.
///
/// `w`/`h` are the tile dimensions (square assumed for indexing).
/// When `molten` is true, cracks glow in `glow` color instead of just darkening.
pub fn apply_cracks(
    base: &[u8],
    w: u32,
    h: u32,
    cracks: &[Crack],
    glow: LinearRgba,
    molten: bool,
) -> Vec<u8> {
    let mut out = base.to_vec();
    if out.len() != (w * h * 4) as usize {
        return out;
    }
    let glow: Srgba = Srgba::from(glow);

    for crack in cracks {
        let width = crack.width.round() as i32;
        let depth = crack.depth.clamp(0.1, 1.0);
        // Draw each segment with a soft falloff for organic edges.
        for pair in crack.points.windows(2) {
            let a = pair[0];
            let b = pair[1];
            let steps = ((a.distance(b) * 24.0).ceil() as u32).max(2);
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let p = a.lerp(b, t);
                let cx = (p.x * w as f32) as i32;
                let cy = (p.y * h as f32) as i32;
                for dy in -width..=width {
                    for dx in -width..=width {
                        let d = ((dx * dx + dy * dy) as f32).sqrt();
                        if d > width as f32 {
                            continue;
                        }
                        let falloff = 1.0 - d / (width as f32 + 1.0);
                        if let Some(idx) = pixel_idx(cx + dx, cy + dy, w, h) {
                            if molten {
                                tint_pixel(&mut out, idx, &glow, 0.3 * falloff * depth);
                            } else {
                                darken_pixel(&mut out, idx, 0.6 * falloff * depth);
                            }
                        }
                    }
                }
            }
        }

        // Impact pit: a small dark (or glowing) crater at the anchor. Its size
        // and depth scale with severity.
        let pit = crack.points[0];
        let px = (pit.x * w as f32) as i32;
        let py = (pit.y * h as f32) as i32;
        let pr = (crack.width.round() as i32) + (depth * 3.0) as i32;
        for dy in -pr..=pr {
            for dx in -pr..=pr {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                if d > pr as f32 {
                    continue;
                }
                let falloff = 1.0 - d / (pr as f32 + 1.0);
                if let Some(idx) = pixel_idx(px + dx, py + dy, w, h) {
                    if molten {
                        tint_pixel(&mut out, idx, &glow, 0.55 * falloff * depth);
                    } else {
                        darken_pixel(&mut out, idx, 0.75 * falloff * depth);
                    }
                }
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile() -> (Vec<u8>, u32) {
        let w = 32u32;
        (vec![200u8; (w * w * 4) as usize], w)
    }

    #[test]
    fn impact_creates_cracks() {
        let mut cracks = Vec::new();
        add_impact(&mut cracks, Vec2::new(0.5, 0.5), 0x1234, 0.6);
        assert!(!cracks.is_empty());
        assert!(cracks.len() >= 2);
        for c in &cracks {
            assert!(c.points.len() >= 2);
            assert!(c.points[0].distance(Vec2::new(0.5, 0.5)) < 1e-4);
        }
    }

    #[test]
    fn impacts_are_unique() {
        let mut a = Vec::new();
        let mut b = Vec::new();
        add_impact(&mut a, Vec2::new(0.2, 0.2), 0x1234, 0.6);
        add_impact(&mut b, Vec2::new(0.8, 0.7), 0x5678, 0.6);
        let sa: Vec<Vec2> = a.iter().flat_map(|c| c.points.clone()).collect();
        let sb: Vec<Vec2> = b.iter().flat_map(|c| c.points.clone()).collect();
        assert_ne!(sa, sb);
    }

    #[test]
    fn bake_changes_pixels() {
        let (base, w) = tile();
        let mut cracks = Vec::new();
        add_impact(&mut cracks, Vec2::new(0.5, 0.5), 42, 0.6);
        let out = apply_cracks(&base, w, w, &cracks, LinearRgba::WHITE, false);
        assert_ne!(base, out);
        assert_eq!(out.len(), base.len());
    }

    #[test]
    fn bake_non_square_tile_no_panic() {
        // 64x32 tiles (brick aspect) must not read out of bounds.
        let w = 64u32;
        let h = 32u32;
        let base = vec![200u8; (w * h * 4) as usize];
        let mut cracks = Vec::new();
        add_impact(&mut cracks, Vec2::new(0.5, 0.5), 1, 0.8);
        add_impact(&mut cracks, Vec2::new(0.05, 0.05), 2, 0.8);
        add_impact(&mut cracks, Vec2::new(0.95, 0.95), 3, 0.8);
        let out = apply_cracks(&base, w, h, &cracks, LinearRgba::WHITE, true);
        assert_eq!(out.len(), base.len());
        assert_ne!(base, out);
    }

    #[test]
    fn same_seed_same_impact() {
        let mut a = Vec::new();
        let mut b = Vec::new();
        add_impact(&mut a, Vec2::new(0.3, 0.4), 99, 0.5);
        add_impact(&mut b, Vec2::new(0.3, 0.4), 99, 0.5);
        assert_eq!(format!("{a:?}"), format!("{b:?}"));
    }

    #[test]
    fn harder_hits_damage_deeper() {
        let mut light = Vec::new();
        let mut hard = Vec::new();
        add_impact(&mut light, Vec2::new(0.5, 0.5), 7, 0.1);
        add_impact(&mut hard, Vec2::new(0.5, 0.5), 7, 1.0);
        let light_depth: f32 = light.iter().map(|c| c.depth).sum();
        let hard_depth: f32 = hard.iter().map(|c| c.depth).sum();
        assert!(
            hard_depth > light_depth * 2.0,
            "hard {hard_depth} light {light_depth}"
        );
        let hard_cracks = hard.len();
        assert!(hard_cracks >= light.len());
        let light_len: f32 = light.iter().map(|c| c.points.len() as f32).sum();
        let hard_len: f32 = hard.iter().map(|c| c.points.len() as f32).sum();
        assert!(hard_len >= light_len);
    }
}
