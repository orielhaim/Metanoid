use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::grid::BrickGrid;
use crate::level::layout::selector::generate_layout;

pub fn generate_base_structure(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    generate_layout(cols, rows, params, rng)
}
