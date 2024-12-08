extern crate test;

use std::simd::cmp::SimdPartialEq as _;

use fnv::FnvHashSet;

use crate::{direction::Direction, point::Point, SimdBlock};

// Extremely light weight set for the x -> y and y -> x mappings
#[derive(Default)]
pub struct SmallVec {
    length: usize,
    // The most obsticles we have in a row is 17 but
    // We don't get a smaller struct if we shrink-wrap to this
    // due to the word alignment. We could try a #[repr(packed)]
    // and see how that effects performance
    data: [u8; 24],
}

impl SmallVec {
    pub fn as_slice(&self) -> &[u8] {
        &self.data[0..self.length]
    }

    pub fn insert(&mut self, v: u8) {
        self.data[self.length] = v;
        self.length += 1;
    }

    pub fn next_smallest(&self, than: u8) -> Option<u8> {
        self.data[0..self.length]
            .iter()
            .filter(|d| **d < than)
            .max()
            .copied()
    }

    pub fn next_biggest(&self, than: u8) -> Option<u8> {
        self.data[0..self.length]
            .iter()
            .filter(|d| **d > than)
            .min()
            .copied()
    }
}

/// Dense representation for looking up values. Will be used
/// to look up coordinates from x to y and also from y to x.
///
/// Works on the principle that ~every row will have at least 1
/// coordinate entry, but each row only has a few entries. Sort
/// of dense -> sparse mapping.
///
/// We could do something like BTreeMap<u32, BTreeSet<u32>> but
/// the overheads are bad. This is *significantly* faster.
pub struct CoordinateLookupTable(Vec<SmallVec>);

impl CoordinateLookupTable {
    pub fn new() -> Self {
        let mut first_tier = Vec::with_capacity(200);
        // We always have a single dimension... this allows us to
        // start inserting values before we've discovered the number
        // of rows/columns
        first_tier.push(SmallVec::default());
        Self(first_tier)
    }

    pub fn with_size(size: u32) -> Self {
        let mut first_tier = Vec::with_capacity(size as usize);
        first_tier.resize_with(size as usize, SmallVec::default);
        Self(first_tier)
    }

    pub fn contains(&self, first: u32, second: u32) -> bool {
        self.0
            .get(first as usize)
            .is_some_and(|set| set.as_slice().contains(&(second as u8)))
    }

    pub fn get(&self, first: u32) -> Option<&SmallVec> {
        self.0.get(first as usize)
    }

    pub fn insert(&mut self, first: u32, second: u32) {
        let Some(second_tier) = self.0.get_mut(first as usize) else {
            // Silently ignore out of range... it makes some bound checking in part 2 simpler
            // to just ignore this but it would obviously be a deranged API
            return;
        };
        second_tier.insert(second as u8);
    }

    // Special purpose function for when we insert a new temporary block during part 2
    pub fn delete_most_recent_at(&mut self, first: u32) {
        let Some(second_tier) = self.0.get_mut(first as usize) else {
            return;
        };
        second_tier.length -= 1;
    }

    /// Resize the first tier
    pub fn resize(&mut self, first: u32) {
        self.0.resize_with(first as usize, SmallVec::default);
    }

    pub fn iter(&self) -> impl Iterator<Item = &SmallVec> {
        self.0.iter()
    }
}

#[derive(Clone)]
pub struct DirectedLineSegment {
    origin: Point,
    step: u32,
    direction: Direction,
}

impl DirectedLineSegment {
    pub fn new(start: Point, end: Point, direction: Direction) -> Self {
        match direction {
            Direction::Up => Self {
                origin: start,
                step: start.y - end.y,
                direction,
            },
            Direction::Right => Self {
                origin: start,
                step: end.x - start.x,
                direction,
            },
            Direction::Down => Self {
                origin: start,
                step: end.y - start.y,
                direction,
            },
            Direction::Left => Self {
                origin: start,
                step: start.x - end.x,
                direction,
            },
        }
    }
}

// For part 1 its hand to normalize the direction to make the calculations
// of intersections more straight forward

#[derive(Clone, PartialEq, Eq)]
pub enum NormalizedDirection {
    Vertical,
    Horizontal,
}

#[derive(Clone)]
pub struct NormalizedDirectedLineSegment {
    origin: Point,
    step: u32,
    direction: NormalizedDirection,
}

