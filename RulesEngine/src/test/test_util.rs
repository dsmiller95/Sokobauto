use bevy::math::IVec2;
pub use dissimilar::diff as __diff;
use crate::console_interface::{parse_level, render_game_to_string};
use crate::core::{step, Direction, GameState, GameUpdate, SharedGameState, UserAction};

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
                let diff = $crate::test::test_util::__diff(left, right);
                std::eprintln!("Left:\n{}\n\nRight:\n{}\n\nDiff:\n{}\n", left, right, $crate::test::test_util::format_diff(diff));
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

pub struct GameTestState {
    pub game_state: GameState,
    pub shared: SharedGameState,
}

impl GameTestState {
    pub fn new(level: &str) -> Self {
        let (game_state, shared) = parse_level(level);
        Self { game_state, shared }
    }

    pub fn game_to_string(&self) -> String {
        render_game_to_string(&self.shared, &self.game_state).trim_matches('\n').into()
    }

    pub fn assert_move(&mut self, direction: Direction) -> GameUpdate {
        self.assert_step(UserAction::Move(direction))
    }

    pub fn assert_moves(&mut self, directions: &[Direction]) {
        for &dir in directions {
            self.assert_move(dir);
        }
    }

    pub fn assert_step(&mut self, action: UserAction) -> GameUpdate {
        let update = step(&self.shared, &self.game_state, action);
        let GameUpdate::NextState(new_state, _change_type) = &update else {
            panic!("Expected NextState update, got {:?}, in map {}", update, self.game_to_string());
        };

        self.game_state = new_state.clone();
        update
    }

    pub fn try_step(&mut self, action: UserAction) -> GameUpdate {
        let update = step(&self.shared, &self.game_state, action);
        if let GameUpdate::NextState(new_state, _change_type) = &update {
            self.game_state = new_state.clone();
        };

        update
    }

    pub fn assert_matches(&self, expected: &str) {
        let actual = self.game_to_string();
        assert_eq_text!(expected.trim_matches('\n'), actual.as_str().trim_matches('\n'));
    }

    pub fn render_symbols<F>(&self, get_char: F) -> String
    where
        F: Fn(&IVec2) -> char,
    {
        let mut result = String::new();
        for y in 0..self.shared.height() {
            for x in 0..self.shared.width() {
                let pos = IVec2 { y: y as i32, x: x as i32 };
                result.push(get_char(&pos));
            }
            result.push('\n');
        }
        result
    }

    pub fn render_where_present(&self, positions: Vec<IVec2>, present: char, absent: char) -> String
    {
        let get_char = |pos: &IVec2| {
            if positions.contains(pos) {
                present
            } else {
                absent
            }
        };
        self.render_symbols(get_char)
    }
}

pub fn assert_symbols_match(expected: &str, actual: &str) {
    assert_eq_text!(expected.trim_matches('\n'), actual.trim_matches('\n'));
}

pub fn assert_game_set_matches(actual: &Vec<GameState>, shared: SharedGameState, mut expected: Vec<&str>) {
    assert_eq!(actual.len(), expected.len(), "Number of game states differ");
    let mut actual_maps: Vec<String> = actual.iter()
        .map(|state| render_game_to_string(&shared, state))
        .collect();
    actual_maps.sort();
    expected.sort();

    for (actual_str, &expected_str) in actual_maps.iter().zip(expected.iter()) {
        assert_eq_text!(expected_str.trim_matches('\n'), actual_str.trim_matches('\n'));
    }
}