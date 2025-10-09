use bevy::math::Vec3;

pub fn checker_board(v: Vec3, checker_size: f32, flip: bool) -> Vec3 {
    let pos_in_checker = v % checker_size;
    let checker_offset = (v / checker_size).floor() * checker_size;
    checker_cell(pos_in_checker / checker_size, flip) * checker_size + checker_offset
}

fn checker_cell(v: Vec3, flip: bool) -> Vec3 {
    let is_odd_col = (v.x % 1.0) > 0.5;
    let is_odd_row = (v.y % 1.0) > 0.5;
    let is_odd_depth = (v.z % 1.0) > 0.5;
    let mut is_in_checker = false;
    is_in_checker ^= is_odd_col;
    is_in_checker ^= is_odd_row;
    is_in_checker ^= is_odd_depth;
    is_in_checker ^= flip;
    if is_in_checker {
        (v + Vec3::splat(0.5)) % Vec3::splat(1.0)
    } else {
        v
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_checker_cell() {
        assert_eq!(checker_cell(Vec3::new(0.25, 0.25, 0.25), false), Vec3::new(0.25, 0.25, 0.25));
        assert_eq!(checker_cell(Vec3::new(0.75, 0.25, 0.25), false), Vec3::new(0.25, 0.75, 0.75));
        assert_eq!(checker_cell(Vec3::new(0.25, 0.75, 0.25), false), Vec3::new(0.75, 0.25, 0.75));
        assert_eq!(checker_cell(Vec3::new(0.75, 0.75, 0.25), false), Vec3::new(0.75, 0.75, 0.25));
        assert_eq!(checker_cell(Vec3::new(0.25, 0.25, 0.75), false), Vec3::new(0.75, 0.75, 0.25));
        assert_eq!(checker_cell(Vec3::new(0.75, 0.25, 0.75), false), Vec3::new(0.75, 0.25, 0.75));
        assert_eq!(checker_cell(Vec3::new(0.25, 0.75, 0.75), false), Vec3::new(0.25, 0.75, 0.75));
        assert_eq!(checker_cell(Vec3::new(0.75, 0.75, 0.75), false), Vec3::new(0.25, 0.25, 0.25));
    }
    
    #[test]
    fn test_checker_cell_flipped() {
        assert_eq!(checker_cell(Vec3::new(0.25, 0.25, 0.25), true), Vec3::new(0.75, 0.75, 0.75));
        assert_eq!(checker_cell(Vec3::new(0.75, 0.25, 0.25), true), Vec3::new(0.75, 0.25, 0.25));
        assert_eq!(checker_cell(Vec3::new(0.25, 0.75, 0.25), true), Vec3::new(0.25, 0.75, 0.25));
        assert_eq!(checker_cell(Vec3::new(0.75, 0.75, 0.25), true), Vec3::new(0.25, 0.25, 0.75));
        assert_eq!(checker_cell(Vec3::new(0.25, 0.25, 0.75), true), Vec3::new(0.25, 0.25, 0.75));
        assert_eq!(checker_cell(Vec3::new(0.75, 0.25, 0.75), true), Vec3::new(0.25, 0.75, 0.25));
        assert_eq!(checker_cell(Vec3::new(0.25, 0.75, 0.75), true), Vec3::new(0.75, 0.25, 0.25));
        assert_eq!(checker_cell(Vec3::new(0.75, 0.75, 0.75), true), Vec3::new(0.75, 0.75, 0.75));
    }
    
    #[test]
    fn test_checker_board() {
        assert_eq!(checker_board(Vec3::new(1.0, 1.0, 1.0), 4.0, false), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(checker_board(Vec3::new(3.0, 1.0, 1.0), 4.0, false), Vec3::new(1.0, 3.0, 3.0));

        assert_eq!(checker_board(Vec3::new(5.0, 5.0, 5.0), 4.0, false), Vec3::new(5.0, 5.0, 5.0));
        assert_eq!(checker_board(Vec3::new(5.0, 7.0, 5.0), 4.0, false), Vec3::new(7.0, 5.0, 7.0));
    }

    #[test]
    fn test_checker_board_flipped() {
        assert_eq!(checker_board(Vec3::new(1.0, 1.0, 1.0), 4.0, true), Vec3::new(3.0, 3.0, 3.0));
        assert_eq!(checker_board(Vec3::new(3.0, 1.0, 1.0), 4.0, true), Vec3::new(3.0, 1.0, 1.0));

        assert_eq!(checker_board(Vec3::new(5.0, 5.0, 5.0), 4.0, true), Vec3::new(7.0, 7.0, 7.0));
        assert_eq!(checker_board(Vec3::new(5.0, 7.0, 5.0), 4.0, true), Vec3::new(5.0, 7.0, 5.0));
    }
}