impl NormalizedDirectedLineSegment {
    pub fn new(start: Point, end: Point, direction: Direction) -> Self {
        match direction {
            Direction::Up => Self {
                origin: end,
                step: start.y - end.y,
                direction: NormalizedDirection::Vertical,
            },
            Direction::Right => Self {
                origin: start,
                step: end.x - start.x,
                direction: NormalizedDirection::Horizontal,
            },
            Direction::Down => Self {
                origin: start,
                step: end.y - start.y,
                direction: NormalizedDirection::Vertical,
            },
            Direction::Left => Self {
                origin: end,
                step: start.x - end.x,
                direction: NormalizedDirection::Horizontal,
            },
        }
    }

    pub fn step_overlap_count(&self, other: &NormalizedDirectedLineSegment) -> u32 {
        match (&self.direction, &other.direction) {
            // Both vertical
            (NormalizedDirection::Vertical, NormalizedDirection::Vertical) => {
                if self.origin.x != other.origin.x {
                    return 0;
                }

                let min_y1 = self.origin.y;
                let max_y1 = self.origin.y + self.step;
                let min_y2 = other.origin.y;
                let max_y2 = other.origin.y + other.step;

                if min_y1 > max_y2 || max_y1 < min_y2 {
                    0
                } else {
                    let overlap_start = min_y1.max(min_y2);
                    let overlap_end = max_y1.min(max_y2);
                    overlap_end - overlap_start
                }
            }

            // Both horizontal
            (NormalizedDirection::Horizontal, NormalizedDirection::Horizontal) => {
                if self.origin.y != other.origin.y {
                    return 0;
                }

                let min_x1 = self.origin.x;
                let max_x1 = self.origin.x + self.step;
                let min_x2 = other.origin.x;
                let max_x2 = other.origin.x + other.step;

                if min_x1 > max_x2 || max_x1 < min_x2 {
                    0
                } else {
                    let overlap_start = min_x1.max(min_x2);
                    let overlap_end = max_x1.min(max_x2);
                    overlap_end - overlap_start
                }
            }

            // Crossing lines
            _ => {
                let (vertical, horizontal) = if self.direction == NormalizedDirection::Vertical {
                    (self, other)
                } else {
                    (other, self)
                };

                let x_range_start = horizontal.origin.x;
                let x_range_end = horizontal.origin.x + horizontal.step;
                let y_range_start = vertical.origin.y;
                let y_range_end = vertical.origin.y + vertical.step;

                if vertical.origin.x >= x_range_start
                    && vertical.origin.x <= x_range_end
                    && horizontal.origin.y >= y_range_start
                    && horizontal.origin.y <= y_range_end
                {
                    1
                } else {
                    0
                }
            }
        }
    }
}

pub enum MovePosition {
    InGrid(Point),
    OffGrid(Point),
}

pub struct SparseGrid {
    pub width: u32,
    pub height: u32,
    pub start_point: Point,
    pub x_to_y: CoordinateLookupTable,
    pub y_to_x: CoordinateLookupTable,
}

impl SparseGrid {
    pub fn from_bytes(input: &[u8]) -> Self {
        let newline_target = SimdBlock::splat(b'\n');
        let start_target = SimdBlock::splat(b'^');
        let obsticles_target = SimdBlock::splat(b'#');

        let mut start_point: Option<Point> = None;
        let mut width: Option<u32> = None;
        let mut height: u32 = 0;

        // We build the y->x map first we can don't know
        // the number of columns before we discover the width
        // but we know we have a single row
        let mut y_to_x = CoordinateLookupTable::new();

        for (chunk_idx, c) in input.chunks(64).enumerate() {
            let chunk = SimdBlock::load_or_default(c);

            // Find start, if we haven't found it already
            if start_point.is_none() {
                let mask = chunk.simd_eq(start_target).to_bitmask();
                if mask != 0 {
                    let idx = mask.trailing_zeros();
                    let i = chunk_idx as u32 * 64 + idx;
                    // Unwrap or it doesn't matter cos our start point is n the first line
                    let width = width.unwrap_or(u32::MAX - 1);
                    start_point = Some(Point::new(i % (width + 1), i / (width + 1)));
                }
            }

            // Find newlines
            let mask = chunk.simd_eq(newline_target).to_bitmask();

            // We've found our first newline!
            if mask != 0 && width.is_none() {
                let idx_within_chunk = mask.trailing_zeros();
                let idx_within_row = chunk_idx as u32 * 64 + idx_within_chunk;
                width = Some(idx_within_row);
                y_to_x.resize(idx_within_row);
            }

            height += mask.count_ones();

            // Find obsticles
            let mask = chunk.simd_eq(obsticles_target).to_bitmask();

            if mask != 0 {
                let mut next_mask = mask;

                // Drain the mask from the low order bits to the higher order bits
                while next_mask != 0 {
                    let idx_within_chunk = next_mask.trailing_zeros();

                    let i = chunk_idx as u32 * 64 + idx_within_chunk;
                    let width = width.unwrap_or(u32::MAX - 1);

                    let p = Point::new(i % (width + 1), i / (width + 1));

                    y_to_x.insert(p.y, p.x);

                    next_mask ^= 1 << idx_within_chunk;
                }
            }
        }

        let width = width.unwrap();
        let mut x_to_y = CoordinateLookupTable::with_size(width);

        for (row, y) in y_to_x.iter().zip(0_u8..) {
            for x in row.as_slice() {
                x_to_y.insert(*x as u32, y as u32);
            }
        }

        Self {
            width,
            height,
            start_point: start_point.unwrap(),
            x_to_y,
            y_to_x,
        }
    }

