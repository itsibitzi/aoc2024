use crate::direction::Direction;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn step(&self, dir: Direction) -> Point {
        match dir {
            Direction::Up => Point::new(self.x, self.y - 1),
            Direction::Down => Point::new(self.x, self.y + 1),
            Direction::Left => Point::new(self.x - 1, self.y),
            Direction::Right => Point::new(self.x + 1, self.y),
        }
    }

    pub fn step_n(&self, dir: Direction, n: u32) -> Point {
        match dir {
            Direction::Up => Point::new(self.x, self.y - n),
            Direction::Down => Point::new(self.x, self.y + n),
            Direction::Left => Point::new(self.x - n, self.y),
            Direction::Right => Point::new(self.x + n, self.y),
        }
    }
}
