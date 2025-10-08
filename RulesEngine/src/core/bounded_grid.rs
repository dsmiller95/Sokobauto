use crate::core::Vec2;

pub struct BoundedGrid<T> {
    size: Vec2,
    cells: Vec<T>,
}

impl<T> BoundedGrid<T> {
    pub fn new(size: Vec2, default: T) -> Self
    where
        T: Clone,
    {
        let cells = vec![default; (size.i * size.j) as usize];
        BoundedGrid {
            size,
            cells,
        }
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }
    
    pub fn contains(&self, pos: &Vec2) -> bool {
        pos.inside(&self.size)
    }
}

impl<T> std::ops::Index<&Vec2> for BoundedGrid<T> {
    type Output = T;

    fn index(&self, index: &Vec2) -> &Self::Output {
        &self.cells[(index.i * self.size.j + index.j) as usize]
    }
}

impl<T> std::ops::IndexMut<&Vec2> for BoundedGrid<T> {
    fn index_mut(&mut self, index: &Vec2) -> &mut Self::Output {
        &mut self.cells[(index.i * self.size.j + index.j) as usize]
    }
}