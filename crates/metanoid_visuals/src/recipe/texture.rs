//! Procedural pixel baker: generates brick / entity textures entirely from
//! noise + recipes, with no asset files involved.

use bevy::prelude::*;

use super::TextureKind;

/// A raw RGBA8 pixel buffer (sRGB byte space).
pub struct Pixels {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}

impl Pixels {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            data: vec![0; (w * h * 4) as usize],
        }
    }

    pub fn set(&mut self, x: u32, y: u32, c: LinearRgba) {
        if x >= self.w || y >= self.h {
            return;
        }
        let s: Srgba = Srgba::from(c);
        let i = ((y * self.w + x) * 4) as usize;
        self.data[i] = (s.red * 255.0).clamp(0.0, 255.0) as u8;
        self.data[i + 1] = (s.green * 255.0).clamp(0.0, 255.0) as u8;
        self.data[i + 2] = (s.blue * 255.0).clamp(0.0, 255.0) as u8;
        self.data[i + 3] = (s.alpha * 255.0).clamp(0.0, 255.0) as u8;
    }

    pub fn mix_at(&mut self, x: u32, y: u32, c: LinearRgba, amount: f32) {
        if x >= self.w || y >= self.h {
            return;
        }
        let s: Srgba = Srgba::from(c);
        let i = ((y * self.w + x) * 4) as usize;
        let lerp = |a: u8, b: f32| -> u8 {
            (a as f32 + (b * 255.0 - a as f32) * amount).clamp(0.0, 255.0) as u8
        };
        self.data[i] = lerp(self.data[i], s.red);
        self.data[i + 1] = lerp(self.data[i + 1], s.green);
        self.data[i + 2] = lerp(self.data[i + 2], s.blue);
        self.data[i + 3] = lerp(self.data[i + 3], s.alpha);
    }

    pub fn darken_at(&mut self, x: u32, y: u32, amount: f32) {
        if x >= self.w || y >= self.h {
            return;
        }
        let i = ((y * self.w + x) * 4) as usize;
        let d = 1.0 - amount;
        self.data[i] = (self.data[i] as f32 * d).clamp(0.0, 255.0) as u8;
        self.data[i + 1] = (self.data[i + 1] as f32 * d).clamp(0.0, 255.0) as u8;
        self.data[i + 2] = (self.data[i + 2] as f32 * d).clamp(0.0, 255.0) as u8;
    }
}

// ---- deterministic value noise --------------------------------------------

