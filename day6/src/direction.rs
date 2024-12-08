#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Direction {
    Up = 0b00000001,
    Right = 0b00000010,
    Down = 0b00000100,
    Left = 0b00001000,
}

impl Direction {
    pub fn rotate_clockwise(&self) -> Direction {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    pub fn potential_blocker_count(byte: u8) -> u8 {
        let mut sum = 0;

        const UP_RIGHT: u8 = Direction::Up as u8 | Direction::Right as u8;
        const RIGHT_DOWN: u8 = Direction::Right as u8 | Direction::Down as u8;
        const DOWN_LEFT: u8 = Direction::Down as u8 | Direction::Left as u8;
        const LEFT_UP: u8 = Direction::Left as u8 | Direction::Up as u8;

        let has_up_and_right = ((byte & UP_RIGHT) == UP_RIGHT) as u8;
        sum += has_up_and_right;

        let has_right_and_down = ((byte & RIGHT_DOWN) == RIGHT_DOWN) as u8;
        sum += has_right_and_down;

        let has_down_and_left = ((byte & DOWN_LEFT) == DOWN_LEFT) as u8;
        sum += has_down_and_left;

        let has_left_and_up = ((byte & LEFT_UP) == LEFT_UP) as u8;
        sum += has_left_and_up;

        sum
    }
}
