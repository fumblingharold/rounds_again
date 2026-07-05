use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier2d::prelude::*;

/// Sets up the walls as Fixed RigidBodies.
pub fn setup_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((Name::new("Floor"), RigidBody::Fixed))
        .insert(Transform::from_xyz(0., 0., 0.))
        .insert(Visibility::default())
        .with_children(|parent| {
            let width = 1920.;
            let height = 1080.;
            let wall_width = 100.;
            let half_wall_width = wall_width / 2.;

            let horiz_wall_mesh = meshes.add(Rectangle::new(width, wall_width));
            let vert_wall_mesh = meshes.add(Rectangle::new(wall_width, height));
            let wall_material = materials.add(Color::from(tailwind::RED_600));

            parent
                .spawn(Collider::cuboid(width / 2.0, half_wall_width))
                .insert(Mesh2d(horiz_wall_mesh.clone()))
                .insert(MeshMaterial2d(wall_material.clone()))
                .insert(Transform::from_xyz(0., -height / 2. + half_wall_width, 0.));
            parent
                .spawn(Collider::cuboid(width / 2.0, half_wall_width))
                .insert(Mesh2d(horiz_wall_mesh.clone()))
                .insert(MeshMaterial2d(wall_material.clone()))
                .insert(Transform::from_xyz(0., height / 2. - half_wall_width, 0.));
            parent
                .spawn(Collider::cuboid(half_wall_width, height / 2.0))
                .insert(Mesh2d(vert_wall_mesh.clone()))
                .insert(MeshMaterial2d(wall_material.clone()))
                .insert(Transform::from_xyz(-width / 2. + half_wall_width, 0., 0.));
            parent
                .spawn(Collider::cuboid(half_wall_width, height / 2.0))
                .insert(Mesh2d(vert_wall_mesh.clone()))
                .insert(MeshMaterial2d(wall_material.clone()))
                .insert(Transform::from_xyz(width / 2. - half_wall_width, 0., 0.));
        });
}
