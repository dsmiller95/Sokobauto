use std::hash::{Hash, Hasher};
use std::hint::black_box;
use bevy::prelude::IVec2;
use crate::core::DEDUPLICATE_BOXES;
use crate::core::models::Vec2;

#[derive(Clone, Debug)]
pub struct GameStateEnvironment {
    boxes: [Vec2; BOX_COUNT],
}

const BOX_COUNT: usize = 8;
const EMPTY_BOX: Vec2 = Vec2 { i: i8::MAX, j: i8::MAX };

impl GameStateEnvironment {
    pub fn new(boxes: Vec<IVec2>) -> GameStateEnvironment {
        // TODO: use something like SmallVec to get best of both worlds?
        assert!(boxes.len() <= 15, "boxes length should be less than 15");
        let mut boxes_fixed = [EMPTY_BOX; BOX_COUNT];
        for (i, &b) in boxes.iter().enumerate() {
            boxes_fixed[i] = b.into();
        }
        let mut result = GameStateEnvironment {
            boxes: boxes_fixed,
        };
        result.complete_moves();
        result
    }

    pub fn new_empty() -> GameStateEnvironment {
        GameStateEnvironment {
            boxes: [EMPTY_BOX; BOX_COUNT],
        }
    }

    pub fn iter_boxes(&self) -> impl Iterator<Item=&Vec2> {
        self.boxes.iter().take_while(|&&b| b != EMPTY_BOX)
    }

    pub fn has_box_at(&self, position: &Vec2) -> bool {
        assert_ne!(position, &EMPTY_BOX, "position cannot be empty box special value");
        self.iter_boxes().any(|b| b == position)
    }

    pub fn index_of_box_at(&self, position: &Vec2) -> Option<usize> {
        assert_ne!(position, &EMPTY_BOX, "position cannot be empty box special value");
        self.iter_boxes().position(|b| b == position)
    }

    pub fn set_box(&mut self, box_index: usize, position: &Vec2) {
        assert_ne!(position, &EMPTY_BOX, "position cannot be empty box special value");
        self.boxes[box_index] = *position;
    }

    pub fn complete_moves(&mut self) {
        if black_box(DEDUPLICATE_BOXES) {
            self.boxes.sort_unstable()
        }
    }
}

impl Hash for GameStateEnvironment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter_boxes().for_each(|b| {
            b.hash(state)
        });
    }
}

impl Eq for GameStateEnvironment{}
impl PartialEq<Self> for GameStateEnvironment {
    fn eq(&self, other: &Self) -> bool {
        self.iter_boxes().eq(other.iter_boxes())
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_has_box_finds_box() {
        let mut boxes = vec![];
        boxes.push(IVec2 { x: 0, y: 0 });
        boxes.push(IVec2 { x: 9, y: 10 });
        boxes.push(IVec2 { x: -10, y: -9 });

        let environment = GameStateEnvironment::new(boxes);

        assert!(environment.has_box_at(&Vec2 { i: 0, j: 0 }));
        assert!(environment.has_box_at(&Vec2 { i: 10, j: 9 }));
        assert!(environment.has_box_at(&Vec2 { i: -9, j: -10 }));
        assert!(!environment.has_box_at(&Vec2 { i: 1, j: 1 }));
        assert!(!environment.has_box_at(&Vec2 { i: -1, j: -1 }));
    }
}