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

    /// Alpha-composite an sRGB color over the pixel.
    pub fn blend(&mut self, x: u32, y: u32, color: Srgba, alpha: f32) {
        if x >= self.w || y >= self.h {
            return;
        }
        let i = ((y * self.w + x) * 4) as usize;
        let a = alpha.clamp(0.0, 1.0);
        self.data[i] =
            (self.data[i] as f32 * (1.0 - a) + color.red * 255.0 * a).clamp(0.0, 255.0) as u8;
        self.data[i + 1] =
            (self.data[i + 1] as f32 * (1.0 - a) + color.green * 255.0 * a).clamp(0.0, 255.0) as u8;
        self.data[i + 2] =
            (self.data[i + 2] as f32 * (1.0 - a) + color.blue * 255.0 * a).clamp(0.0, 255.0) as u8;
    }
}

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

pub fn bake_radial_glow(size: u32) -> Pixels {
    let mut p = Pixels::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let nx = (x as f32 + 0.5) / size as f32;
            let ny = (y as f32 + 0.5) / size as f32;
            let d = ((nx - 0.5) * 2.0).hypot((ny - 0.5) * 2.0);
            let alpha = (1.0 - d).clamp(0.0, 1.0).powi(2);
            p.set(x, y, LinearRgba::from(Srgba::new(1.0, 1.0, 1.0, alpha)));
        }
    }
    p
}

/// Thin soft ring sprite (for powerup auras / shields).
pub fn bake_ring(size: u32, thickness: f32) -> Pixels {
    let mut p = Pixels::new(size, size);
    let radius = size as f32 * 0.40;
    for y in 0..size {
        for x in 0..size {
            let nx = (x as f32 + 0.5) - size as f32 / 2.0;
            let ny = (y as f32 + 0.5) - size as f32 / 2.0;
            let d = (nx.hypot(ny) - radius).abs();
            let alpha = (1.0 - d / thickness).clamp(0.0, 1.0);
            p.set(x, y, LinearRgba::from(Srgba::new(1.0, 1.0, 1.0, alpha)));
        }
    }
    p
}

/// Icon shapes drawn on powerup textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PwIcon {
    Flame,
    BigDot,
    Dots3,
    Chevrons,
    SlowBar,
    Beams,
    UShape,
    Expand,
    Shrink,
    Ring,
    Heart,
    Cross,
    Star,
    Skull,
    Clock,
    Blocks,
    Bolt,
    Bomb,
    Burst,
    Rings,
    Arrows,
    Eclipse,
}

