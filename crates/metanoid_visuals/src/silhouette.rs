//! Procedural parallax silhouette layers: trees, crags, crystals, dunes, etc.
//! Generated from the recipe's silhouette spec, with a gentle time-based sway.

use bevy::prelude::*;
use metanoid_core::settings::GameSettings;

use crate::recipe::{BiomeRecipe, SilhouetteKind};

/// Parallax depth marker; the game crate shifts these with the paddle.
#[derive(Component)]
pub struct ParallaxLayer {
    pub depth: f32,
}

/// Marker: part of the generated background silhouette set.
#[derive(Component)]
pub struct SilhouettePart;

/// Per-structure sway animation data.
#[derive(Component)]
pub struct Sway {
    pub phase: f32,
    pub amp: f32,
    pub base_x: f32,
}

/// Spawn two parallax layers (far + near) of silhouettes.
pub fn spawn_silhouette_layers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    recipe: &BiomeRecipe,
) {
    let spec = &recipe.silhouettes;
    let far_mat = materials.add(ColorMaterial::from_color(spec.far));
    let near_mat = materials.add(ColorMaterial::from_color(spec.near));

    spawn_layer(
        commands,
        meshes,
        far_mat,
        spec.kind,
        spec.density,
        0.15,
        -8.5,
        spec.sway * 0.3,
        recipe.var_seed ^ 0xA7,
    );
    spawn_layer(
        commands,
        meshes,
        near_mat,
        spec.kind,
        spec.density * 0.7,
        0.4,
        -7.0,
        spec.sway,
        recipe.var_seed ^ 0xE5,
    );
}

fn spawn_layer(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<ColorMaterial>,
    kind: SilhouetteKind,
    density: f32,
    depth: f32,
    z: f32,
    sway: f32,
    seed: u64,
) {
    let mut h = seed;
    let mut rng = || {
        h ^= h << 13;
        h ^= h >> 7;
        h ^= h << 17;
        (h % 10000) as f32 / 10000.0
    };

    // Spread structures across the whole width, denser near the sides.
    let count = (14.0 * (0.5 + density * 0.6)).round() as u32;
    for i in 0..count {
        let t = i as f32 / count as f32;
        let x = -760.0 + t * 1520.0 + (rng() - 0.5) * 60.0;
        let scale = 0.6 + rng() * 0.9;
        let phase = rng() * 6.2832;
        // Sit flush with the bottom of the view (y = -ARENA_HEIGHT / 2).
        let base_y = -360.0;
        spawn_structure(
            commands,
            meshes,
            &material,
            kind,
            x,
            base_y,
            scale,
            depth,
            z,
            sway,
            phase,
            seed ^ (i as u64 * 0x9E37_79B9),
        );
    }
}

