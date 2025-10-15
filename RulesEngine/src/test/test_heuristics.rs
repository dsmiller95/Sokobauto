
#[cfg(test)]
mod test {
    use crate::core::*;
    use crate::test::test_util::{GameTestState};

    fn assert_level_winnable_state(expected: WinnableState, level: &str) {
        let game = GameTestState::new(level);
        let winnable = is_winnable(&game.shared, &game.game_state);

        assert_eq!(expected, winnable, "Expected {:?} but found was {:?}, for level: {}", expected, winnable, level);
    }

    #[test]
    fn is_winnable_with_single_free_box_win_possible() {
        assert_level_winnable_state(WinnableState::WinMaybePossible, r#"
####
#@ #
# $#
#. #
####
"#);
        assert_level_winnable_state(WinnableState::WinMaybePossible, r#"
#####
#@$.#
#####
"#);
        assert_level_winnable_state(WinnableState::WinMaybePossible, r#"
######
#@.$ #
######
"#);
        assert_level_winnable_state(WinnableState::WinMaybePossible, r#"
#####
#@ *#
#####
"#);
    }

    #[test]
    fn is_winnable_with_two_free_boxes_one_target_win_possible() {
        assert_level_winnable_state(WinnableState::WinMaybePossible, r#"
####
#@ #
#$$#
#. #
####
"#);
        assert_level_winnable_state(WinnableState::WinMaybePossible, r#"
#####
#@$.#
#$  #
#####
"#);
        assert_level_winnable_state(WinnableState::WinMaybePossible, r#"
####
#@$#
# *#
####
"#);
    }

    #[test]
    fn is_winnable_with_box_trapped_win_impossible() {
        assert_level_winnable_state(WinnableState::WinImpossible, r#"
####
#@$#
#. #
####
"#);
        assert_level_winnable_state(WinnableState::WinImpossible, r#"
####
#@ #
#.$#
####
"#);
        assert_level_winnable_state(WinnableState::WinImpossible, r#"
####
#@.#
#$ #
####
"#);
        assert_level_winnable_state(WinnableState::WinImpossible, r#"
####
#$.#
# @#
####
"#);
        assert_level_winnable_state(WinnableState::WinImpossible, r#"
####
#@$####
#    .#
#######
"#);
        assert_level_winnable_state(WinnableState::WinImpossible, r#"
####
#@ ##
#. $#
#####
"#);
    }

    #[test]
    fn is_winnable_with_one_box_two_targets_win_impossible() {
        assert_level_winnable_state(WinnableState::WinImpossible, r#"
#####
#@$ #
#.. #
#####
"#);
    }

    #[test]
    fn is_winnable_with_one_trapped_box_two_free_two_targets_win_possible() {
        assert_level_winnable_state(WinnableState::WinMaybePossible, r#"
#######
#@$  $#
#..$  #
#######
"#);
    }
}