/// Icon + accent color per powerup kind.
fn powerup_icon(kind: metanoid_core::components::powerup::PowerUpKind) -> (PwIcon, Srgba) {
    use metanoid_core::components::powerup::PowerUpKind as K;
    let icon = match kind {
        K::Fireball => PwIcon::Flame,
        K::MegaBall => PwIcon::BigDot,
        K::SplitBall => PwIcon::Dots3,
        K::FastBall => PwIcon::Chevrons,
        K::SlowBall => PwIcon::SlowBar,
        K::LaserPaddle => PwIcon::Beams,
        K::GrabPaddle => PwIcon::UShape,
        K::ExpandPaddle => PwIcon::Expand,
        K::ShrinkPaddle => PwIcon::Shrink,
        K::Shield => PwIcon::Ring,
        K::ExtraLife => PwIcon::Heart,
        K::DoublePoints => PwIcon::Cross,
        K::LevelWarp => PwIcon::Star,
        K::KillPaddle => PwIcon::Skull,
        K::TimeSlow => PwIcon::Clock,
        K::FallingBricks => PwIcon::Blocks,
        K::Zap => PwIcon::Bolt,
        K::Explode => PwIcon::Bomb,
        K::ExpandExploding => PwIcon::Burst,
        K::Lightning => PwIcon::Bolt,
        K::Shockwave => PwIcon::Rings,
        K::ShuffleBricks => PwIcon::Arrows,
        K::Blackout => PwIcon::Eclipse,
    };
    let accent = match kind {
        K::Fireball => Srgba::new(1.0, 0.34, 0.05, 1.0),
        K::MegaBall => Srgba::new(1.0, 0.85, 0.15, 1.0),
        K::SplitBall => Srgba::new(0.2, 0.9, 0.55, 1.0),
        K::FastBall => Srgba::new(1.0, 0.6, 0.15, 1.0),
        K::SlowBall => Srgba::new(0.3, 0.6, 1.0, 1.0),
        K::LaserPaddle => Srgba::new(1.0, 0.2, 0.25, 1.0),
        K::GrabPaddle => Srgba::new(0.6, 0.3, 1.0, 1.0),
        K::ExpandPaddle => Srgba::new(0.2, 0.85, 0.9, 1.0),
        K::ShrinkPaddle => Srgba::new(0.85, 0.2, 0.5, 1.0),
        K::Shield => Srgba::new(0.25, 0.75, 1.0, 1.0),
        K::ExtraLife => Srgba::new(1.0, 0.25, 0.5, 1.0),
        K::DoublePoints => Srgba::new(1.0, 0.9, 0.4, 1.0),
        K::LevelWarp => Srgba::new(0.55, 0.9, 0.3, 1.0),
        K::KillPaddle => Srgba::new(0.4, 0.05, 0.05, 1.0),
        K::TimeSlow => Srgba::new(0.35, 0.4, 1.0, 1.0),
        K::FallingBricks => Srgba::new(0.65, 0.5, 0.3, 1.0),
        K::Zap => Srgba::new(1.0, 0.95, 0.7, 1.0),
        K::Explode => Srgba::new(1.0, 0.25, 0.1, 1.0),
        K::ExpandExploding => Srgba::new(1.0, 0.5, 0.1, 1.0),
        K::Lightning => Srgba::new(1.0, 0.95, 0.5, 1.0),
        K::Shockwave => Srgba::new(0.5, 0.55, 1.0, 1.0),
        K::ShuffleBricks => Srgba::new(0.8, 0.55, 1.0, 1.0),
        K::Blackout => Srgba::new(0.25, 0.1, 0.4, 1.0),
    };
    (icon, accent)
}

fn pw_fill_circle(p: &mut Pixels, cx: f32, cy: f32, r: f32, color: Srgba, alpha: f32) {
    let size = p.w as f32;
    let r0 = r * size;
    let (cx0, cy0) = (cx * size, cy * size);
    let rng = r0.ceil() as i32;
    for dy in -rng..=rng {
        for dx in -rng..=rng {
            if (dx as f32).hypot(dy as f32) <= r0 {
                let x = (cx0 + dx as f32).round() as u32;
                let y = (cy0 + dy as f32).round() as u32;
                p.blend(x, y, color, alpha);
            }
        }
    }
}

fn pw_fill_rect(p: &mut Pixels, x0: f32, y0: f32, x1: f32, y1: f32, color: Srgba, alpha: f32) {
    let size = p.w as f32;
    let (xa, ya) = (
        (x0 * size).min(x1 * size) as u32,
        (y0 * size).min(y1 * size) as u32,
    );
    let (xb, yb) = (
        (x0 * size).max(x1 * size) as u32,
        (y0 * size).max(y1 * size) as u32,
    );
    for y in ya..=yb {
        for x in xa..=xb {
            p.blend(x, y, color, alpha);
        }
    }
}

fn pw_ring(p: &mut Pixels, cx: f32, cy: f32, r: f32, thick: f32, color: Srgba, alpha: f32) {
    let size = p.w as f32;
    let (cx0, cy0) = (cx * size, cy * size);
    let r0 = r * size;
    let t = (thick * size).max(1.0);
    let ext = (r0 + t).ceil() as i32;
    for dy in -ext..=ext {
        for dx in -ext..=ext {
            let d = ((dx as f32).hypot(dy as f32) - r0).abs();
            if d <= t {
                p.blend(
                    (cx0 + dx as f32) as u32,
                    (cy0 + dy as f32) as u32,
                    color,
                    alpha,
                );
            }
        }
    }
}

