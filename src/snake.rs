use bevy::{
	prelude::*,
};

use crate::{
	Game,
	Direction,
	BOARD_SIZE_I,
	BOARD_SIZE_J,
};

pub fn move_snake(
	mut commands: Commands,
	keyboard_input: Res<Input<KeyCode>>,
	mut game: ResMut<Game>,
	mut transform: Query<&mut Transform>,
	time: Res<Time>,
) {
	for i in 0..game.snake.i.len() {
		info!("{} {}", game.snake.i[i], game.snake.j[i])
	}
	info!("------------------");
	
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

		if game.snake.matched_direction == Direction::Up && game.snake.direction != Direction::Down {
			game.snake.direction = game.snake.matched_direction;
		} else if game.snake.matched_direction == Direction::Down && game.snake.direction != Direction::Up {
			game.snake.direction = game.snake.matched_direction;
		} else if game.snake.matched_direction == Direction::Right && game.snake.direction != Direction::Left {
			game.snake.direction = game.snake.matched_direction;
		} else if game.snake.matched_direction == Direction::Left && game.snake.direction != Direction::Right {
			game.snake.direction = game.snake.matched_direction;
		}
		
		// change direction 
		let size: usize = game.snake.i.len();
		for index in 0..size - 1 {
			game.snake.i[size - index - 1] = game.snake.i[size - index];
			game.snake.j[size - index - 1] = game.snake.j[size - index];
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

		for _i in 1..game.snake.size {

		}
		
		game.snake.move_cooldown.reset();
		*transform.get_mut(game.snake.entity.unwrap()).unwrap() = Transform {
			translation: Vec3::new(
				game.snake.i[0] as f32,
				game.board[game.snake.j[0]][game.snake.i[0]].height,
				game.snake.j[0] as f32,
			),
			..default()
		};
	
		if let Some(entity) = game.apple.entity {
			if game.snake.i[0] == game.apple.i && game.snake.j[0] == game.apple.j {
				game.snake.size += 1;
				commands.entity(entity).despawn_recursive();
				game.apple.entity = None;

				let new_i: usize = game.snake.i[0];
				let new_j: usize = game.snake.j[0];
				game.snake.i.push(new_i);
				game.snake.j.push(new_j);
			}
		}
	}
}
