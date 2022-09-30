// #![allow(
// 	dead_code,
// 	unused_imports,
// 	unused_variables,
// 	unused_assignments,
// 	unused_mut
// )]

use rand::Rng;
use bevy::{
	ecs::schedule::SystemSet, 
	time::FixedTimestep,
	prelude::*, 
};

mod snake;
mod apples;

struct Cell {
	height: f32,
}

#[derive(Default)]
struct Snake {
	entity: Option<Entity>,
	direction: Direction,
	matched_direction: Direction,
	i: Vec<usize>,
	j: Vec<usize>,
	size: usize,
	move_cooldown: Timer,
}

#[derive(PartialEq, Default, Clone, Copy, Debug)]
enum Direction {
	#[default] Up,
	Down,
	Left,
	Right,
}

#[derive(Default)]
struct Apple {
	entity: Option<Entity>,
	i: usize,
	j: usize,
	handle: Handle<Scene>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
	Playing,
	GameOver,
}

#[derive(Default)]
pub struct Game {
	board: Vec<Vec<Cell>>,
	snake: Snake,
	apple: Apple,
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
				.with_system(snake::move_snake)
				.with_system(focus_camera)
				.with_system(scoreboard_system)
		)
		.add_system_set(SystemSet::on_exit(GameState::Playing).with_system(teardown))
		.add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(teardown))
		.add_system_set(SystemSet::on_update(GameState::GameOver).with_system(gameover_keyboard))
		.add_system_set(
			SystemSet::new()
				.with_run_criteria(FixedTimestep::step(0.01))
				.with_system(apples::spawn_apple)
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
	game.snake.size = 1;
	game.snake.i = vec!(BOARD_SIZE_I / 2);
	game.snake.j = vec!(BOARD_SIZE_J / 2);
	game.snake.direction = Direction::Down;
	game.snake.move_cooldown = Timer::from_seconds(0.3, false);
	game.snake.direction = Direction::Right;
	game.snake.matched_direction = Direction::Right;

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
					let height = rand::thread_rng().gen_range(-0.05..0.05);
					commands.spawn_bundle(SceneBundle {
						transform: Transform::from_xyz(i as f32, height - 0.2, j as f32),
						scene: cell_scene.clone(),
						..default()
					});
					Cell { height }
				})
				.collect()
		})
		.collect();

	game.snake.entity = Some(
		commands
		.spawn_bundle(SceneBundle {
			transform: Transform {
				translation: Vec3::new(
					game.snake.i[0] as f32,
					0_f32,
					game.snake.j[0] as f32,
				),
				..default()
			},
			scene: asset_server.load("models/alien.glb#Scene0"),
			..default()
		})
		.id()
	);

	game.apple.handle = asset_server.load("models/red_cube.glb#Scene0");

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

	if let(Some(player_entity), Some(apple_entity)) = (game.snake.entity, game.apple.entity) {
		let transform_query = transform.p1();
		if let (Ok(player_transform), Ok(apple_transform)) = (
			transform_query.get(player_entity),
			transform_query.get(apple_entity),
		) {
			game.camera_should_focus = player_transform
				.translation
				.lerp(apple_transform.translation, 0.5);
		}
	} else if let Some(player_entity) = game.snake.entity {
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
	text.sections[0].value = format!("Score: {}", game.snake.size * 10 - 10);
}