fn hash2(x: u32, y: u32, seed: u64) -> f32 {
    let mut h = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((x as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9))
        .wrapping_add((y as u64).wrapping_mul(0x94D0_49BB_1331_11EB));
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    (h >> 8) as f32 / (1u64 << 56) as f32
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn vnoise(x: f32, y: f32, seed: u64) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let a = hash2(xi as u32, yi as u32, seed);
    let b = hash2(xi as u32 + 1, yi as u32, seed);
    let c = hash2(xi as u32, yi as u32 + 1, seed);
    let d = hash2(xi as u32 + 1, yi as u32 + 1, seed);
    let u = smooth(xf);
    let v = smooth(yf);
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

fn fbm(x: f32, y: f32, seed: u64, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for i in 0..octaves {
        sum += vnoise(x * freq, y * freq, seed + i as u64 * 7919) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

fn exp_falloff(d: f32, k: f32) -> f32 {
    (-d * d * k).exp()
}

// ---- per-kind generators: return (shade, emissive) ------------------------

fn wood(nx: f32, ny: f32, seed: u64) -> (f32, f32) {
    let grain = fbm(nx * 0.12, ny * 1.4, seed, 4);
    let ring = (0.5 + 0.5 * (ny * 0.55 + grain * 4.0).sin()) * 0.35 + grain * 0.5;
    let shade = 0.68 + ring * 0.6;
    let knot = hash2((nx * 0.12) as u32, (ny * 0.4) as u32, seed ^ 0xDEADBEEF);
    let emissive = if knot > 0.985 { 0.5 } else { 0.05 };
    (shade, emissive)
}

fn stone(nx: f32, ny: f32, seed: u64) -> (f32, f32) {
    let n = fbm(nx * 0.6, ny * 0.6, seed, 3);
    let speckle = hash2(nx as u32 * 7, ny as u32 * 13, seed ^ 0xABCDEF);
    let shade = (0.72 + n * 0.5) * if speckle > 0.96 { 0.55 } else { 1.0 };
    (shade, 0.03)
}

fn lava(nx: f32, ny: f32, seed: u64) -> (f32, f32) {
    let n = fbm(nx * 0.7, ny * 0.7, seed, 3);
    let shade = 0.45 + n * 0.55;
    // Sharp filamentary cracks from a power-boosted noise field.
    let crack = vnoise(nx * 0.9 + 13.0, ny * 0.9, seed ^ 0x1337).powf(14.0);
    let glow: f32 = exp_falloff(1.0 - crack, 2.0).clamp(0.0, 1.0);
    let emissive: f32 = if crack > 0.8 { 0.9 } else { 0.0 };
    (shade * (1.0 - glow * 0.3), emissive.max(glow * 0.5))
}

fn ice(nx: f32, ny: f32, seed: u64) -> (f32, f32) {
    let cell = 10.0;
    let cx = (nx / cell).floor();
    let cy = (ny / cell).floor();
    let fx = (nx / cell).fract();
    let fy = (ny / cell).fract();
    let cell_val = hash2(cx as u32, cy as u32, seed);
    let shade = 0.55 + cell_val * 0.6;
    // bright facet edges
    let edge = (fx.min(1.0 - fx)).min(fy.min(1.0 - fy)).clamp(0.0, 0.5);
    let emissive = if edge < 0.12 { 0.7 } else { 0.15 };
    (shade, emissive)
}

fn crystal(nx: f32, ny: f32, seed: u64) -> (f32, f32) {
    let cell = 14.0;
    let cx = (nx / cell).floor();
    let cy = (ny / cell).floor();
    let fx = (nx / cell).fract();
    let fy = (ny / cell).fract();
    let cell_val = hash2(cx as u32, cy as u32, seed ^ 0xC0FFEE);
    // Diagonal facet shading.
    let d = (fx + fy).fract();
    let shade = 0.45 + cell_val * 0.4 + d * 0.5;
    let edge = (fx.min(1.0 - fx)).min(fy.min(1.0 - fy));
    let emissive = if edge < 0.09 { 1.0 } else { 0.25 };
    (shade, emissive)
}

fn neon(nx: f32, ny: f32, seed: u64) -> (f32, f32) {
    let gx = 14.0;
    let gy = 7.0;
    let dgx = (nx.rem_euclid(gx)).min(gx - nx.rem_euclid(gx));
    let dgy = (ny.rem_euclid(gy)).min(gy - ny.rem_euclid(gy));
    let d = dgx.min(dgy);
    let base_noise = fbm(nx * 0.5, ny * 0.5, seed, 2);
    let shade = 0.35 + base_noise * 0.25;
    let emissive = exp_falloff(d, 3.0).clamp(0.0, 1.0) * (0.8 + base_noise);
    (shade, emissive)
}

fn metal(nx: f32, ny: f32, seed: u64) -> (f32, f32) {
    let brushed = fbm(nx * 0.1, ny * 0.9, seed, 3);
    let mut shade = 0.55 + brushed * 0.5;
    let rivet = (nx.rem_euclid(30.0) < 3.0) && (ny.rem_euclid(14.0) < 3.0);
    let mut emissive = 0.05;
    if rivet {
        shade *= 0.4;
        emissive = 0.35;
    }
    (shade, emissive)
}

fn cosmic(nx: f32, ny: f32, seed: u64) -> (f32, f32) {
    let n = fbm(nx * 0.4, ny * 0.4, seed, 4);
    let shade = 0.2 + n * 0.35;
    let star = hash2((nx / 2.0) as u32, (ny / 2.0) as u32, seed ^ 0x5EED);
    let emissive = if star > 0.995 { 1.0 } else { 0.05 };
    (shade, emissive)
}

fn charred(nx: f32, ny: f32, seed: u64) -> (f32, f32) {
    let n = fbm(nx * 0.7, ny * 0.7, seed, 3);
    let shade = 0.3 + n * 0.3;
    let crack = vnoise(nx * 0.9 + 7.0, ny * 0.9, seed ^ 0xBAD).powf(16.0);
    let emissive = if crack > 0.82 { 0.7 } else { 0.04 };
    (shade, emissive)
}

fn apply_damage(p: &mut Pixels, seed: u64, damage: f32) {
    // damage in 0..1; draws dark cracks over the tile.
    let crack_count = (2 + (damage * 6.0) as u32).min(7);
    let mut cracks: Vec<(f32, f32, f32)> = Vec::new();
    let mut h = seed;
    for i in 0..crack_count {
        let sx = (h % 64) as f32;
        h ^= h << 13;
        h ^= h >> 17;
        h ^= h << 5;
        let sy = (h % 32) as f32;
        h ^= h << 13;
        h ^= h >> 17;
        h ^= h << 5;
        let angle = ((h % 1000) as f32 / 1000.0) * 3.14159 * 2.0;
        cracks.push((sx, sy, angle));
        let _ = i;
    }
    let steps = 24;
    for (sx, sy, angle) in &cracks {
        let dirx = angle.cos();
        let diry = angle.sin();
        for s in 0..steps {
            let t = s as f32 / steps as f32;
            let wx = sx + dirx * t * 60.0 + diry * (s as f32 * 0.7).sin() * 3.0;
            let wy = sy + diry * t * 32.0;
            let x = wx.round() as u32;
            let y = wy.round() as u32;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let px = x as i32 + dx;
                    let py = y as i32 + dy;
                    if px >= 0 && px < p.w as i32 && py >= 0 && py < p.h as i32 {
                        p.darken_at(px as u32, py as u32, 0.55 * damage);
                    }
                }
            }
        }
    }
}

/// Bake a brick texture tile.
pub fn bake_brick(
    kind: TextureKind,
    base: LinearRgba,
    glow: LinearRgba,
    w: u32,
    h: u32,
    seed: u64,
    damage: f32,
) -> Pixels {
    let mut p = Pixels::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let (nx, ny) = (x as f32, y as f32);
            let (shade, emissive) = match kind {
                TextureKind::Wood => wood(nx, ny, seed),
                TextureKind::Stone => stone(nx, ny, seed),
                TextureKind::Lava => lava(nx, ny, seed),
                TextureKind::Ice => ice(nx, ny, seed),
                TextureKind::Crystal => crystal(nx, ny, seed),
                TextureKind::Neon => neon(nx, ny, seed),
                TextureKind::Metal => metal(nx, ny, seed),
                TextureKind::Cosmic => cosmic(nx, ny, seed),
                TextureKind::Charred => charred(nx, ny, seed),
            };
            let mut col = base.mix(&glow, emissive.clamp(0.0, 1.0));
            let s: Srgba = Srgba::from(col);
            col = LinearRgba::from(Srgba::new(
                (s.red * shade).clamp(0.0, 1.0),
                (s.green * shade).clamp(0.0, 1.0),
                (s.blue * shade).clamp(0.0, 1.0),
                s.alpha,
            ));
            p.set(x, y, col);
        }
    }
    if damage > 0.05 {
        apply_damage(&mut p, seed ^ 0x5F00D, damage);
    }
    p
}

