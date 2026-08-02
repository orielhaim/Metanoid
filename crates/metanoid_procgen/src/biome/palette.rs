use super::parameters::BiomeParams;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

impl Hsl {
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self {
            h: h.rem_euclid(360.0),
            s: s.clamp(0.0, 1.0),
            l: l.clamp(0.0, 1.0),
        }
    }

    pub fn to_rgb(self) -> (f32, f32, f32) {
        hsl_to_rgb(self.h, self.s, self.l)
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }
    let h = h / 60.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + m, g + m, b + m)
}

#[derive(Debug, Clone)]
pub struct ProceduralPalette {
    pub primary: Hsl,
    pub secondary: Hsl,
    pub accent: Hsl,
    pub glow: Hsl,
    pub background: Hsl,
    pub danger: Hsl,
}

impl ProceduralPalette {
    pub fn generate(params: &BiomeParams) -> Self {
        let dominant_hue = params.temperature * 360.0;

        let sat_base = 0.5 + params.energy * 0.4;
        let light_base = 0.4 + params.energy * 0.2;

        let primary = Hsl::new(dominant_hue, sat_base, light_base);

        let secondary = if params.chaos < 0.33 {
            Hsl::new(dominant_hue + 30.0, sat_base * 0.9, light_base + 0.1)
        } else if params.chaos < 0.66 {
            Hsl::new(dominant_hue + 180.0, sat_base, light_base)
        } else {
            Hsl::new(dominant_hue + 120.0, sat_base * 0.85, light_base + 0.05)
        };

        let accent = Hsl::new(
            dominant_hue + 180.0,
            (sat_base + 0.2).min(1.0),
            light_base + 0.15,
        );

        let glow = Hsl::new(
            dominant_hue + 15.0,
            (sat_base + 0.3).min(1.0),
            (light_base + 0.3).min(0.95),
        );

        let bg_sat = (sat_base * 0.3).max(0.1);
        let bg_light = (0.05 + (1.0 - params.energy) * 0.15).min(0.25);
        let background = Hsl::new(dominant_hue + 200.0, bg_sat, bg_light);

        let danger = Hsl::new(0.0, 0.9, 0.55);

        Self {
            primary,
            secondary,
            accent,
            glow,
            background,
            danger,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biome::generator::BiomeGenerator;
    use crate::seed::hierarchy::MasterSeed;

    #[test]
    fn palette_hues_valid() {
        let master = MasterSeed(42);
        for i in 0..50 {
            let params = BiomeGenerator::generate(master.galaxy(0).biome(i));
            let pal = ProceduralPalette::generate(&params);
            assert!(
                pal.primary.h >= 0.0 && pal.primary.h < 360.0,
                "primary hue {i}"
            );
            assert!(
                pal.secondary.h >= 0.0 && pal.secondary.h < 360.0,
                "secondary hue {i}"
            );
            assert!(
                pal.accent.h >= 0.0 && pal.accent.h < 360.0,
                "accent hue {i}"
            );
            assert!(pal.glow.h >= 0.0 && pal.glow.h < 360.0, "glow hue {i}");
            assert!(
                pal.background.h >= 0.0 && pal.background.h < 360.0,
                "bg hue {i}"
            );
        }
    }

    #[test]
    fn palette_deterministic() {
        let master = MasterSeed(77);
        let params = BiomeGenerator::generate(master.galaxy(0).biome(0));
        let a = ProceduralPalette::generate(&params);
        let b = ProceduralPalette::generate(&params);
        assert_eq!(format!("{a:?}"), format!("{b:?}"));
    }

    #[test]
    fn hsl_to_rgb_pure_red() {
        let (r, g, b) = hsl_to_rgb(0.0, 1.0, 0.5);
        assert!((r - 1.0).abs() < 1e-5);
        assert!(g.abs() < 1e-5);
        assert!(b.abs() < 1e-5);
    }

    #[test]
    fn hsl_to_rgb_white() {
        let (r, g, b) = hsl_to_rgb(0.0, 0.0, 1.0);
        assert!((r - 1.0).abs() < 1e-5);
        assert!((g - 1.0).abs() < 1e-5);
        assert!((b - 1.0).abs() < 1e-5);
    }
}
