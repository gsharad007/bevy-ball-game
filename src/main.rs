use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::random;

/// The main function that runs the Bevy app.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (spawn_camera, spawn_player, spawn_enemies))
        .add_systems(Update, (player_movement, confine_player_movement))
        .add_systems(
            Update,
            (
                enemy_movement,
                bounce_enemies_off_edges,
                confine_enemy_movement,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player {}

#[derive(Component)]
struct Enemy {
    pub direction: Vec2,
}

/// Spawns a player entity with a blue ball sprite in the center of the primary window.
fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
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

    const NUMBER_OF_ENEMIES: usize = 4;

    for _ in 0..NUMBER_OF_ENEMIES {
        let random_x = random::<f32>() * window.width();
        let random_y = random::<f32>() * window.height();
        let enemy_direction =
            Vec2::new(random::<f32>() * 2.0 - 1.0, random::<f32>() * 2.0 - 1.0).normalize();

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(random_x, random_y, 0.0),
                texture: asset_server.load("sprites/ball_red_large.png"),
                ..default()
            },
            Enemy {
                direction: enemy_direction,
            },
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

    const PLAYER_SIZE: f32 = 64.0;
    const HALF_PLAYER_SIZE: f32 = PLAYER_SIZE / 2.0;

    let x_min = 0.0_f32 + HALF_PLAYER_SIZE;
    let y_min = 0.0_f32 + HALF_PLAYER_SIZE;
    let x_max = window.width() - HALF_PLAYER_SIZE;
    let y_max = window.height() - HALF_PLAYER_SIZE;

    if let Ok(mut player_transform) = player_query.get_single_mut() {
        player_transform.translation.x = player_transform.translation.x.clamp(x_min, x_max);
        player_transform.translation.y = player_transform.translation.y.clamp(y_min, y_max);
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
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();

    for (enemy_transform, mut enemy) in enemy_query.iter_mut() {
        const ENEMY_SIZE: f32 = 64.0;
        const HALF_ENEMY_SIZE: f32 = ENEMY_SIZE / 2.0;

        let x_min = 0.0_f32 + HALF_ENEMY_SIZE;
        let y_min = 0.0_f32 + HALF_ENEMY_SIZE;
        let x_max = window.width() - HALF_ENEMY_SIZE;
        let y_max = window.height() - HALF_ENEMY_SIZE;

        if enemy_transform.translation.x <= x_min || enemy_transform.translation.x >= x_max {
            enemy.direction.x = -enemy.direction.x;
        }
        if enemy_transform.translation.y <= y_min || enemy_transform.translation.y >= y_max {
            enemy.direction.y = -enemy.direction.y;
        }
    }
}

fn confine_enemy_movement(
    mut enemy_query: Query<&mut Transform, With<Enemy>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();

    const ENEMY_SIZE: f32 = 64.0;
    const HALF_ENEMY_SIZE: f32 = ENEMY_SIZE / 2.0;

    let x_min = 0.0_f32 + HALF_ENEMY_SIZE;
    let y_min = 0.0_f32 + HALF_ENEMY_SIZE;
    let x_max = window.width() - HALF_ENEMY_SIZE;
    let y_max = window.height() - HALF_ENEMY_SIZE;

    for mut enemy_transform in enemy_query.iter_mut() {
        enemy_transform.translation.x = enemy_transform.translation.x.clamp(x_min, x_max);
        enemy_transform.translation.y = enemy_transform.translation.y.clamp(y_min, y_max);
    }
}