fn pw_line(
    p: &mut Pixels,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    thick: f32,
    color: Srgba,
    alpha: f32,
) {
    let size = p.w as f32;
    let steps = ((Vec2::new(x0, y0).distance(Vec2::new(x1, y1)) * size).ceil() as u32).max(2);
    let t = (thick * size * 0.5).max(0.7);
    for i in 0..=steps {
        let u = i as f32 / steps as f32;
        let px = (x0 + (x1 - x0) * u) * size;
        let py = (y0 + (y1 - y0) * u) * size;
        pw_fill_circle(p, px / size, py / size, t / size, color, alpha);
    }
}

fn pw_draw_icon(p: &mut Pixels, icon: PwIcon) {
    let white = Srgba::new(1.0, 1.0, 1.0, 1.0);
    let dark = Srgba::new(0.08, 0.08, 0.1, 1.0);
    match icon {
        PwIcon::Flame => {
            pw_fill_circle(p, 0.5, 0.58, 0.16, white, 0.95);
            pw_fill_circle(p, 0.5, 0.34, 0.10, white, 0.9);
        }
        PwIcon::BigDot => pw_fill_circle(p, 0.5, 0.5, 0.18, white, 0.95),
        PwIcon::Dots3 => {
            for i in 0..3 {
                pw_fill_circle(p, 0.5, 0.3 + i as f32 * 0.2, 0.05, white, 0.95);
            }
        }
        PwIcon::Chevrons => {
            for i in 0..2 {
                let x = 0.3 + i as f32 * 0.28;
                pw_line(p, x, 0.68, x + 0.12, 0.5, 0.06, white, 0.95);
                pw_line(p, x + 0.12, 0.5, x, 0.32, 0.06, white, 0.95);
            }
        }
        PwIcon::SlowBar => {
            pw_fill_rect(p, 0.2, 0.44, 0.8, 0.56, white, 0.9);
            pw_line(p, 0.72, 0.6, 0.6, 0.5, 0.06, white, 0.95);
            pw_line(p, 0.72, 0.4, 0.6, 0.5, 0.06, white, 0.95);
        }
        PwIcon::Beams => {
            pw_fill_rect(p, 0.36, 0.2, 0.45, 0.8, white, 0.95);
            pw_fill_rect(p, 0.55, 0.2, 0.64, 0.8, white, 0.95);
        }
        PwIcon::UShape => {
            pw_line(p, 0.3, 0.75, 0.3, 0.42, 0.06, white, 0.95);
            pw_line(p, 0.3, 0.42, 0.7, 0.42, 0.06, white, 0.95);
            pw_line(p, 0.7, 0.42, 0.7, 0.75, 0.06, white, 0.95);
        }
        PwIcon::Expand => {
            pw_fill_rect(p, 0.2, 0.44, 0.8, 0.56, white, 0.9);
            pw_line(p, 0.32, 0.5, 0.2, 0.5, 0.05, white, 0.9);
            pw_line(p, 0.2, 0.5, 0.26, 0.44, 0.05, white, 0.9);
            pw_line(p, 0.2, 0.5, 0.26, 0.56, 0.05, white, 0.9);
            pw_line(p, 0.68, 0.5, 0.8, 0.5, 0.05, white, 0.9);
            pw_line(p, 0.8, 0.5, 0.74, 0.44, 0.05, white, 0.9);
            pw_line(p, 0.8, 0.5, 0.74, 0.56, 0.05, white, 0.9);
        }
        PwIcon::Shrink => {
            pw_fill_rect(p, 0.2, 0.44, 0.8, 0.56, white, 0.9);
            pw_line(p, 0.3, 0.44, 0.36, 0.5, 0.05, white, 0.9);
            pw_line(p, 0.36, 0.5, 0.3, 0.56, 0.05, white, 0.9);
            pw_line(p, 0.7, 0.44, 0.64, 0.5, 0.05, white, 0.9);
            pw_line(p, 0.64, 0.5, 0.7, 0.56, 0.05, white, 0.9);
        }
        PwIcon::Ring => pw_ring(p, 0.5, 0.5, 0.2, 0.05, white, 0.95),
        PwIcon::Heart => {
            pw_fill_circle(p, 0.36, 0.38, 0.12, white, 0.95);
            pw_fill_circle(p, 0.64, 0.38, 0.12, white, 0.95);
            pw_fill_rect(p, 0.36, 0.36, 0.64, 0.5, white, 0.95);
            pw_line(p, 0.27, 0.5, 0.5, 0.74, 0.06, white, 0.95);
            pw_line(p, 0.73, 0.5, 0.5, 0.74, 0.06, white, 0.95);
        }
        PwIcon::Cross => {
            pw_fill_rect(p, 0.42, 0.2, 0.58, 0.8, white, 0.95);
            pw_fill_rect(p, 0.2, 0.42, 0.8, 0.58, white, 0.95);
        }
        PwIcon::Star => {
            for (dx, dy) in [(0.0, -1.0), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)] {
                pw_fill_rect(
                    p,
                    0.5 + dx * 0.04 - 0.06,
                    0.5 + dy * 0.04 - 0.06,
                    0.5 + dx * 0.04 + 0.06,
                    0.5 + dy * 0.04 + 0.06,
                    white,
                    0.95,
                );
            }
            pw_fill_circle(p, 0.5, 0.5, 0.1, white, 0.95);
        }
        PwIcon::Skull => {
            pw_fill_circle(p, 0.5, 0.52, 0.2, white, 0.95);
            pw_fill_rect(p, 0.4, 0.58, 0.6, 0.68, white, 0.95);
            pw_fill_circle(p, 0.42, 0.5, 0.05, dark, 0.95);
            pw_fill_circle(p, 0.58, 0.5, 0.05, dark, 0.95);
        }
        PwIcon::Clock => {
            pw_ring(p, 0.5, 0.5, 0.2, 0.045, white, 0.95);
            pw_line(p, 0.5, 0.5, 0.5, 0.38, 0.045, white, 0.95);
            pw_line(p, 0.5, 0.5, 0.6, 0.55, 0.045, white, 0.95);
        }
        PwIcon::Blocks => {
            pw_fill_rect(p, 0.3, 0.24, 0.7, 0.44, white, 0.95);
            pw_fill_rect(p, 0.3, 0.56, 0.7, 0.76, white, 0.95);
        }
        PwIcon::Bolt => {
            pw_line(p, 0.58, 0.2, 0.4, 0.52, 0.08, white, 0.95);
            pw_line(p, 0.42, 0.5, 0.6, 0.5, 0.08, white, 0.95);
            pw_line(p, 0.6, 0.5, 0.42, 0.8, 0.08, white, 0.95);
        }
        PwIcon::Bomb => {
            pw_fill_circle(p, 0.5, 0.55, 0.18, white, 0.95);
            pw_line(p, 0.5, 0.37, 0.5, 0.27, 0.04, white, 0.95);
            pw_fill_circle(p, 0.55, 0.24, 0.045, white, 0.9);
        }
        PwIcon::Burst => {
            pw_fill_circle(p, 0.5, 0.5, 0.09, white, 0.95);
            for (dx, dy) in [
                (0.0, -1.0),
                (0.0, 1.0),
                (-1.0, 0.0),
                (1.0, 0.0),
                (-0.7, -0.7),
                (0.7, -0.7),
                (-0.7, 0.7),
                (0.7, 0.7),
            ] {
                pw_fill_rect(
                    p,
                    0.5 + dx * 0.14 - 0.03,
                    0.5 + dy * 0.14 - 0.03,
                    0.5 + dx * 0.14 + 0.03,
                    0.5 + dy * 0.14 + 0.03,
                    white,
                    0.95,
                );
            }
        }
        PwIcon::Rings => {
            pw_ring(p, 0.5, 0.5, 0.16, 0.04, white, 0.9);
            pw_ring(p, 0.5, 0.5, 0.28, 0.03, white, 0.6);
        }
        PwIcon::Arrows => {
            pw_line(p, 0.5, 0.7, 0.5, 0.32, 0.06, white, 0.95);
            pw_line(p, 0.5, 0.32, 0.42, 0.42, 0.05, white, 0.95);
            pw_line(p, 0.5, 0.32, 0.58, 0.42, 0.05, white, 0.95);
            pw_line(p, 0.5, 0.7, 0.42, 0.6, 0.05, white, 0.95);
            pw_line(p, 0.5, 0.7, 0.58, 0.6, 0.05, white, 0.95);
        }
        PwIcon::Eclipse => {
            pw_fill_circle(p, 0.5, 0.5, 0.22, white, 0.95);
            pw_fill_circle(p, 0.62, 0.42, 0.2, dark, 0.95);
        }
    }
}

