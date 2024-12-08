#![feature(test, portable_simd)]

mod direction;
mod point;
mod rectangle;
mod simd_grid;
mod sparse;

use std::{
    collections::{HashMap, HashSet},
    simd::{cmp::SimdPartialEq, Simd},
};

use direction::Direction;
use point::Point;
use rectangle::Rectangle;

pub type SimdBlock = Simd<u8, 64>;

pub fn find_width(input: &[u8]) -> u32 {
    input
        .iter()
        .zip(0_u32..)
        .find(|(&c, _)| c == b'\n')
        .map(|(_, i)| i)
        .unwrap()
}

pub fn find_height(input: &[u8]) -> u32 {
    input.iter().filter(|c| **c == b'\n').count() as u32
}

pub fn find_extents(input: &[u8]) -> Rectangle {
    let mut x = 0;
    let mut y = 0;

    for (c, i) in input.iter().zip(0_u32..) {
        if *c == b'\n' {
            y += 1;
            if x == 0 {
                x = i;
            }
        }
    }

    Rectangle::from_origin(x, y)
}

pub fn find_start(input: &[u8], width: u32) -> Option<Point> {
    input
        .iter()
        .position(|&c| c == b'^')
        .map(|i| Point::new(i as u32 % (width + 1), i as u32 / (width + 1)))
}

pub fn find_start_simd(input: &[u8], width: u32) -> Option<Point> {
    let target = SimdBlock::splat(b'^');

    for (chunk_idx, c) in input.chunks(64).enumerate() {
        let chunk = SimdBlock::load_or_default(c);
        let mask = chunk.simd_eq(target).to_bitmask();

        if mask != 0 {
            let idx = mask.trailing_zeros();
            let i = chunk_idx as u32 * 64 + idx;
            return Some(Point::new(i % (width + 1), i / (width + 1)));
        }
    }

    None
}

pub fn part_1(input: &[&[u8]], mut position: Point, extents: Rectangle) -> usize {
    let mut direction = Direction::Up;

    let mut visited: HashSet<Point> = HashSet::new();

    loop {
        visited.insert(position);

        let next_pos = position.step(direction);

        if extents.contains(next_pos) {
            let next_tile = input[next_pos.y as usize][next_pos.x as usize];

            if next_tile == b'#' {
                direction = direction.rotate_clockwise();
            } else {
                position = next_pos;
            }
        } else {
            break;
        }
    }

    visited.len()
}

pub fn part_2(input: &[&[u8]], mut position: Point, extents: Rectangle) -> usize {
    let mut direction = Direction::Up;

    let mut visited: HashMap<Point, u8> = HashMap::new();

    loop {
        let next_pos = position.step(direction);

        let direction_flags = visited
            .get(&next_pos)
            .map(|&existing_direction| existing_direction | direction as u8)
            .unwrap_or(direction as u8);

        visited.insert(next_pos, direction_flags);

        if extents.contains(next_pos) {
            let next_tile = input[next_pos.y as usize][next_pos.x as usize];

            if next_tile == b'#' {
                direction = direction.rotate_clockwise();
            } else {
                position = next_pos;
            }
        } else {
            break;
        }
    }

    visited
        .values()
        .map(|direction_flags| Direction::potential_blocker_count(*direction_flags) as usize)
        .inspect(|v| println!("{}", v))
        .sum()
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;

    const INPUT: &[u8] = include_bytes!("input.txt");

    #[test]
    fn test_find_extents() {
        let w = find_width(INPUT);
        let h = find_height(INPUT);

        let Rectangle { width, height, .. } = find_extents(INPUT);

        assert_eq!(w, 130, "width");
        assert_eq!(h, 130, "height");
        assert_eq!(w, width, "extents width");
        assert_eq!(w, height, "extents height");
    }

    #[test]
    fn test_find_start() {
        let p = find_start(INPUT, find_width(INPUT)).expect("find the start");
        assert_eq!(p, Point::new(41, 73));
    }

    #[test]
    fn test_find_start_simd() {
        let p = find_start_simd(INPUT, find_width(INPUT)).expect("find the start");
        assert_eq!(p, Point::new(41, 73));
    }

    #[test]
    fn test_part_1() {
        let extents = find_extents(INPUT);

        let position = find_start_simd(INPUT, find_width(INPUT)).expect("find the start");

        let grid = INPUT.split(|c| *c == b'\n').collect::<Vec<_>>();

        let count = part_1(&grid, position, extents);

        assert_eq!(count, 4939);
    }

    // #[test]
    // fn test_part_2() {
    //     let extents = find_extents(INPUT);

    //     let position = find_start_simd(INPUT, find_width(INPUT)).expect("find the start");

    //     let grid = INPUT.split(|c| *c == b'\n').collect::<Vec<_>>();

    //     let count = part_2(&grid, position, extents);

    //     assert_eq!(count, 4939);
    // }

    // #[test]
    // fn test_part_2() {
    //     let extents = find_extents(INPUT);
    //     let position = find_start_simd(INPUT, find_width(INPUT)).expect("find the start");
    //     let grid = INPUT.split(|c| *c == b'\n').collect::<Vec<_>>();
    //     let count = part_2(&grid, position, extents.x, extents.y);

    //     assert_eq!(count, 0);
    // }

    #[bench]
    fn bench_find_start(b: &mut Bencher) {
        b.iter(|| find_start(INPUT, 130).expect("find the start"));
    }

    #[bench]
    fn bench_find_start_simd(b: &mut Bencher) {
        b.iter(|| find_start_simd(INPUT, 130).expect("find the start"));
    }

    #[bench]
    fn bench_prelude(b: &mut Bencher) {
        b.iter(|| {
            let extents = find_extents(INPUT);

            let position = find_start_simd(INPUT, extents.width).expect("find the start");

            let grid = INPUT.split(|c| *c == b'\n').collect::<Vec<_>>();

            assert_eq!(position, Point::new(41, 73));
            assert_eq!(grid[0].len(), 130);
            assert_eq!(grid[129].len(), 130);
        });
    }

    #[bench]
    fn bench_part_1_complete(b: &mut Bencher) {
        b.iter(|| {
            let extents = find_extents(INPUT);

            let position = find_start_simd(INPUT, extents.width).expect("find the start");

            let grid = INPUT.split(|c| *c == b'\n').collect::<Vec<_>>();

            let count = part_1(&grid, position, extents);

            assert_eq!(count, 4939);
        });
    }
}
