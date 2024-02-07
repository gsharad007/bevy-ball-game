use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::random;

/// The main function that runs the Bevy app.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<AppState>()
        .add_state::<SimulationState>()
        .init_resource::<Score>()
        .init_resource::<HighScore>()
        .init_resource::<EnemySpawnTimer>()
        .init_resource::<StarSpawnTimer>()
        .add_event::<GameOver>()
        .add_systems(Update, transition_to_game_state)
        .add_systems(Update, toggle_simulation.run_if(in_state(AppState::Game)))
        .add_systems(Startup, spawn_camera)
        // .add_system_set(
        //     SystemSet::on_enter(AppState::Game),
        //     (spawn_player, spawn_enemies, spawn_stars),
        // )
        .add_systems(
            OnEnter(AppState::Game),
            (spawn_player, spawn_enemies, spawn_stars),
        )
        .add_systems(
            OnExit(AppState::Game),
            (despawn_player, despawn_enemies, despawn_stars),
        )
        .add_systems(
            Update,
            (
                player_movement,
                confine_player_movement.after(player_movement),
                player_hit_star,
                update_score,
            )
                .run_if(in_state(AppState::Game))
                .run_if(in_state(SimulationState::Running)),
        )
        .add_systems(
            Update,
            (
                enemy_movement,
                bounce_enemies_off_edges,
                confine_enemy_movement.after(enemy_movement),
                enemy_hit_player,
                spawn_enemies_over_time,
            )
                .run_if(in_state(AppState::Game))
                .run_if(in_state(SimulationState::Running)),
        )
        .add_systems(
            Update,
            (tick_timers, spawn_stars_over_time)
                .run_if(in_state(AppState::Game))
                .run_if(in_state(SimulationState::Running)),
        )
        .add_systems(
            Update,
            (
                exit_game,
                handle_game_over,
                update_high_scores,
                high_scores_updated,
            ),
        )
        .run();
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
enum AppState {
    #[default]
    MainMenu,
    Game,
    GameOver,
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
enum SimulationState {
    Running,
    #[default]
    Paused,
}

fn toggle_simulation(
    mut next_simulation_state: ResMut<NextState<SimulationState>>,
    keyboard_input: Res<Input<KeyCode>>,
    simulation_state: Res<State<SimulationState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if *simulation_state == SimulationState::Running {
            next_simulation_state.set(SimulationState::Paused);
            println!("Simulation paused");
        } else if *simulation_state == SimulationState::Paused {
            next_simulation_state.set(SimulationState::Running);
            println!("Simulation running");
        }
    }
}

fn transition_to_game_state(
    mut next_app_state: ResMut<NextState<AppState>>,
    keyboard_input: Res<Input<KeyCode>>,
    app_state: Res<State<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::G) {
        if *app_state != AppState::Game {
            next_app_state.set(AppState::Game);
            println!("Game started");
        }
    } else if keyboard_input.just_pressed(KeyCode::M) {
        if *app_state != AppState::MainMenu {
            next_app_state.set(AppState::MainMenu);
            println!("Enter main menu");
        }
    }
}

const PLAYER_SIZE: f32 = 64.0;
const PLAYER_HALF_SIZE: f32 = PLAYER_SIZE / 2.0;

#[derive(Component)]
struct Player {}

const ENEMY_SIZE: f32 = 64.0;
const ENEMY_HALF_SIZE: f32 = ENEMY_SIZE / 2.0;
const NUMBER_OF_ENEMIES: usize = 4;
const ENEMY_SPAWN_TIME: f32 = 5.0;

#[derive(Component)]
struct Enemy {
    pub direction: Vec2,
}

const STAR_SIZE: f32 = 30.0;
const STAR_HALF_SIZE: f32 = STAR_SIZE / 2.0;
const NUMBER_OF_STARS: usize = 10;
const STAR_SPAWN_TIME: f32 = 1.0;

#[derive(Component)]
struct Star {}

#[derive(Resource)]
struct Score {
    pub value: u32,
}

impl Default for Score {
    fn default() -> Self {
        Self { value: 0 }
    }
}

#[derive(Resource, Debug)]
struct HighScore {
    pub scores: Vec<(String, u32)>,
}

impl Default for HighScore {
    fn default() -> Self {
        Self { scores: Vec::new() }
    }
}

#[derive(Resource)]
struct EnemySpawnTimer {
    pub timer: Timer,
}

impl Default for EnemySpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(ENEMY_SPAWN_TIME, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
struct StarSpawnTimer {
    pub timer: Timer,
}

impl Default for StarSpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(STAR_SPAWN_TIME, TimerMode::Repeating),
        }
    }
}

#[derive(Event)]
struct GameOver {
    pub score: u32,
}

/// Spawns a player entity with a blue ball sprite in the center of the primary window.
fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    let position = get_player_position_clamped(window);
    commands.spawn(get_player_bundle(position, asset_server));
}

/// Spawns a player entity with a blue ball sprite in the center of the primary window.
fn despawn_player(mut commands: Commands, entity_query: Query<Entity, With<Player>>) {
    if let Ok(entity) = entity_query.get_single() {
        commands.entity(entity).despawn();
    }
}

fn get_player_position_clamped(window: &Window) -> Vec3 {
    clamp_half_sized_to_window(
        Vec3::new(window.width() / 2.0, window.height() / 2.0, 0.0),
        window,
        PLAYER_HALF_SIZE,
    )
}

fn get_player_bundle(position: Vec3, asset_server: Res<AssetServer>) -> (SpriteBundle, Player) {
    (
        SpriteBundle {
            transform: Transform::from_translation(position),
            texture: asset_server.load("sprites/ball_blue_large.png"),
            ..default()
        },
        Player {},
    )
}

