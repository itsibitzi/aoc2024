use crate::point::Point;

#[derive(Copy, Clone)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    // Could have non-origin rects but I cba with the transforms
    pub fn from_origin(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x > self.x && point.y > self.y && point.x < self.width && point.y < self.height
    }
}