/// Bake a distinctive powerup texture: a radial accent base + white icon.
pub fn bake_powerup(kind: metanoid_core::components::powerup::PowerUpKind, size: u32) -> Pixels {
    let (icon, accent) = powerup_icon(kind);
    let mut p = Pixels::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let nx = (x as f32 + 0.5) / size as f32;
            let ny = (y as f32 + 0.5) / size as f32;
            let d = ((nx - 0.5) * 2.0).hypot((ny - 0.5) * 2.0);
            let core = (1.0 - d * 0.75).clamp(0.0, 1.0);
            let col = Srgba::new(
                (accent.red * 0.35 + (accent.red + 0.3) * core).min(1.0),
                (accent.green * 0.35 + (accent.green + 0.3) * core).min(1.0),
                (accent.blue * 0.35 + (accent.blue + 0.3) * core).min(1.0),
                1.0,
            );
            let edge_alpha = (1.0 - ((d - 0.7) / 0.4)).clamp(0.0, 1.0);
            p.set(
                x,
                y,
                LinearRgba::from(Srgba::new(col.red, col.green, col.blue, edge_alpha)),
            );
        }
    }
    pw_draw_icon(&mut p, icon);
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

    #[test]
    fn glow_and_ring_bake() {
        let g = bake_radial_glow(32);
        assert_eq!(g.data.len(), 32 * 32 * 4);
        // Center pixel is opaque, corner is transparent.
        assert!(g.data[(16 * 32 + 16) * 4 + 3] > 200);
        assert!(g.data[3] < 60);
        let r = bake_ring(32, 2.0);
        assert_eq!(r.data.len(), 32 * 32 * 4);
    }

    #[test]
    fn all_powerups_bake() {
        use metanoid_core::components::powerup::PowerUpKind;
        for kind in [
            PowerUpKind::Fireball,
            PowerUpKind::MegaBall,
            PowerUpKind::SplitBall,
            PowerUpKind::FastBall,
            PowerUpKind::SlowBall,
            PowerUpKind::LaserPaddle,
            PowerUpKind::GrabPaddle,
            PowerUpKind::ExpandPaddle,
            PowerUpKind::ShrinkPaddle,
            PowerUpKind::Shield,
            PowerUpKind::ExtraLife,
            PowerUpKind::DoublePoints,
            PowerUpKind::LevelWarp,
            PowerUpKind::KillPaddle,
            PowerUpKind::TimeSlow,
            PowerUpKind::FallingBricks,
            PowerUpKind::Zap,
            PowerUpKind::Explode,
            PowerUpKind::ExpandExploding,
            PowerUpKind::Lightning,
            PowerUpKind::Shockwave,
            PowerUpKind::ShuffleBricks,
            PowerUpKind::Blackout,
        ] {
            let p = bake_powerup(kind, 32);
            assert_eq!(p.data.len(), 32 * 32 * 4, "kind {kind:?}");
        }
    }
}
