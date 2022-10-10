use rand::Rng;
use bevy::{
	prelude::*,
};

use crate::{
	GameState,
	Game,
	BOARD_SIZE_I,
	BOARD_SIZE_J,
};

pub fn spawn_apple(
	state: ResMut<State<GameState>>,
	mut game: ResMut<Game>,
	mut commands: Commands,
) {
	if *state.current() == GameState::Playing && game.apple.entity != None {
		return;
	}

	loop {
		game.apple.i = rand::thread_rng().gen_range(0..BOARD_SIZE_I);
		game.apple.j = rand::thread_rng().gen_range(0..BOARD_SIZE_J);
		let mut flag: bool = true;
		for k in 0..game.snake.size - 1 {
			if game.apple.i == game.snake.i[k] && game.apple.j == game.snake.j[k] {
				flag = false;
			}
		}
		if flag { break }
	}

	game.apple.entity = Some(
		commands.spawn_bundle(
			SceneBundle {
				transform: Transform::from_xyz(
					game.apple.i as f32,
					game.board[game.apple.i][game.apple.j].height + 0.2,
					game.apple.j as f32,
				),
				scene: game.apple.handle.clone(),
				..default()
			}
		)
		.with_children(|children| {
			children.spawn_bundle( PointLightBundle {
				point_light: PointLight {
					color: Color::rgb(1.0, 1.0, 0.0),
					intensity: 1000.0,
					range: 10.0,
					..default()
				},
				transform: Transform::from_xyz(0.0, 2.0, 0.0),
				..default()
			});
		})
		.id(),
	);
}
