use bevy::prelude::*;
use metanoid_core::components::paddle::Paddle;
use metanoid_core::constants::*;

use crate::systems::level_spawner::LevelEntity;

#[derive(Component)]
pub struct ParallaxLayer {
    pub depth: f32,
}

pub fn setup_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bg_color = Color::srgb(0.02, 0.02, 0.05);
    let bg_mesh = meshes.add(Rectangle::new(ARENA_WIDTH * 2.0, ARENA_HEIGHT * 2.0));
    let bg_mat = materials.add(bg_color);
    commands.spawn((
        LevelEntity,
        ParallaxLayer { depth: 0.0 },
        Mesh2d(bg_mesh),
        MeshMaterial2d(bg_mat),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    let shapes_mesh = meshes.add(Rectangle::new(60.0, 60.0));
    for i in 0..12 {
        let x = (i as f32 - 5.5) * 100.0 + 30.0;
        let y = ((i * 37) % 400) as f32 - 200.0;
        let opacity = 0.06 + (i as f32 % 3.0) * 0.02;
        let mat = materials.add(Color::srgba(0.3, 0.3, 0.5, opacity));
        commands.spawn((
            LevelEntity,
            ParallaxLayer { depth: 0.1 },
            Mesh2d(shapes_mesh.clone()),
            MeshMaterial2d(mat),
            Transform::from_xyz(x, y, -8.0)
                .with_rotation(Quat::from_rotation_z(i as f32 * 0.5)),
        ));
    }

    let particle_mesh = meshes.add(Circle::new(2.0));
    for i in 0..30 {
        let x = (i as f32 - 15.0) * 80.0 + (i as f32 * 17.0).sin() * 40.0;
        let y = (i as f32 * 23.0).cos() * 250.0;
        let opacity = 0.04 + (i as f32 % 4.0) * 0.015;
        let mat = materials.add(Color::srgba(0.5, 0.5, 0.7, opacity));
        commands.spawn((
            LevelEntity,
            ParallaxLayer { depth: 0.3 },
            Mesh2d(particle_mesh.clone()),
            MeshMaterial2d(mat),
            Transform::from_xyz(x, y, -6.0),
        ));
    }

    let star_mesh = meshes.add(Circle::new(1.0));
    for i in 0..50 {
        let x = (i as f32 - 25.0) * 50.0 + (i as f32 * 31.0).sin() * 60.0;
        let y = (i as f32 * 41.0).cos() * 300.0;
        let opacity = 0.03 + (i as f32 % 5.0) * 0.01;
        let mat = materials.add(Color::srgba(0.7, 0.7, 0.9, opacity));
        commands.spawn((
            LevelEntity,
            ParallaxLayer { depth: 0.5 },
            Mesh2d(star_mesh.clone()),
            MeshMaterial2d(mat),
            Transform::from_xyz(x, y, -4.0),
        ));
    }
}

pub fn parallax_shift(
    paddle: Query<&Transform, With<Paddle>>,
    mut layers: Query<(&ParallaxLayer, &mut Transform), Without<Paddle>>,
) {
    let Ok(paddle_transform) = paddle.single() else { return };
    let px = paddle_transform.translation.x / (ARENA_WIDTH / 2.0);

    for (layer, mut transform) in &mut layers {
        let shift = px * layer.depth * 30.0;
        transform.translation.x = transform.translation.x + shift * 0.02;
    }
}