    pub fn move_to_next_obsticle_above(&self, p: Point) -> MovePosition {
        let maybe_point = self
            .x_to_y
            .get(p.x)
            .and_then(|set| set.next_smallest(p.y as u8))
            .map(|y| Point::new(p.x, y as u32 + 1));

        match maybe_point {
            Some(point) => MovePosition::InGrid(point),
            None => MovePosition::OffGrid(Point::new(p.x, 0)),
        }
    }

    pub fn move_to_next_obsticle_below(&self, p: Point, height: u32) -> MovePosition {
        let maybe_point = self
            .x_to_y
            .get(p.x)
            .and_then(|set| set.next_biggest(p.y as u8))
            .map(|y| Point::new(p.x, y as u32 - 1));

        match maybe_point {
            Some(point) => MovePosition::InGrid(point),
            None => MovePosition::OffGrid(Point::new(p.x, height - 1)),
        }
    }

    pub fn move_to_next_obsticle_to_left(&self, p: Point) -> MovePosition {
        let maybe_point = self
            .y_to_x
            .get(p.y)
            .and_then(|set| set.next_smallest(p.x as u8))
            .map(|x| Point::new(x as u32 + 1, p.y));

        match maybe_point {
            Some(point) => MovePosition::InGrid(point),
            None => MovePosition::OffGrid(Point::new(0, p.y)),
        }
    }

    pub fn move_to_next_obsticle_to_right(&self, p: Point, width: u32) -> MovePosition {
        let maybe_point = self
            .y_to_x
            .get(p.y)
            .and_then(|set| set.next_biggest(p.x as u8))
            .map(|x| Point::new(x as u32 - 1, p.y));

        match maybe_point {
            Some(point) => MovePosition::InGrid(point),
            None => MovePosition::OffGrid(Point::new(width - 1, p.y)),
        }
    }

    pub fn part_1(&self) -> u32 {
        let mut direction = Direction::Up;

        // A directed line segement, which is like a cast ray but with a finite direction
        // and the number of steps to subtract from it. For part 1 we "normalize" this so
        // we are always dealing with a positive movement from left to right or from top to bottom
        let mut segments: Vec<NormalizedDirectedLineSegment> = Vec::with_capacity(1024);

        let mut position = self.start_point;
        loop {
            let next_position = match direction {
                Direction::Up => self.move_to_next_obsticle_above(position),
                Direction::Down => self.move_to_next_obsticle_below(position, self.height),
                Direction::Left => self.move_to_next_obsticle_to_left(position),
                Direction::Right => self.move_to_next_obsticle_to_right(position, self.width),
            };

            match next_position {
                MovePosition::InGrid(next_position) => {
                    let dls =
                        NormalizedDirectedLineSegment::new(position, next_position, direction);
                    segments.push(dls);

                    position = next_position;
                    direction = direction.rotate_clockwise();
                }
                MovePosition::OffGrid(edge_position) => {
                    let dls =
                        NormalizedDirectedLineSegment::new(position, edge_position, direction);
                    segments.push(dls);

                    break;
                }
            }
        }

        let mut total_visited = 0;

        for (i, segment) in segments.iter().enumerate() {
            // Origin plus every step along the way.
            total_visited += 1 + segment.step;

            for other in segments.iter().skip(i + 1) {
                total_visited -= segment.step_overlap_count(other);
            }
        }

        total_visited
    }

