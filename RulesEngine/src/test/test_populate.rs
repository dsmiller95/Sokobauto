
#[cfg(test)]
mod test {
    use crate::core::*;
    use crate::state_graph::{get_all_adjacent_nodes, UniqueNode};
    use crate::test::test_util::{assert_game_set_matches, assert_symbols_match, GameTestState};

    #[test]
    fn find_reachable_finds_all_reachable(){
        let level = r#"
#@ #
#  #
##$#
#  #
"#;
        let game = GameTestState::new(level);
        let reachable = game.shared.reachable_positions(&game.game_state).into_iter().map(|x| x.into()).collect();

        let expected_symbols = r#"
_**_
_**_
____
____
"#;
        let actual_symbols = game.render_where_present(reachable, '*', '_');
        assert_symbols_match(expected_symbols, actual_symbols.as_str());
    }

    #[test]
    fn find_adjacent_nodes_finds_all_possible_actions(){
        let level = r#"
######
#@ $ #
# $  #
#  $ #
######
"#;
        let game = GameTestState::new(level);
        let source_node = UniqueNode {
            environment: game.game_state.environment.clone(),
            minimum_reachable_player_position: game.game_state.player.into(),
        };
        let new_game_states: Vec<GameState> = get_all_adjacent_nodes(&source_node, &game.shared).into_iter()
            .map(|node| GameState {
                player: node.minimum_reachable_player_position.into(),
                environment: node.environment,
            })
            .collect();

        let expected_moves = vec![r#"
######
#@  $#
# $  #
#  $ #
######
"#,r#"
######
#@ $ #
#    #
# $$ #
######
"#,r#"
######
#@ $ #
#  $ #
#  $ #
######
"#,r#"
######
#@$$ #
#    #
#  $ #
######
"#,r#"
######
#@ $ #
# $  #
#   $#
######
"#,
        ];

        assert_game_set_matches(&new_game_states, game.shared, expected_moves);
    }
}