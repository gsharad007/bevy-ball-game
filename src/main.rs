use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::random;

/// The main function that runs the Bevy app.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (spawn_camera, spawn_player, spawn_enemies, spawn_stars),
        )
        .add_systems(Update, (player_movement, confine_player_movement))
        .add_systems(
            Update,
            (
                enemy_movement,
                bounce_enemies_off_edges,
                confine_enemy_movement,
                enemy_hit_player,
            ),
        )
        .run();
}

const PLAYER_SIZE: f32 = 64.0;
const PLAYER_HALF_SIZE: f32 = PLAYER_SIZE / 2.0;

#[derive(Component)]
struct Player {}

const ENEMY_SIZE: f32 = 64.0;
const ENEMY_HALF_SIZE: f32 = ENEMY_SIZE / 2.0;
const NUMBER_OF_ENEMIES: usize = 4;

#[derive(Component)]
struct Enemy {
    pub direction: Vec2,
}

const STAR_SIZE: f32 = 30.0;
const STAR_HALF_SIZE: f32 = STAR_SIZE / 2.0;
const NUMBER_OF_STARS: usize = 10;

#[derive(Component)]
struct Star {}

/// Spawns a player entity with a blue ball sprite in the center of the primary window.
fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    let position = clamp_half_sized_to_window(
        Vec3::new(window.width() / 2.0, window.height() / 2.0, 0.0),
        window,
        PLAYER_HALF_SIZE,
    );

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(position),
            texture: asset_server.load("sprites/ball_blue_large.png"),
            ..default()
        },
        Player {},
    ));
}

fn spawn_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    for _ in 0..NUMBER_OF_ENEMIES {
        let position = clamp_half_sized_to_window(
            Vec3::new(
                random::<f32>() * window.width(),
                random::<f32>() * window.height(),
                0.0,
            ),
            window,
            ENEMY_HALF_SIZE,
        );
        let enemy_direction =
            Vec2::new(random::<f32>() * 2.0 - 1.0, random::<f32>() * 2.0 - 1.0).normalize();

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(position),
                texture: asset_server.load("sprites/ball_red_large.png"),
                ..default()
            },
            Enemy {
                direction: enemy_direction,
            },
        ));
    }
}

fn spawn_stars(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    for _ in 0..NUMBER_OF_STARS {
        let position = clamp_half_sized_to_window(
            Vec3::new(
                random::<f32>() * window.width(),
                random::<f32>() * window.height(),
                0.0,
            ),
            window,
            STAR_HALF_SIZE,
        );
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(position),
                texture: asset_server.load("sprites/star.png"),
                ..default()
            },
            Star {},
        ));
    }
}

/// Spawns a 2D camera in the center of the primary window.
fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            direction += Vec3::new(0.0, -1.0, 0.0);
        }

        direction = direction.normalize_or_zero();

        let delta_time = time.delta_seconds();
        const PLAYER_SPEED: f32 = 500.0;
        player_transform.translation += direction * PLAYER_SPEED * delta_time;
    }
}

fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();

    if let Ok(mut player_transform) = player_query.get_single_mut() {
        player_transform.translation =
            clamp_half_sized_to_window(player_transform.translation, &window, PLAYER_HALF_SIZE);
    }
}

fn enemy_movement(mut enemy_query: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
    for (mut enemy_transform, enemy) in enemy_query.iter_mut() {
        let delta_time = time.delta_seconds();
        let direction = Vec3::new(enemy.direction.x, enemy.direction.y, 0.0);
        const ENEMY_SPEED: f32 = 200.0;
        enemy_transform.translation += direction * ENEMY_SPEED * delta_time;
    }
}

fn bounce_enemies_off_edges(
    mut commands: Commands,
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    for (enemy_transform, mut enemy) in enemy_query.iter_mut() {
        if let Some(new_direction) = calculate_new_direction(
            enemy.direction,
            enemy_transform.translation,
            window,
            ENEMY_HALF_SIZE,
        ) {
            enemy.direction = new_direction;

            let sound_effect = if random::<f32>() < 0.5 {
                asset_server.load("audio/pluck_001.ogg")
            } else {
                asset_server.load("audio/pluck_002.ogg")
            };
            commands.spawn(AudioBundle {
                source: sound_effect,
                ..default()
            });
        }
    }
}

fn clamp_half_sized_to_window(translation: Vec3, window: &Window, half_size: f32) -> Vec3 {
    let (x_min, y_min, x_max, y_max) = calculate_play_area_limits(half_size, window);

    Vec3::new(
        translation.x.clamp(x_min, x_max),
        translation.y.clamp(y_min, y_max),
        translation.z,
    )
}

fn calculate_play_area_limits(half_size: f32, window: &Window) -> (f32, f32, f32, f32) {
    let x_min = 0.0_f32 + half_size;
    let y_min = 0.0_f32 + half_size;
    let x_max = window.width() - half_size;
    let y_max = window.height() - half_size;
    (x_min, y_min, x_max, y_max)
}

fn calculate_new_direction(
    direction: Vec2,
    translation: Vec3,
    window: &Window,
    half_size: f32,
) -> Option<Vec2> {
    let (x_min, y_min, x_max, y_max) = calculate_play_area_limits(half_size, window);

    let mut new_direction = None;

    if translation.x <= x_min {
        new_direction = Some(Vec2::new(direction.x.abs(), direction.y));
    } else if translation.x >= x_max {
        new_direction = Some(Vec2::new(-direction.x.abs(), direction.y));
    }

    if translation.y <= y_min {
        new_direction = Some(new_direction.map_or_else(
            || Vec2::new(direction.x, direction.y.abs()),
            |d| Vec2::new(d.x, direction.y.abs()),
        ));
    } else if translation.y >= y_max {
        new_direction = Some(new_direction.map_or_else(
            || Vec2::new(direction.x, -direction.y.abs()),
            |d| Vec2::new(d.x, -direction.y.abs()),
        ));
    }

    new_direction
}

fn confine_enemy_movement(
    mut enemy_query: Query<&mut Transform, With<Enemy>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();

    for mut enemy_transform in enemy_query.iter_mut() {
        enemy_transform.translation =
            clamp_half_sized_to_window(enemy_transform.translation, &window, ENEMY_HALF_SIZE);
    }
}

fn enemy_hit_player(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    asset_server: Res<AssetServer>,
) {
    if let Ok((player_entity, player_transform)) = player_query.get_single_mut() {
        for enemy_transform in enemy_query.iter() {
            let distance = player_transform
                .translation
                .distance(enemy_transform.translation);
            if distance < PLAYER_HALF_SIZE + ENEMY_HALF_SIZE {
                println!("Player Hit! GAME OVER!");

                commands.entity(player_entity).despawn();

                let sound_effect = asset_server.load("audio/explosionCrunch_000.ogg");
                commands.spawn(AudioBundle {
                    source: sound_effect,
                    settings: PlaybackSettings::DESPAWN,
                    ..default()
                });
            }

            // if collide(
            //     player_transform.translation,
            //     Vec2::new(PLAYER_SIZE, PLAYER_SIZE),
            //     enemy_transform.translation,
            //     Vec2::new(ENEMY_SIZE, ENEMY_SIZE),
            // )
            // .is_some()
            // {
            //     commands.entity(player_entity).despawn();
            //     let sound_effect = asset_server.load("audio/explosionCrunch_000.ogg");
            //     commands.spawn(AudioBundle {
            //         source: sound_effect,
            //         ..default()
            //     });
            // }
        }
    }
}
