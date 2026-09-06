use super::*;
use crate::game_state::wall::Wall;
use crate::shared::Hp;
use crate::{AppState, collision_groups};
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::{SliceRandom, ThreadRng};
use serde::Deserialize;
use std::fs;

/// A file with maps.
#[derive(Deserialize)]
struct MapFile {
    maps: Vec<Map>,
}

/// A map.
#[derive(Deserialize)]
struct Map {
    name: String,
    rect_walls: Vec<RectWall>,
    circle_phys_objects: Vec<CirclePhysObjects>,
    player_spawn_points: Vec<PlayerSpawnPoint>,
}

/// A point on the map where a player can be spawned.
#[derive(Deserialize)]
struct PlayerSpawnPoint {
    x: f32,
    y: f32,
}

/// A rectangular wall.
#[derive(Deserialize)]
struct RectWall {
    width: f32,
    height: f32,
    x: f32,
    y: f32,
}

impl RectWall {
    fn setup(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        rect_walls: &[Self],
    ) {
        let wall_material = materials.add(Color::from(tailwind::RED_600));
        for rect_wall in rect_walls.iter() {
            let wall_mesh = meshes.add(Rectangle::new(rect_wall.width, rect_wall.height));
            commands
                .spawn(Wall)
                .insert(Collider::cuboid(
                    rect_wall.width / 2.0,
                    rect_wall.height / 2.0,
                ))
                .insert(Mesh2d(wall_mesh.clone()))
                .insert(MeshMaterial2d(wall_material.clone()))
                .insert(CollisionGroups::new(
                    collision_groups::WALLS,
                    Group::default(),
                ))
                .insert(Transform::from_xyz(rect_wall.x, rect_wall.y, 0.));
        }
    }
}

/// A circular physics object.
#[derive(Deserialize)]
struct CirclePhysObjects {
    radius: f32,
    x: f32,
    y: f32,
    hp: Option<f32>,
}

impl CirclePhysObjects {
    fn setup(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        circle_phys_objects: &[Self],
    ) {
        let invincible_circle_material = materials.add(Color::from(tailwind::AMBER_500));
        let destructible_circle_material = materials.add(Color::from(tailwind::AMBER_800));
        for circle_phys_object in circle_phys_objects.iter() {
            let ball_mesh = meshes.add(Circle::new(circle_phys_object.radius));
            let mut bang = commands.spawn(PhysObject);
            bang.insert(Name::new("Ball"))
                .insert(Mesh2d(ball_mesh.clone()))
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(circle_phys_object.radius))
                .insert(Restitution::coefficient(0.7))
                .insert(CollisionGroups::new(
                    collision_groups::PHYS_OBJECTS,
                    Group::default(),
                ))
                .insert(Transform::from_xyz(
                    circle_phys_object.x,
                    circle_phys_object.y,
                    0.,
                ))
                .insert(ExternalImpulse::default());
            let (material, hp) = match circle_phys_object.hp {
                Some(hp) if hp <= 0. => panic!("physics objects must have health greater than 0"),
                Some(hp) if !hp.is_infinite() => (destructible_circle_material.clone(), hp),
                _ => (invincible_circle_material.clone(), f32::INFINITY),
            };
            bang.insert(MeshMaterial2d(material)).insert(Hp::new(hp));
        }
    }
}

/// Loads a random map.
pub fn setup_match(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    players: Query<&mut Transform, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let map_file: MapFile = toml::from_str(
        &fs::read_to_string("./default_maps.toml").expect("Could not access default_maps.toml"),
    )
    .expect("Could not parse default_maps.toml");
    let mut maps = map_file.maps;
    maps.shuffle(&mut ThreadRng::default());
    let map = &mut maps[0];

    RectWall::setup(&mut commands, &mut meshes, &mut materials, &map.rect_walls);
    CirclePhysObjects::setup(
        &mut commands,
        &mut meshes,
        &mut materials,
        &map.circle_phys_objects,
    );

    move_players(players, &mut map.player_spawn_points);

    next_state.set(AppState::Game);
}

fn move_players(
    mut players: Query<&mut Transform, With<Player>>,
    player_spawn_points: &mut [PlayerSpawnPoint],
) {
    assert!(
        players.iter().len() <= player_spawn_points.len(),
        "too many players for the map"
    );
    player_spawn_points.shuffle(&mut ThreadRng::default());
    for (mut player_transform, player_spawn_point) in
        players.iter_mut().zip(player_spawn_points.iter())
    {
        player_transform.translation.x = player_spawn_point.x;
        player_transform.translation.y = player_spawn_point.y;
    }
}

/// Clears all entities from the match
pub fn cleanup_match(
    mut commands: Commands,
    walls: Query<Entity, With<Wall>>,
    phys_objects: Query<Entity, With<PhysObject>>,
    bullets: Query<Entity, With<Bullet>>,
) {
    for entity in walls
        .iter()
        .chain(phys_objects.iter())
        .chain(bullets.iter())
    {
        commands.entity(entity).despawn();
    }
}
