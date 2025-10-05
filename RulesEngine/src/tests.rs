pub use dissimilar::diff as __diff;
use crate::console_interface::{parse_level, render_game_to_string};
use crate::core::*;

#[macro_export]
macro_rules! assert_eq_text {
    ($left:expr, $right:expr) => {
        assert_eq_text!($left, $right,)
    };
    ($left:expr, $right:expr, $($tt:tt)*) => {{
        let left = $left;
        let right = $right;
        if left != right {
            if left.trim() == right.trim() {
                std::eprintln!("Left:\n{:?}\n\nRight:\n{:?}\n\nWhitespace difference\n", left, right);
            } else {
                let diff = $crate::tests::__diff(left, right);
                std::eprintln!("Left:\n{}\n\nRight:\n{}\n\nDiff:\n{}\n", left, right, $crate::tests::format_diff(diff));
            }
            std::eprintln!($($tt)*);
            panic!("text differs");
        }
    }};
}

pub fn format_diff(chunks: Vec<dissimilar::Chunk>) -> String {
    let mut buf = String::new();
    for chunk in chunks {
        let formatted = match chunk {
            dissimilar::Chunk::Equal(text) => text.into(),
            dissimilar::Chunk::Delete(text) => format!("\x1b[41m{}\x1b[0m", text),
            dissimilar::Chunk::Insert(text) => format!("\x1b[42m{}\x1b[0m", text),
        };
        buf.push_str(&formatted);
    }
    buf
}

struct GameTestState {
    game_state: GameState,
    shared: SharedGameState,
}

impl GameTestState {
    fn new(level: &str) -> Self {
        let (game_state, shared) = parse_level(level);
        Self { game_state, shared }
    }

    fn game_to_string(&self) -> String {
        render_game_to_string(&self.shared, &self.game_state).trim_matches('\n').into()
    }

    fn assert_move(&mut self, direction: Direction) -> GameUpdate {
        self.assert_step(UserAction::Move(direction))
    }

    fn assert_moves(&mut self, directions: &[Direction]) {
        for &dir in directions {
            self.assert_move(dir);
        }
    }

    fn assert_step(&mut self, action: UserAction) -> GameUpdate {
        let update = step(&self.shared, &self.game_state, action);
        let GameUpdate::NextState(new_state, _change_type) = &update else {
            panic!("Expected NextState update, got {:?}, in map {}", update, self.game_to_string());
        };

        self.game_state = new_state.clone();
        update
    }

    fn assert_matches(&self, expected: &str) {
        let actual = self.game_to_string();
        assert_eq_text!(expected.trim_matches('\n'), actual.as_str().trim_matches('\n'));
    }
}


mod test {
    use Direction::*;
    use crate::core::*;
    use crate::tests::GameTestState;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn when_move_right_observes_move_right(){
        let level = r#"
#@ #
"#;
        let mut game = GameTestState::new(level);
        game.assert_step(UserAction::Move(Right));

        let expected_level = r#"
# @#
"#;
        game.assert_matches(expected_level);
    }

    #[test]
    fn when_push_pushes(){
        let level = r#"
#@$ #
"#;
        let mut game = GameTestState::new(level);
        game.assert_step(UserAction::Move(Right));

        let expected_level = r#"
# @$#
"#;
        game.assert_matches(expected_level);
    }

    #[test]
    fn when_block_moves_game_is_inequal(){
        let level = r#"
#@$ #
"#;
        let mut game = GameTestState::new(level);
        let original_state = game.game_state.clone();
        game.assert_move(Right);
        let new_state = game.game_state.clone();

        let expected_level = r#"
# @$#
"#;
        game.assert_matches(expected_level);
        assert_ne!(original_state, new_state);
    }

    #[test]
    fn when_player_moves_back_game_is_equal(){
        let level = r#"
#@ $#
"#;
        let mut game = GameTestState::new(level);
        let original_state = game.game_state.clone();
        game.assert_move(Right);
        game.assert_move(Left);
        let new_state = game.game_state.clone();

        let expected_level = r#"
#@ $#
"#;
        game.assert_matches(expected_level);
        assert_eq!(original_state, new_state);
    }

    #[test]
    fn when_blocks_swap_game_remains_equal(){
        let level = r#"
#    #
#@$  #
# $  #
#    #
"#;
        let mut game = GameTestState::new(level);
        let original_state = game.game_state.clone();
        game.assert_moves(&[
            Right, Left,
            Down, Down,
            Right, Up,
            Right, Right, Up, Up,
            Left, Down, Right, Down, Left,]);
        game.assert_matches(r#"
#    #
# $  #
# $@ #
#    #
"#);
        game.assert_moves(&[
            Down, Left, Left, Up, Up,]);
        let new_state = game.game_state.clone();


        let expected_level = r#"
#    #
#@$  #
# $  #
#    #
"#;
        game.assert_matches(expected_level);

        assert_eq!(original_state, new_state);
    }
}