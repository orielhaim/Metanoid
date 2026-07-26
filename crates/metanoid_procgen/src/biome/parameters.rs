use rand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiomeParams {
    pub temperature: f32,
    pub density: f32,
    pub chaos: f32,
    pub energy: f32,
    pub weirdness: f32,
}

impl BiomeParams {
    pub fn sample(rng: &mut impl Rng) -> Self {
        Self {
            temperature: rng.random::<f32>(),
            density: rng.random::<f32>(),
            chaos: rng.random::<f32>(),
            energy: rng.random::<f32>(),
            weirdness: rng.random::<f32>(),
        }
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            temperature: self.temperature + (other.temperature - self.temperature) * t,
            density: self.density + (other.density - self.density) * t,
            chaos: self.chaos + (other.chaos - self.chaos) * t,
            energy: self.energy + (other.energy - self.energy) * t,
            weirdness: self.weirdness + (other.weirdness - self.weirdness) * t,
        }
    }

    pub fn clamp01(self) -> Self {
        Self {
            temperature: self.temperature.clamp(0.0, 1.0),
            density: self.density.clamp(0.0, 1.0),
            chaos: self.chaos.clamp(0.0, 1.0),
            energy: self.energy.clamp(0.0, 1.0),
            weirdness: self.weirdness.clamp(0.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn sample_in_range() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..1000 {
            let p = BiomeParams::sample(&mut rng);
            assert!((0.0..=1.0).contains(&p.temperature));
            assert!((0.0..=1.0).contains(&p.density));
            assert!((0.0..=1.0).contains(&p.chaos));
            assert!((0.0..=1.0).contains(&p.energy));
            assert!((0.0..=1.0).contains(&p.weirdness));
        }
    }

    #[test]
    fn lerp_endpoints() {
        let a = BiomeParams {
            temperature: 0.0,
            density: 0.0,
            chaos: 0.0,
            energy: 0.0,
            weirdness: 0.0,
        };
        let b = BiomeParams {
            temperature: 1.0,
            density: 1.0,
            chaos: 1.0,
            energy: 1.0,
            weirdness: 1.0,
        };
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
        let mid = a.lerp(b, 0.5);
        assert!((mid.temperature - 0.5).abs() < 1e-6);
    }
}