/// Bake a seamless-ish tile for ground / glow.
pub fn bake_noise_tile(
    base: LinearRgba,
    glow: LinearRgba,
    w: u32,
    h: u32,
    seed: u64,
    grain: f32,
) -> Pixels {
    let mut p = Pixels::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let (nx, ny) = (x as f32, y as f32);
            let n = fbm(nx * 0.25, ny * 0.25, seed, 3);
            let shade = 0.7 + n * 0.5;
            let col = base.mix(&glow, n * grain);
            let s: Srgba = Srgba::from(col);
            p.set(
                x,
                y,
                LinearRgba::from(Srgba::new(
                    (s.red * shade).clamp(0.0, 1.0),
                    (s.green * shade).clamp(0.0, 1.0),
                    (s.blue * shade).clamp(0.0, 1.0),
                    s.alpha,
                )),
            );
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_bounds() {
        let mut p = Pixels::new(64, 32);
        p.set(0, 0, LinearRgba::WHITE);
        p.set(63, 31, LinearRgba::BLACK);
        p.set(100, 100, LinearRgba::WHITE); // no panic
        assert_eq!(p.data.len(), 64 * 32 * 4);
    }

    #[test]
    fn all_kinds_bake() {
        let base = LinearRgba::from(Srgba::new(0.5, 0.4, 0.3, 1.0));
        let glow = LinearRgba::from(Srgba::new(1.0, 0.6, 0.2, 1.0));
        for kind in [
            TextureKind::Wood,
            TextureKind::Stone,
            TextureKind::Lava,
            TextureKind::Ice,
            TextureKind::Crystal,
            TextureKind::Neon,
            TextureKind::Metal,
            TextureKind::Cosmic,
            TextureKind::Charred,
        ] {
            let p = bake_brick(kind, base, glow, 64, 32, 42, 0.6);
            assert_eq!(p.data.len(), 64 * 32 * 4);
            // Some alpha byte present
            assert!(p.data[3] > 0);
        }
    }

    #[test]
    fn damage_changes_pixels() {
        let base = LinearRgba::WHITE;
        let glow = LinearRgba::BLACK;
        let a = bake_brick(TextureKind::Stone, base, glow, 64, 32, 1, 0.0);
        let b = bake_brick(TextureKind::Stone, base, glow, 64, 32, 1, 0.9);
        assert_ne!(a.data, b.data);
    }
}
