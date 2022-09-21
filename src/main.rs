#![allow(
	dead_code,
	unused_imports,
	unused_variables,
	unused_assignments,
	unused_mut
)]

use rand::Rng;
use bevy::{
	ecs::schedule::SystemSet, 
	time::FixedTimestep,
	prelude::*, 
};

#[derive(PartialEq, Default)]
enum Direction {
	#[default] Up,
	Down,
	Left,
	Right,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
	Playing,
	GameOver,
}

struct Cell {
	height: f32,
}

#[derive(Default)]
struct Snake {
	entity: Option<Entity>,
	direction: Direction,
    // i: Vec<usize>,
    // j: Vec<usize>,
	i: usize,
    j: usize,
    move_cooldown: Timer,
}

#[derive(Default)]
struct Apple {
	entity: Option<Entity>,
	i: usize,
	j: usize,
	handle: Handle<Scene>,
}

#[derive(Default)]
struct Game {
    board: Vec<Vec<Cell>>,
    player: Snake,
	apple: Apple,
    score: u32,
    apples_eaten: u32,
	camera_should_focus: Vec3,
	camera_is_focus: Vec3,
}

const BOARD_SIZE_I: usize = 10;
const BOARD_SIZE_J: usize = 10;

const RESET_FOCUS: [f32; 3] = [
	BOARD_SIZE_I as f32,
	0.0,
	BOARD_SIZE_J as f32,
];

fn main() {
	App::new()
		.init_resource::<Game>()
		.add_plugins(DefaultPlugins)
		.add_state(GameState::Playing)
		.insert_resource(WindowDescriptor {
			title: "Snake".to_string(),
			width: 1280.,
			height: 720.,
			..default()
		})
		.add_startup_system(setup_camera)
		.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
		.add_system_set(
			SystemSet::on_update(GameState::Playing)
				.with_system(move_snake)
				.with_system(focus_camera)
				.with_system(scoreboard_system)
		)
		.add_system_set(SystemSet::on_exit(GameState::Playing).with_system(teardown))
		.add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(teardown))
		// .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(display_score))
		.add_system_set(SystemSet::on_update(GameState::GameOver).with_system(gameover_keyboard))
		.add_system_set(
			SystemSet::new()
				.with_run_criteria(FixedTimestep::step(5.0))
		)
		.add_system(bevy::window::close_on_esc)
		.run();
}

fn setup_camera(
	mut commands: Commands,
	mut game: ResMut<Game>,
) {
	game.camera_should_focus = Vec3::from(RESET_FOCUS);
	game.camera_is_focus = game.camera_should_focus;

	commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(
            -(BOARD_SIZE_I as f32 / 2.0),
            2.0 * BOARD_SIZE_J as f32 / 3.0,
            BOARD_SIZE_J as f32 / 2.0 - 0.5,
        )
        .looking_at(game.camera_is_focus, Vec3::Y),
        ..default()
    });
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut game: ResMut<Game>
) {
	game.score = 0;
	game.player.i = 5;
	game.player.j = 5;
	game.player.direction = Direction::Down;
	game.player.move_cooldown = Timer::from_seconds(0.3, false);

	commands.spawn_bundle(PointLightBundle {
		transform: Transform::from_xyz(4.0, 10.0, 4.0),
		point_light: PointLight {
			intensity: 3000.0,
			shadows_enabled: true,
			range: 30.0,
			..default()
		},
		..default()
	});

	let cell_scene = asset_server.load("models/tile.glb#Scene0");
	game.board = (0..BOARD_SIZE_J)
        .map(|j| {
            (0..BOARD_SIZE_I)
                .map(|i| {
                    let height = 0_f32;
                    commands.spawn_bundle(SceneBundle {
                        transform: Transform::from_xyz(i as f32, height, j as f32),
                        scene: cell_scene.clone(),
                        ..default()
                    });
                    Cell { height }
                })
                .collect()
        })
        .collect();

	game.player.entity = Some(
		commands
		.spawn_bundle(SceneBundle {
			transform: Transform {
				translation: Vec3::new(
					game.player.i as f32,
					0_f32,
					game.player.j as f32,
				),
				..default()
			},
			scene: asset_server.load("models/red_cube.glb#Scene0"),
			..default()
		})
		.id()
	);

	commands.spawn_bundle(
		TextBundle::from_section(
			"Score: ",
			TextStyle {
				font: asset_server.load("fonts/pixeled.ttf"),
				font_size: 60.0,
				color: Color::rgb(0.5, 0.5, 1.0),
			},
		)
		.with_style(Style {
			position_type: PositionType::Absolute,
			position: UiRect {
				top: Val::Px(5.0),
				left: Val::Px(5.0),
				..default()
			},
			..default()
		}),
	);
}