fn spawn_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    for _ in 0..NUMBER_OF_ENEMIES {
        let position = get_enemy_position_clamped(window);
        let direction = get_enemy_direction();
        commands.spawn(get_enemy_bundle(position, &asset_server, direction));
    }
}

fn despawn_enemies(mut commands: Commands, entity_query: Query<Entity, With<Enemy>>) {
    for entity in entity_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn_enemies_over_time(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    enemy_spawn_timer: Res<EnemySpawnTimer>,
) {
    if enemy_spawn_timer.timer.finished() {
        let window = window_query.get_single().unwrap();

        let position = get_enemy_position_clamped(window);
        let direction = get_enemy_direction();
        commands.spawn(get_enemy_bundle(position, &asset_server, direction));
    }
}

fn get_enemy_position_clamped(window: &Window) -> Vec3 {
    clamp_half_sized_to_window(
        Vec3::new(
            random::<f32>() * window.width(),
            random::<f32>() * window.height(),
            0.0,
        ),
        window,
        ENEMY_HALF_SIZE,
    )
}

fn get_enemy_direction() -> Vec2 {
    Vec2::new(random::<f32>() * 2.0 - 1.0, random::<f32>() * 2.0 - 1.0).normalize()
}

fn get_enemy_bundle(
    position: Vec3,
    asset_server: &Res<AssetServer>,
    enemy_direction: Vec2,
) -> (SpriteBundle, Enemy) {
    (
        SpriteBundle {
            transform: Transform::from_translation(position),
            texture: asset_server.load("sprites/ball_red_large.png"),
            ..default()
        },
        Enemy {
            direction: enemy_direction,
        },
    )
}

fn spawn_stars(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    for _ in 0..NUMBER_OF_STARS {
        let position = get_star_position_clamped(window);
        commands.spawn(get_star_bundle(position, &asset_server));
    }
}

fn despawn_stars(mut commands: Commands, entity_query: Query<Entity, With<Star>>) {
    for entity in entity_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn tick_timers(
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    mut star_spawn_timer: ResMut<StarSpawnTimer>,
    time: Res<Time>,
) {
    enemy_spawn_timer.timer.tick(time.delta());
    star_spawn_timer.timer.tick(time.delta());
}

fn spawn_stars_over_time(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    star_spawn_timer: Res<StarSpawnTimer>,
) {
    if star_spawn_timer.timer.finished() {
        let window = window_query.get_single().unwrap();
        let position = get_star_position_clamped(window);
        commands.spawn(get_star_bundle(position, &asset_server));
    }
}

fn get_star_bundle(position: Vec3, asset_server: &Res<'_, AssetServer>) -> (SpriteBundle, Star) {
    (
        SpriteBundle {
            transform: Transform::from_translation(position),
            texture: asset_server.load("sprites/star.png"),
            ..default()
        },
        Star {},
    )
}

fn get_star_position_clamped(window: &Window) -> Vec3 {
    clamp_half_sized_to_window(
        Vec3::new(
            random::<f32>() * window.width(),
            random::<f32>() * window.height(),
            0.0,
        ),
        window,
        STAR_HALF_SIZE,
    )
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
    mut game_over_event_writer: EventWriter<GameOver>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
) {
    if let Ok((player_entity, player_transform)) = player_query.get_single() {
        for enemy_transform in enemy_query.iter() {
            let colided = circular_collision(
                player_transform,
                PLAYER_HALF_SIZE,
                enemy_transform,
                ENEMY_HALF_SIZE,
            );
            if colided {
                println!("Player Hit! GAME OVER!");

                commands.entity(player_entity).despawn();

                let sound_effect = asset_server.load("audio/explosionCrunch_000.ogg");
                commands.spawn(AudioBundle {
                    source: sound_effect,
                    settings: PlaybackSettings::DESPAWN,
                    ..default()
                });
                game_over_event_writer.send(GameOver { score: score.value });
            }
        }
    }
}

fn player_hit_star(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    star_query: Query<(Entity, &Transform), With<Star>>,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Score>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (star_entity, star_transform) in star_query.iter() {
            let colided = circular_collision(
                player_transform,
                PLAYER_HALF_SIZE,
                star_transform,
                STAR_HALF_SIZE,
            );
            if colided {
                println!("Player Hit Star! Score +1!");
                score.value += 1;
                commands.entity(star_entity).despawn();
                let sound_effect = asset_server.load("audio/laserLarge_000.ogg");
                commands.spawn(AudioBundle {
                    source: sound_effect,
                    settings: PlaybackSettings::DESPAWN,
                    ..default()
                });
            }
        }
    }
}

fn circular_collision(
    first_transform: &Transform,
    first_half_size: f32,
    second_transform: &Transform,
    second_half_size: f32,
) -> bool {
    let distance = first_transform
        .translation
        .distance(second_transform.translation);
    let colided = distance < first_half_size + second_half_size;
    colided
}

fn update_score(score: Res<Score>) {
    if score.is_changed() {
        println!("Score: {}", score.value);
    }
}

fn handle_game_over(mut game_over_event_reader: EventReader<GameOver>) {
    for game_over in game_over_event_reader.read() {
        println!("Game Over! Score: {}", game_over.score);
    }
}

fn exit_game(keyboard_input: Res<Input<KeyCode>>, mut app_exit_event_writer: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_event_writer.send(AppExit);
    }
}

fn update_high_scores(
    mut game_over_event_reader: EventReader<GameOver>,
    mut high_score: ResMut<HighScore>,
) {
    for game_over in game_over_event_reader.read() {
        high_score
            .scores
            .push(("Player".to_string(), game_over.score));
    }
}

fn high_scores_updated(high_scores: Res<HighScore>) {
    if high_scores.is_changed() {
        println!("High Scores: {:?}", high_scores.scores);
    }
}
