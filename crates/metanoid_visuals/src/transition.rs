//! The loading "curtain" — biome-specific panels that cover the screen and
//! slide apart with staggered easing to reveal the stage (a forest of trunks
//! parting, crystal shards shattering, volcanic columns sinking, neon blinds
//! rolling up, ...).

use bevy::color::Mix;
use bevy::prelude::*;
use metanoid_core::constants::*;
use metanoid_core::settings::GameSettings;

use crate::recipe::{BiomeRecipe, CurtainKind};

/// Root of a curtain reveal.
#[derive(Component)]
pub struct Curtain {
    pub elapsed: f32,
    pub total: f32,
    pub started: bool,
}

impl Curtain {
    pub fn finished(&self) -> bool {
        self.started && self.elapsed >= self.total
    }

    /// Reveal progress 0..1 (only meaningful once started).
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
}

/// One panel of the curtain.
#[derive(Component)]
pub struct CurtainPanel {
    pub base_x: f32,
    pub dir: f32,
    pub delay: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct CurtainRoot;

fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

/// Spawn the curtain for a recipe, fully closed. The curtain reads as a living
/// biome wall: two-tone panels with a luminous seam, plus recognizable crowns /
/// tips per biome kind.
pub fn spawn_curtain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    recipe: &BiomeRecipe,
) -> Entity {
    let spec = recipe.curtain;
    let total = 1.55;

    let panel_count = match spec.kind {
        CurtainKind::Trees => 12,
        CurtainKind::Blinds => 18,
        _ => 10,
    };
    let panel_w = ARENA_WIDTH / panel_count as f32 + 4.0;
    let panel_h = ARENA_HEIGHT + 80.0;

    let root = commands
        .spawn((
            CurtainRoot,
            Curtain {
                elapsed: 0.0,
                total,
                started: false,
            },
        ))
        .id();

    // Brighten the base so panels read as scenery, not a void.
    let panel_color = spec.primary.mix(&LinearRgba::WHITE, 0.10);
    let seam_color = spec.secondary.mix(&LinearRgba::WHITE, 0.55);
    let crown_color = spec
        .secondary
        .mix(&spec.primary, 0.35)
        .mix(&LinearRgba::WHITE, 0.22);

    let material = materials.add(ColorMaterial::from_color(panel_color));
    let seam_material = materials.add(ColorMaterial::from_color(seam_color));
    let crown_material = materials.add(ColorMaterial::from_color(crown_color));

    let half = panel_count / 2;
    for i in 0..panel_count {
        let center = (i as f32 + 0.5) * panel_w - ARENA_WIDTH / 2.0 - 2.0;
        let dir = if (i as f32) < half as f32 { -1.0 } else { 1.0 };
        // Center panels open first, edges later.
        let dist_from_center = ((i as f32 + 0.5) - half as f32).abs();
        let delay = 0.12 + dist_from_center * 0.06;
        let duration = 0.95;

        let panel = commands
            .spawn((
                CurtainPanel {
                    base_x: center,
                    dir,
                    delay,
                    duration,
                },
                Mesh2d(meshes.add(Rectangle::new(panel_w, panel_h))),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(center, 0.0, 4.0),
            ))
            .id();
        commands.entity(root).add_child(panel);

        // Luminous leading-edge seam on the edge facing the screen center
        // (the "crack of light" that appears first as panels part).
        let seam_x = center - dir * (panel_w / 2.0 - 2.0);
        let seam = commands
            .spawn((
                Mesh2d(meshes.add(Rectangle::new(4.0, panel_h))),
                MeshMaterial2d(seam_material.clone()),
                Transform::from_xyz(seam_x, 0.0, 4.1),
            ))
            .id();
        commands.entity(root).add_child(seam);

        // Decorative crowns / tips.
        match spec.kind {
            CurtainKind::Trees => {
                let crown_w = panel_w * 1.5;
                let crown_h = 170.0;
                let crown = commands
                    .spawn((
                        Mesh2d(meshes.add(Triangle2d::new(
                            Vec2::new(-crown_w / 2.0, -crown_h / 2.0),
                            Vec2::new(crown_w / 2.0, -crown_h / 2.0),
                            Vec2::new(0.0, crown_h / 2.0),
                        ))),
                        MeshMaterial2d(crown_material.clone()),
                        Transform::from_xyz(center, ARENA_HEIGHT / 2.0 + crown_h * 0.22, 4.15),
                    ))
                    .id();
                commands.entity(root).add_child(crown);
            }
            CurtainKind::Shards | CurtainKind::IceWalls | CurtainKind::Columns => {
                let tip_h = 90.0;
                let tip_w = panel_w * 1.3;
                let tip = commands
                    .spawn((
                        Mesh2d(meshes.add(Triangle2d::new(
                            Vec2::new(-tip_w / 2.0, -tip_h / 2.0),
                            Vec2::new(tip_w / 2.0, -tip_h / 2.0),
                            Vec2::new(0.0, tip_h / 2.0),
                        ))),
                        MeshMaterial2d(crown_material.clone()),
                        Transform::from_xyz(center, ARENA_HEIGHT / 2.0 + tip_h * 0.2, 4.15),
                    ))
                    .id();
                commands.entity(root).add_child(tip);
            }
            CurtainKind::Void => {
                // sprinkle a few bright stars
                let mut h =
                    (recipe.var_seed ^ (i as u64 * 0x9E37_79B9)).wrapping_mul(0x100000001b3);
                for _ in 0..8 {
                    h ^= h << 13;
                    h ^= h >> 7;
                    h ^= h << 17;
                    let sx = center + ((h % 2000) as f32 / 1000.0 - 1.0) * panel_w * 0.4;
                    h ^= h << 13;
                    h ^= h >> 7;
                    h ^= h << 17;
                    let sy = ((h % 1400) as f32 / 1000.0 - 0.3) * ARENA_HEIGHT * 0.8;
                    h ^= h << 13;
                    h ^= h >> 7;
                    h ^= h << 17;
                    let r = 1.0 + (h % 24) as f32 / 10.0;
                    let star = commands
                        .spawn((
                            Mesh2d(meshes.add(Circle::new(r))),
                            MeshMaterial2d(seam_material.clone()),
                            Transform::from_xyz(sx, sy, 4.2),
                        ))
                        .id();
                    commands.entity(root).add_child(star);
                }
            }
            CurtainKind::Blinds => {}
        }
    }

    root
}

/// Advance the curtain and move panels to their eased offsets with a slight
/// outward tilt so the reveal reads as doors / trunks parting.
pub fn tick_curtain(
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut curtains: Query<&mut Curtain, Without<CurtainPanel>>,
    mut panels: Query<(&CurtainPanel, &mut Transform), Without<Curtain>>,
) {
    let mut curtain = match curtains.single_mut() {
        Ok(c) => c,
        Err(_) => return,
    };
    if !curtain.started {
        return;
    }
    curtain.elapsed += time.delta_secs();
    if settings.reduce_motion {
        curtain.elapsed = curtain.total;
    }
    let travel = ARENA_WIDTH / 2.0 + 220.0;
    for (panel, mut transform) in &mut panels {
        let t = ((curtain.elapsed - panel.delay) / panel.duration).clamp(0.0, 1.0);
        let eased = ease_out_cubic(t);
        transform.translation.x = panel.base_x + panel.dir * eased * travel;
        // Slight outward tilt while parting.
        transform.rotation = Quat::from_rotation_z(panel.dir * eased * 0.05);
    }
}
