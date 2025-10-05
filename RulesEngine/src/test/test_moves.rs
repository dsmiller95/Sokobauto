
mod test {
    use Direction::*;
    use crate::core::*;
    use crate::test::test_util::GameTestState;

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
    fn when_block_pushed_into_block_remains_two_blocks(){
        let level = r#"
#@$$ #
"#;
        let mut game = GameTestState::new(level);
        game.try_step(UserAction::Move(Right));

        let expected_level = r#"
#@$$ #
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