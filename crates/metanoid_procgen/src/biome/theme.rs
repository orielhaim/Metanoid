use super::palette::ProceduralPalette;
use super::parameters::BiomeParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleStyle {
    Spark,
    Flame,
    Bubble,
    Snow,
    Dust,
    Glitch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundStyle {
    Gradient,
    NoiseField,
    GeometricShapes,
    Starfield,
    Fractal,
}

#[derive(Debug, Clone)]
pub struct BiomeTheme {
    pub palette: ProceduralPalette,
    pub particle_style: ParticleStyle,
    pub background_style: BackgroundStyle,
    pub bloom_intensity: f32,
    pub chromatic_aberration: f32,
    pub vignette_strength: f32,
}

impl BiomeTheme {
    pub fn generate(params: &BiomeParams) -> Self {
        let palette = ProceduralPalette::generate(params);

        let particle_style = if params.temperature > 0.7 {
            ParticleStyle::Flame
        } else if params.temperature < 0.2 {
            ParticleStyle::Snow
        } else if params.weirdness > 0.6 {
            ParticleStyle::Glitch
        } else if params.energy > 0.7 {
            ParticleStyle::Spark
        } else if params.density > 0.6 {
            ParticleStyle::Bubble
        } else {
            ParticleStyle::Dust
        };

        let background_style = if params.weirdness > 0.7 {
            BackgroundStyle::Fractal
        } else if params.temperature < 0.3 {
            BackgroundStyle::Starfield
        } else if params.chaos > 0.5 {
            BackgroundStyle::NoiseField
        } else {
            BackgroundStyle::GeometricShapes
        };

        let bloom_intensity = 0.2 + params.energy * 0.8;
        let chromatic_aberration = params.weirdness * 0.02 + params.chaos * 0.01;
        let vignette_strength = 0.2 + (1.0 - params.energy) * 0.3;

        Self {
            palette,
            particle_style,
            background_style,
            bloom_intensity,
            chromatic_aberration,
            vignette_strength,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biome::generator::BiomeGenerator;
    use crate::seed::hierarchy::MasterSeed;

    #[test]
    fn theme_deterministic() {
        let master = MasterSeed(42);
        let params = BiomeGenerator::generate(master.galaxy(0).biome(0));
        let a = BiomeTheme::generate(&params);
        let b = BiomeTheme::generate(&params);
        assert_eq!(format!("{a:?}"), format!("{b:?}"));
    }

    #[test]
    fn bloom_intensity_range() {
        let master = MasterSeed(100);
        for i in 0..50 {
            let params = BiomeGenerator::generate(master.galaxy(0).biome(i));
            let theme = BiomeTheme::generate(&params);
            assert!(
                theme.bloom_intensity >= 0.2 && theme.bloom_intensity <= 1.0,
                "bloom {i}"
            );
        }
    }

    #[test]
    fn hot_biome_flame() {
        let hot = BiomeParams {
            temperature: 0.9,
            density: 0.5,
            chaos: 0.3,
            energy: 0.5,
            weirdness: 0.1,
        };
        let theme = BiomeTheme::generate(&hot);
        assert_eq!(theme.particle_style, ParticleStyle::Flame);
    }

    #[test]
    fn cold_biome_snow() {
        let cold = BiomeParams {
            temperature: 0.1,
            density: 0.5,
            chaos: 0.3,
            energy: 0.5,
            weirdness: 0.1,
        };
        let theme = BiomeTheme::generate(&cold);
        assert_eq!(theme.particle_style, ParticleStyle::Snow);
    }

    #[test]
    fn weird_biome_glitch() {
        let weird = BiomeParams {
            temperature: 0.5,
            density: 0.5,
            chaos: 0.3,
            energy: 0.5,
            weirdness: 0.8,
        };
        let theme = BiomeTheme::generate(&weird);
        assert_eq!(theme.particle_style, ParticleStyle::Glitch);
    }
}
