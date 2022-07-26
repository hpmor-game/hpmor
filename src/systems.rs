#![allow(clippy::type_complexity)]

use crate::{components::*, plugins::SpriteSort};
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ecs_ldtk::prelude::*;

use std::collections::{HashMap, HashSet};

use heron::prelude::*;

#[derive(Component)]
pub struct MainCamera;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = OrthographicCameraBundle::new_2d();
    commands
        .spawn_bundle(camera)
        .insert(Name::new("MainCamera"))
        .insert(MainCamera);

    asset_server.watch_for_changes().unwrap();

    let ldtk_handle = asset_server.load("maps/verres-home/map.ldtk");
    commands
        .spawn_bundle(LdtkWorldBundle {
            ldtk_handle,
            ..Default::default()
        })
        .insert(Name::new("Map"))
        .insert(SpriteSort::default());
}

pub fn pause_physics_during_load(
    mut level_events: EventReader<LevelEvent>,
    mut physics_time: ResMut<PhysicsTime>,
) {
    for event in level_events.iter() {
        match event {
            LevelEvent::SpawnTriggered(_) => physics_time.set_scale(0.),
            LevelEvent::Transformed(_) => physics_time.set_scale(1.),
            _ => (),
        }
    }
}

fn calc_movement_input(keyboard: &Res<Input<KeyCode>>) -> Vec2 {
    let mut input = Vec2::ZERO;

    let left = keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left);
    let right = keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right);

    let up = keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up);
    let down = keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down);

    input.x = (right as i32 - left as i32) as f32;
    input.y = (up as i32 - down as i32) as f32;

    input
}

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    // consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, &Parent(parent))| {
        // the intgrid tiles' direct parents will be bevy_ecs_tilemap chunks, not the level
        // To get the level, you need their grandparents, which is where parent_query comes in
        if let Ok(&Parent(level_entity)) = parent_query.get(parent) {
            level_to_wall_locations
                .entry(level_entity)
                .or_insert_with(HashSet::new)
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right
                    // edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut wall_rects: Vec<Rect<i32>> = Vec::new();
                let mut previous_rects: HashMap<Plate, Rect<i32>> = HashMap::new();

                // an extra empty row so the algorithm "terminates" the rects that touch the top
                // edge
                plate_stack.push(Vec::new());

                for (y, row) in plate_stack.iter().enumerate() {
                    let mut current_rects: HashMap<Plate, Rect<i32>> = HashMap::new();
                    for plate in row {
                        if let Some(previous_rect) = previous_rects.remove(plate) {
                            current_rects.insert(
                                *plate,
                                Rect {
                                    top: previous_rect.top + 1,
                                    ..previous_rect
                                },
                            );
                        } else {
                            current_rects.insert(
                                *plate,
                                Rect {
                                    bottom: y as i32,
                                    top: y as i32,
                                    left: plate.left,
                                    right: plate.right,
                                },
                            );
                        }
                    }

                    // Any plates that weren't removed above have terminated
                    wall_rects.append(&mut previous_rects.values().copied().collect());
                    previous_rects = current_rects;
                }

                // spawn colliders for every rectangle
                for wall_rect in wall_rects {
                    commands
                        .spawn()
                        .insert(CollisionShape::Cuboid {
                            half_extends: Vec3::new(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                0.,
                            ),
                            border_radius: None,
                        })
                        .insert(RigidBody::Static)
                        .insert(PhysicMaterial {
                            friction: 0.1,
                            ..Default::default()
                        })
                        .insert(Transform::from_xyz(
                            (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32 / 2.,
                            (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32 / 2.,
                            0.,
                        ))
                        .insert(GlobalTransform::default())
                        // Making the collider a child of the level serves two purposes:
                        // 1. Adjusts the transforms to be relative to the level for free
                        // 2. the colliders will be despawned automatically when levels unload
                        .insert(Parent(level_entity));
                }
            }
        });
    }
}

const ASPECT_RATIO: f32 = 16. / 9.;

