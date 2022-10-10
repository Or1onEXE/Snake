use bevy::{
	prelude::*,
};

use crate::{
	Game,
	Direction,
	BOARD_SIZE_I,
	BOARD_SIZE_J, GameState,
};

pub fn move_snake(
	mut state: ResMut<State<GameState>>,
	mut commands: Commands,
	mut game: ResMut<Game>,
	mut transform: Query<&mut Transform>,
	keyboard_input: Res<Input<KeyCode>>,
	time: Res<Time>,
) {
	fn check(keyboard_input: Input<KeyCode>, key_code: KeyCode) -> bool {
		if keyboard_input.pressed(key_code) || keyboard_input.just_pressed(key_code) 
			{ return true }
		else 
			{ return false }
	}
	
	if check(keyboard_input.clone(), KeyCode::Up) {
		game.snake.matched_direction = Direction::Up;
	} else if check(keyboard_input.clone(), KeyCode::Down) {
		game.snake.matched_direction = Direction::Down;
	} else if check(keyboard_input.clone(), KeyCode::Right) {
		game.snake.matched_direction = Direction::Right;
	} else if check(keyboard_input.clone(), KeyCode::Left) {
		game.snake.matched_direction = Direction::Left;
	}
	
	if game.snake.move_cooldown.tick(time.delta()).finished() {

		for i in 0..game.snake.i.len() {
			info!("i{} = {}; j{} = {}", i, game.snake.i[i], i, game.snake.j[i])
		}
		info!("------------------");
		
		if game.snake.matched_direction == Direction::Up && game.snake.direction != Direction::Down {
			game.snake.direction = game.snake.matched_direction;
		} else if game.snake.matched_direction == Direction::Down && game.snake.direction != Direction::Up {
			game.snake.direction = game.snake.matched_direction;
		} else if game.snake.matched_direction == Direction::Right && game.snake.direction != Direction::Left {
			game.snake.direction = game.snake.matched_direction;
		} else if game.snake.matched_direction == Direction::Left && game.snake.direction != Direction::Right {
			game.snake.direction = game.snake.matched_direction;
		}

		let size: usize = game.snake.i.len();
		for index in 0..size - 1 {
			game.snake.i[size - index - 1] = game.snake.i[size - index - 2];
			game.snake.j[size - index - 1] = game.snake.j[size - index - 2];
		}
			
		match game.snake.direction {
			Direction::Up => {
				if game.snake.i[0] == BOARD_SIZE_I - 1 { game.snake.i[0] = 0; }
				else { game.snake.i[0] += 1; }
			},
			Direction::Down => {
				if game.snake.i[0] == 0 { game.snake.i[0] = BOARD_SIZE_I - 1; }
				else { game.snake.i[0] -= 1; }
			},
			Direction::Right => {
				if game.snake.j[0] == BOARD_SIZE_J - 1 { game.snake.j[0] = 0; }
				else { game.snake.j[0] += 1; }
			},
			Direction::Left => {
				if game.snake.j[0] == 0 { game.snake.j[0] = BOARD_SIZE_J - 1; }
				else { game.snake.j[0] -= 1; }
			}
		}
			
		game.snake.move_cooldown.reset();
		for index in 0..game.snake.size - 1 {
			*transform.get_mut(game.snake.entity[index].unwrap()).unwrap() = Transform {
				translation: Vec3::new(
					game.snake.i[index] as f32,
					game.board[game.snake.j[index]][game.snake.i[index]].height,
					game.snake.j[index] as f32,
				),
				..default()
			};
		}

		game.snake.skip_move = false;
		if let Some(entity) = game.apple.entity {
			if game.snake.i[0] == game.apple.i && game.snake.j[0] == game.apple.j {
				game.snake.new_size += 1;
				commands.entity(entity).despawn_recursive();
				game.apple.entity = None;

				let last: usize = game.snake.i.len();
				let difference_i: i64 = game.snake.i[last - 2] as i64 - game.snake.i[last - 1] as i64;
				let difference_j: i64 = game.snake.j[last - 2] as i64 - game.snake.j[last - 1] as i64;

				let new_i: i64 = (game.snake.i[last - 1] as i64 - difference_i) % BOARD_SIZE_I as i64;
				let new_j: i64 = (game.snake.j[last - 1] as i64 - difference_j) % BOARD_SIZE_J as i64;
				
				game.snake.i.push(new_i as usize);
				game.snake.j.push(new_j as usize);

				game.snake.skip_move = true;
			}
		}

		for index in 1..game.snake.size {
			if game.snake.i[0] == game.snake.i[index] && game.snake.j[0] == game.snake.j[index] {
				let _ = state.overwrite_set(GameState::GameOver);
			}
		}
		
		let mut vec: Vec<Vec<&str>> = vec![vec!["_"; 10]; 10];

		for index in 0..game.snake.i.len() {
			vec[game.snake.i[index]][game.snake.j[index]] = "X";
		}
		
		for i in 0..10 {
			println!("{:?}", vec[10 - i - 1]);
		}
	}
}

pub fn spawn_snake_tile(
	mut game: ResMut<Game>,
	mut commands: Commands,
) {
	if game.snake.new_size == game.snake.size {
		return;
	}
	
	let last: usize = game.snake.size - 1;
	
	let snake_x: f32 = game.snake.i[last] as f32;
	let snake_y: f32 = game.board[game.snake.i[last]][game.snake.j[last]].height + 0.2;
	let snake_z: f32 = game.snake.j[last] as f32;
	let snake_scene: Handle<Scene> = game.snake.handle.clone();
	game.snake.entity.push(
		Some(
			commands.spawn_bundle(
				SceneBundle {
					transform: Transform::from_xyz(
						snake_x,						
						snake_y,						
						snake_z,						
					),
					scene: snake_scene,
					..default()
				}
			)
			.id(),
		)
	);

	game.snake.size += 1;
}