fn teardown(
	mut commands: Commands,
	entities: Query<Entity, Without<Camera>>,
) {
	for entity in &entities {
		commands.entity(entity).despawn_recursive();
	}
}

fn focus_camera(
	time: Res<Time>,
	mut game: ResMut<Game>,
	mut transform: ParamSet<(Query<&mut Transform, With<Camera3d>>, Query<&Transform>)>
) {
	const SPEED: f32 = 2.0;

	if let(Some(player_entity), Some(apple_entity)) = (game.player.entity, game.apple.entity) {
		let transform_query = transform.p1();
		if let (Ok(player_transform), Ok(apple_transform)) = (
			transform_query.get(player_entity),
			transform_query.get(apple_entity),
		) {
			game.camera_should_focus = player_transform
				.translation
				.lerp(apple_transform.translation, 0.5);
		}
	} else if let Some(player_entity) = game.player.entity {
		if let Ok(player_transform) = transform.p1().get(player_entity) {
			game.camera_should_focus = player_transform.translation;
		}
	} else {
		game.camera_should_focus = Vec3::from(RESET_FOCUS);
	}

	let mut camera_motion = game.camera_should_focus - game.camera_is_focus;
	if camera_motion.length() > 0.2 {
		camera_motion *= SPEED * time.delta_seconds();
		game.camera_is_focus += camera_motion;
	}

	for mut transform in transform.p0().iter_mut() {
		*transform = transform.looking_at(game.camera_is_focus, Vec3::Y);
	}
}

fn move_snake(
	mut commands: Commands,
	keyboard_input: Res<Input<KeyCode>>,
	mut game: ResMut<Game>,
	mut transform: Query<&mut Transform>,
	time: Res<Time>,
) {
	if game.player.move_cooldown.tick(time.delta()).finished() {
        let mut moved: bool = false;
        let mut rotation: f32 = 0.0;

        if keyboard_input.pressed(KeyCode::Up) {
            if game.player.i < BOARD_SIZE_I - 1 {
                game.player.i += 1;
            }
            moved = true;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            if game.player.i > 0 {
                game.player.i -= 1;
            }
            moved = true;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            if game.player.j < BOARD_SIZE_J - 1 {
                game.player.j += 1;
            }
            moved = true;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            if game.player.j > 0 {
                game.player.j -= 1;
            }
            moved = true;
        }

		if moved {
			game.player.move_cooldown.reset();
			*transform.get_mut(game.player.entity.unwrap()).unwrap() = Transform {
				translation: Vec3::new(
					game.player.i as f32,
					0_f32,
					game.player.j as f32,
				),
				..default()
			};
		}	

		if let Some(entity) = game.apple.entity {
			if game.player.i == game.apple.i && game.player.j == game.apple.j {
				game.score += 1;
				commands.entity(entity).despawn_recursive();
				game.apple.entity = None;
			}
		}
	}
}

fn gameover_keyboard(
	mut state: ResMut<State<GameState>>,
	keyboard_input: Res<Input<KeyCode>>,
) {
	if keyboard_input.just_pressed(KeyCode::Space) {
		state.set(GameState::Playing).unwrap();
	}
}

fn scoreboard_system(
	game: Res<Game>,
	mut query: Query<&mut Text>,
) {
	let mut text = query.single_mut();
	text.sections[0].value = format!("Score: {}", game.apples_eaten);
}