    pub fn part_2(&mut self) -> u32 {
        let mut segments: Vec<DirectedLineSegment> = Vec::with_capacity(1024);

        //
        // Generate list of paths
        //

        {
            let mut direction = Direction::Up;
            let mut position = self.start_point;

            loop {
                let next_position = match direction {
                    Direction::Up => self.move_to_next_obsticle_above(position),
                    Direction::Down => self.move_to_next_obsticle_below(position, self.height),
                    Direction::Left => self.move_to_next_obsticle_to_left(position),
                    Direction::Right => self.move_to_next_obsticle_to_right(position, self.width),
                };

                match next_position {
                    MovePosition::InGrid(next_position) => {
                        let dls = DirectedLineSegment::new(position, next_position, direction);
                        segments.push(dls);
                        position = next_position;
                        direction = direction.rotate_clockwise();
                    }
                    MovePosition::OffGrid(edge_position) => {
                        let dls = DirectedLineSegment::new(position, edge_position, direction);
                        segments.push(dls);
                        break;
                    }
                }
            }
        }

        //
        // Use list of paths to find potential places for obsticles
        //

        let mut total_potential_obsticles = 0;

        // Set of all placed candidates. Prevents retesting loops in a place we've
        // already put a block.
        let mut placed = FnvHashSet::<Point>::default();

        // Map to keep track of where we've visited during testing of a candidate
        // position - will be manually cleared after each iteration to remove
        // the need for reallocation
        let mut visited = FnvHashSet::<(Point, Direction)>::default();

        for segment in segments.iter() {
            for step in 0..=segment.step {
                let start_position = segment.origin.step_n(segment.direction, step);
                let block_position = start_position.step(segment.direction);

                if !placed.insert(block_position) {
                    continue;
                }

                // Let's see if a block already exists!
                if self.x_to_y.contains(block_position.x, block_position.y)
                // || block_position == self.start_point
                {
                    continue;
                }

                let mut position = start_position;
                let mut direction = segment.direction.rotate_clockwise();

                // Insert candidate block y->x and x->y mapping
                self.y_to_x.insert(block_position.y, block_position.x);
                self.x_to_y.insert(block_position.x, block_position.y);

                loop {
                    let next_position = match direction {
                        Direction::Up => self.move_to_next_obsticle_above(position),
                        Direction::Down => self.move_to_next_obsticle_below(position, self.height),
                        Direction::Left => self.move_to_next_obsticle_to_left(position),
                        Direction::Right => {
                            self.move_to_next_obsticle_to_right(position, self.width)
                        }
                    };

                    match next_position {
                        MovePosition::InGrid(next_position) => {
                            if !visited.insert((next_position, direction)) {
                                total_potential_obsticles += 1;
                                break;
                            }

                            position = next_position;
                            direction = direction.rotate_clockwise();
                        }
                        MovePosition::OffGrid(_) => {
                            break;
                        }
                    }
                }

                // Remove candidate block
                self.y_to_x.delete_most_recent_at(block_position.y);
                self.x_to_y.delete_most_recent_at(block_position.x);
                visited.clear();
            }
        }

        total_potential_obsticles
    }
}

const INPUT: &[u8] = include_bytes!("input.txt");
const EXAMPLE: &[u8] = include_bytes!("example.txt");

#[bench]
fn bench_sparse_prelude(b: &mut test::Bencher) {
    b.iter(|| {
        let grid = SparseGrid::from_bytes(INPUT);

        assert_eq!(grid.width, 130);
        assert_eq!(grid.height, 130);
        assert_eq!(grid.start_point, Point::new(41, 73));
    });
}

#[bench]
fn bench_sparse_part_1_complete(b: &mut test::Bencher) {
    b.iter(|| {
        let grid = SparseGrid::from_bytes(INPUT);

        assert_eq!(grid.width, 130);
        assert_eq!(grid.height, 130);
        assert_eq!(grid.start_point, Point::new(41, 73));

        let count = grid.part_1();
        assert_eq!(count, 4939);
    });
}

#[bench]
fn bench_sparse_part_2_complete(b: &mut test::Bencher) {
    b.iter(|| {
        let mut grid = SparseGrid::from_bytes(INPUT);

        assert_eq!(grid.width, 130);
        assert_eq!(grid.height, 130);
        assert_eq!(grid.start_point, Point::new(41, 73));

        let count = grid.part_2();
        assert_eq!(count, 1434);
    });
}