pub fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (&mut OrthographicProjection, &mut Transform),
        (Without<Player>, With<MainCamera>),
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<
        (&Transform, &Handle<LdtkLevel>),
        (Without<OrthographicProjection>, Without<Player>),
    >,
    level_selection: Res<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    if let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.get_single()
    {
        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();
        for (level_transform, level_handle) in level_query.iter() {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                let level = &ldtk_level.level;
                if level_selection.is_match(&0, level) {
                    let level_ratio = level.px_wid as f32 / ldtk_level.level.px_hei as f32;

                    orthographic_projection.scaling_mode = ScalingMode::None;
                    orthographic_projection.bottom = 0.;
                    orthographic_projection.left = 0.;
                    if level_ratio > ASPECT_RATIO {
                        // level is wider than the screen
                        orthographic_projection.top = (level.px_hei as f32 / 9.).round() * 9.;
                        orthographic_projection.right = orthographic_projection.top * ASPECT_RATIO;
                        camera_transform.translation.x = (player_translation.x
                            - level_transform.translation.x
                            - orthographic_projection.right / 2.)
                            .clamp(0., level.px_wid as f32 - orthographic_projection.right);
                        camera_transform.translation.y = 0.;
                    } else {
                        // level is taller than the screen
                        orthographic_projection.right = (level.px_wid as f32 / 16.).round() * 16.;
                        orthographic_projection.top = orthographic_projection.right / ASPECT_RATIO;
                        camera_transform.translation.y = (player_translation.y
                            - level_transform.translation.y
                            - orthographic_projection.top / 2.)
                            .clamp(0., level.px_hei as f32 - orthographic_projection.top);
                        camera_transform.translation.x = 0.;
                    }

                    camera_transform.translation.x += level_transform.translation.x;
                    camera_transform.translation.y += level_transform.translation.y;
                }
            }
        }
    }
}

pub fn update_level_selection(
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    for (handle, level) in level_query.iter() {
        if let Some(ldtk_level) = ldtk_levels.get(handle) {
            let level_bounds = Rect {
                bottom: level.translation.y,
                top: level.translation.y + ldtk_level.level.px_hei as f32,
                left: level.translation.x,
                right: level.translation.x + ldtk_level.level.px_wid as f32,
            };

            for player in player_query.iter() {
                if player.translation.x < level_bounds.right
                    && player.translation.x > level_bounds.left
                    && player.translation.y < level_bounds.top
                    && player.translation.y > level_bounds.bottom
                    && !level_selection.is_match(&0, &ldtk_level.level)
                {
                    *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
                }
            }
        }
    }
}

pub fn player_movement(mut query: Query<(&mut Velocity, &MovementController)>) {
    for (mut velocity, movement) in query.iter_mut() {
        let input = movement.0.normalize_or_zero() * 96.;

        velocity.linear.x = input.x;
        velocity.linear.y = input.y;
    }
}

#[derive(Clone, Default, Component)]
pub struct MovementController(Vec2);

pub fn read_player_input(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut MovementController,), With<Player>>,
) {
    for (mut movement,) in query.iter_mut() {
        movement.0 = calc_movement_input(&keyboard);
    }
}

#[derive(Clone, Component, Deref, DerefMut)]
pub struct PlayerAnimationTimer(Timer);

impl Default for PlayerAnimationTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, true))
    }
}

// TODO: Make animator plugin (animations with and without rules)
pub fn player_animation(
    time: Res<Time>,
    mut query: Query<
        (
            &mut PlayerAnimationTimer,
            &mut TextureAtlasSprite,
            &MovementController,
        ),
        With<Player>,
    >,
    mut tick: Local<usize>,
) {
    for (mut timer, mut sprite, movement) in query.iter_mut() {
        let up_indices = vec![0, 1, 2, 1];
        let down_indices = vec![9, 10, 11, 10];

        let left_indices = vec![3, 4, 5, 4];
        let right_indices = vec![6, 7, 8, 7];

        timer.tick(time.delta());
        if timer.just_finished() {
            if movement.0.length_squared() > 0.0 {
                *tick += 1;
                *tick %= 4;

                use std::cmp::Ordering::*;
                sprite.index = match movement.0.x.total_cmp(&0.0) {
                    Less => left_indices[*tick],
                    Greater => right_indices[*tick],
                    Equal => match movement.0.y.total_cmp(&0.0) {
                        Less => up_indices[*tick],
                        Greater => down_indices[*tick],
                        _ => sprite.index,
                    },
                }
            } else {
                *tick = 0;
                if up_indices.contains(&sprite.index) {
                    sprite.index = up_indices[3];
                }
                if down_indices.contains(&sprite.index) {
                    sprite.index = down_indices[3];
                }
                if left_indices.contains(&sprite.index) {
                    sprite.index = left_indices[3];
                }
                if right_indices.contains(&sprite.index) {
                    sprite.index = right_indices[3];
                }
            }
        }
    }
}
