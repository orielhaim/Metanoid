//! Biome *composition*: what a biome is actually made of.
//!
//! Instead of classifying every biome into a single bucket, we describe it as a
//! weighted blend of attractor "flavors" (Forest, Volcanic, Ocean, ...). A plain
//! Forest biome is `[(Forest, 1.0)]`, a hybrid is `[(Forest, 0.6), (Volcanic,
//! 0.4)]` — the raw material the visuals engine needs to build "a forest on
//! fire". Occasionally the generator produces a rare 3-way "super combo".

use rand::prelude::*;

use super::generator::{ATTRACTORS, BiomeAttractor};
use super::parameters::BiomeParams;
use crate::seed::hierarchy::BiomeSeed;

/// A single ingredient of a biome, with its relative weight.
#[derive(Debug, Clone, Copy)]
pub struct BiomePart {
    pub name: &'static str,
    pub weight: f32,
}

/// The resolved composition of a biome: its blended parameters plus the flavors
/// (and how strongly) that produced them.
#[derive(Debug, Clone)]
pub struct BiomeComposition {
    pub params: BiomeParams,
    pub parts: Vec<BiomePart>,
}

impl BiomeComposition {
    pub fn primary_name(&self) -> &'static str {
        self.parts.first().map(|p| p.name).unwrap_or("Cosmic Void")
    }

    pub fn weight_of(&self, name: &str) -> f32 {
        self.parts
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.weight)
            .unwrap_or(0.0)
    }

    /// Deterministic visual variation seed derived from the biome seed.
    pub fn variation_seed(&self, seed: BiomeSeed) -> u64 {
        // Mix the raw seed with the part hash so identical params on different
        // seeds still look different, while staying fully deterministic.
        let mut h = 0xcbf29ce484222325u64;
        for part in &self.parts {
            for byte in part.name.bytes() {
                h ^= byte as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
        h ^= seed.0.rotate_left(13);
        h.wrapping_mul(0x100000001b3)
    }
}

/// Generate a biome composition from a biome seed.
///
/// Distribution (per design: systematic hybrids + occasional super-combos):
/// - ~55% pure / near-pure biomes
/// - ~35% two-way hybrids
/// - ~10% three-way super-combos (deeper, weirder biomes)
pub fn generate_composition(seed: BiomeSeed) -> BiomeComposition {
    let mut rng = seed.rng();
    let raw = BiomeParams::sample(&mut rng);

    let roll: f32 = rng.random();
    let (count, is_super) = if roll < 0.55 {
        (1usize, false)
    } else if roll < 0.90 {
        (2, false)
    } else {
        (3, true)
    };

    // Choose `count` distinct attractors.
    let mut indices: Vec<usize> = (0..ATTRACTORS.len()).collect();
    indices.shuffle(&mut rng);
    let chosen: Vec<&BiomeAttractor> = indices
        .iter()
        .take(count)
        .map(|&i| &ATTRACTORS[i])
        .collect();

    let mut weights: Vec<f32> = chosen
        .iter()
        .enumerate()
        .map(|(i, _)| {
            if is_super {
                // Super-combos are dominated by one wild flavor.
                if i == 0 {
                    rng.random_range(0.5..0.8)
                } else {
                    rng.random_range(0.1..0.25)
                }
            } else if count == 1 {
                rng.random_range(0.75..1.0)
            } else {
                rng.random_range(0.25..0.65)
            }
        })
        .collect();

    let total: f32 = weights.iter().sum();
    for w in weights.iter_mut() {
        *w /= total;
    }

    // Blend the attractor center params, then lerp the sample toward them.
    let mut center = BiomeParams {
        temperature: 0.0,
        density: 0.0,
        chaos: 0.0,
        energy: 0.0,
        weirdness: 0.0,
    };
    for (attractor, weight) in chosen.iter().zip(weights.iter()) {
        let c = attractor.center;
        center.temperature += c.temperature * weight;
        center.density += c.density * weight;
        center.chaos += c.chaos * weight;
        center.energy += c.energy * weight;
        center.weirdness += c.weirdness * weight;
    }

    // Stronger attractor pull for hybrids so they read clearly.
    let pull = if count == 1 { 0.6 } else { 0.7 };
    let params = raw.lerp(center, pull).clamp01();

    let parts = chosen
        .iter()
        .zip(weights.iter())
        .map(|(a, w)| BiomePart {
            name: a.name,
            weight: *w,
        })
        .collect();

    BiomeComposition { params, parts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::hierarchy::MasterSeed;

    #[test]
    fn weights_normalize() {
        let master = MasterSeed(42);
        for i in 0..200 {
            let c = generate_composition(master.galaxy(0).biome(i));
            let total: f32 = c.parts.iter().map(|p| p.weight).sum();
            assert!((total - 1.0).abs() < 1e-4, "weights {i}: {total}");
        }
    }

    #[test]
    fn params_in_range() {
        let master = MasterSeed(7);
        for i in 0..200 {
            let c = generate_composition(master.galaxy(0).biome(i));
            let p = &c.params;
            assert!((0.0..=1.0).contains(&p.temperature));
            assert!((0.0..=1.0).contains(&p.density));
            assert!((0.0..=1.0).contains(&p.chaos));
            assert!((0.0..=1.0).contains(&p.energy));
            assert!((0.0..=1.0).contains(&p.weirdness));
        }
    }

    #[test]
    fn deterministic() {
        let master = MasterSeed(99);
        let seed = master.galaxy(2).biome(1);
        let a = generate_composition(seed);
        let b = generate_composition(seed);
        assert_eq!(a.parts.len(), b.parts.len());
        for (x, y) in a.parts.iter().zip(b.parts.iter()) {
            assert_eq!(x.name, y.name);
            assert!((x.weight - y.weight).abs() < 1e-6);
        }
    }

    #[test]
    fn produces_hybrids_and_supers() {
        let master = MasterSeed(1234);
        let mut singles = 0;
        let mut hybrids = 0;
        let mut supers = 0;
        for i in 0..400 {
            let c = generate_composition(master.galaxy(0).biome(i));
            match c.parts.len() {
                1 => singles += 1,
                2 => hybrids += 1,
                3 => supers += 1,
                _ => panic!("unexpected part count"),
            }
        }
        assert!(hybrids > 0, "expected some hybrids");
        assert!(supers > 0, "expected some super-combos");
    }

    #[test]
    fn variation_seed_stable() {
        let master = MasterSeed(5);
        let seed = master.galaxy(0).biome(3);
        let c = generate_composition(seed);
        assert_eq!(c.variation_seed(seed), c.variation_seed(seed));
    }
}