fn spawn_structure(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<ColorMaterial>,
    kind: SilhouetteKind,
    x: f32,
    base_y: f32,
    scale: f32,
    depth: f32,
    z: f32,
    sway: f32,
    phase: f32,
    seed: u64,
) {
    let mut h = seed;
    let mut rng = || {
        h ^= h << 13;
        h ^= h >> 7;
        h ^= h << 17;
        (h % 10000) as f32 / 10000.0
    };

    match kind {
        SilhouetteKind::Trees => {
            let trunk_w = 6.0 + rng() * 6.0;
            let trunk_h = 46.0 * scale + 20.0;
            let fol_h = 52.0 * scale + 24.0;
            let fol_w = 30.0 * scale + 12.0;
            spawn_rect(
                commands,
                meshes,
                material,
                trunk_w,
                trunk_h,
                x,
                base_y + trunk_h / 2.0,
                z,
                depth,
                sway,
                phase,
                x,
            );
            spawn_tri(
                commands,
                meshes,
                material,
                fol_w,
                fol_h,
                x,
                base_y + trunk_h + fol_h * 0.35,
                z,
                depth,
                sway,
                phase * 1.3,
                x,
            );
        }
        SilhouetteKind::Crags => {
            let w = 34.0 * scale + 14.0;
            let h = 60.0 * scale + 30.0;
            spawn_tri(
                commands,
                meshes,
                material,
                w,
                h,
                x,
                base_y + 12.0,
                z,
                depth,
                sway,
                phase,
                x,
            );
            spawn_tri(
                commands,
                meshes,
                material,
                w * 0.6,
                h * 0.7,
                x + w * 0.3,
                base_y + 4.0,
                z,
                depth,
                sway,
                phase * 1.5,
                x + w * 0.3,
            );
        }
        SilhouetteKind::Crystals => {
            let w = 18.0 * scale + 8.0;
            let h = 80.0 * scale + 40.0;
            spawn_tri(
                commands,
                meshes,
                material,
                w,
                h,
                x,
                base_y + 8.0,
                z,
                depth,
                sway,
                phase,
                x,
            );
            spawn_tri(
                commands,
                meshes,
                material,
                w * 0.55,
                h * 0.6,
                x - w * 0.4,
                base_y + 2.0,
                z,
                depth,
                sway,
                phase * 1.7,
                x - w * 0.4,
            );
        }
        SilhouetteKind::Dunes => {
            let w = 90.0 * scale + 40.0;
            let h = 26.0 * scale + 12.0;
            let mesh = meshes.add(Ellipse::new(w, h));
            let mut e = commands.spawn((
                SilhouettePart,
                ParallaxLayer { depth },
                Mesh2d(mesh),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(x, base_y - h * 0.4, z),
            ));
            e.insert(Sway {
                phase,
                amp: sway * 1.2,
                base_x: x,
            });
        }
        SilhouetteKind::Skyline => {
            let bw = 30.0 * scale + 14.0;
            let bh = 70.0 * scale + 30.0;
            spawn_rect(
                commands,
                meshes,
                material,
                bw,
                bh,
                x,
                base_y + bh / 2.0,
                z,
                depth,
                sway,
                phase,
                x,
            );
            spawn_rect(
                commands,
                meshes,
                material,
                bw * 0.7,
                bh * 0.6,
                x + bw * 0.8,
                base_y + bh * 0.3,
                z,
                depth,
                sway,
                phase * 1.2,
                x + bw * 0.8,
            );
        }
        SilhouetteKind::FrozenSpires => {
            let w = 22.0 * scale + 10.0;
            let h = 95.0 * scale + 50.0;
            spawn_tri(
                commands,
                meshes,
                material,
                w,
                h,
                x,
                base_y + 6.0,
                z,
                depth,
                sway,
                phase,
                x,
            );
            spawn_tri(
                commands,
                meshes,
                material,
                w * 0.5,
                h * 0.8,
                x - w * 0.5,
                base_y,
                z,
                depth,
                sway,
                phase * 1.4,
                x - w * 0.5,
            );
        }
        SilhouetteKind::Coral => {
            let r = 22.0 * scale + 10.0;
            let mesh = meshes.add(Circle::new(r));
            let mut e = commands.spawn((
                SilhouettePart,
                ParallaxLayer { depth },
                Mesh2d(mesh),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(x, base_y - r * 0.2, z),
            ));
            e.insert(Sway {
                phase,
                amp: sway * 2.0,
                base_x: x,
            });
            let r2 = r * 0.7;
            let mesh2 = meshes.add(Circle::new(r2));
            commands.spawn((
                SilhouettePart,
                ParallaxLayer { depth },
                Mesh2d(mesh2),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(x + r * 0.8, base_y + r * 0.4, z),
            ));
        }
        SilhouetteKind::Monoliths => {
            let bw = 40.0 * scale + 16.0;
            let bh = 100.0 * scale + 50.0;
            spawn_rect(
                commands,
                meshes,
                material,
                bw,
                bh,
                x,
                base_y + bh / 2.0,
                z,
                depth,
                sway,
                phase,
                x,
            );
            let t_w = bw * 0.5;
            let t_h = 26.0 * scale;
            spawn_tri(
                commands,
                meshes,
                material,
                t_w,
                t_h,
                x,
                base_y + bh + t_h * 0.4,
                z,
                depth,
                sway,
                phase * 1.1,
                x,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_rect(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<ColorMaterial>,
    w: f32,
    h: f32,
    x: f32,
    y: f32,
    z: f32,
    depth: f32,
    sway: f32,
    phase: f32,
    base_x: f32,
) {
    let mesh = meshes.add(Rectangle::new(w, h));
    let mut e = commands.spawn((
        SilhouettePart,
        ParallaxLayer { depth },
        Mesh2d(mesh),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(x, y, z),
    ));
    e.insert(Sway {
        phase,
        amp: sway,
        base_x,
    });
}

#[allow(clippy::too_many_arguments)]
fn spawn_tri(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<ColorMaterial>,
    w: f32,
    h: f32,
    x: f32,
    y: f32,
    z: f32,
    depth: f32,
    sway: f32,
    phase: f32,
    base_x: f32,
) {
    let mesh = meshes.add(Triangle2d::new(
        Vec2::new(-w / 2.0, -h / 2.0),
        Vec2::new(w / 2.0, -h / 2.0),
        Vec2::new(0.0, h / 2.0),
    ));
    let mut e = commands.spawn((
        SilhouettePart,
        ParallaxLayer { depth },
        Mesh2d(mesh),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(x, y, z),
    ));
    e.insert(Sway {
        phase,
        amp: sway,
        base_x,
    });
}

/// Gentle breeze + paddle parallax: offset each silhouette around its base.
pub fn tick_silhouette_sway(
    time: Res<Time>,
    settings: Res<GameSettings>,
    paddle: Query<&Transform, With<metanoid_core::components::paddle::Paddle>>,
    mut q: Query<
        (&Sway, &ParallaxLayer, &mut Transform),
        Without<metanoid_core::components::paddle::Paddle>,
    >,
) {
    let motion = if settings.reduce_motion { 0.0 } else { 1.0 };
    let t = time.elapsed_secs();
    let px = paddle
        .single()
        .map(|t| t.translation.x / (metanoid_core::constants::ARENA_WIDTH / 2.0))
        .unwrap_or(0.0);
    for (sway, layer, mut transform) in &mut q {
        let sway_offset = (t * 0.6 + sway.phase).sin() * sway.amp * motion;
        let parallax = px * layer.depth * 44.0 * motion;
        transform.translation.x = sway.base_x + sway_offset + parallax;
    }
}
