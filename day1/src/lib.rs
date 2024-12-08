#![feature(portable_simd, binary_heap_into_iter_sorted, test)]

use std::{
    collections::BinaryHeap,
    simd::{i32x4, i32x8, num::SimdInt},
};

const INPUT: &[u8] = include_bytes!("../input.txt");

fn parse_5_digit_number_i64(ascii_bytes: &[u8]) -> i64 {
    (ascii_bytes[0] - b'0') as i64 * 10000
        + (ascii_bytes[1] - b'0') as i64 * 1000
        + (ascii_bytes[2] - b'0') as i64 * 100
        + (ascii_bytes[3] - b'0') as i64 * 10
        + (ascii_bytes[4] - b'0') as i64
}

fn parse_5_digit_number_i32(ascii_bytes: &[u8]) -> i32 {
    (ascii_bytes[0] - b'0') as i32 * 10000
        + (ascii_bytes[1] - b'0') as i32 * 1000
        + (ascii_bytes[2] - b'0') as i32 * 100
        + (ascii_bytes[3] - b'0') as i32 * 10
        + (ascii_bytes[4] - b'0') as i32
}

fn parse_5_digit_number_i32x8_simd(ascii_bytes: &[u8]) -> i32 {
    let data = i32x8::from_array([
        0,
        0,
        0,
        ascii_bytes[0] as i32,
        ascii_bytes[1] as i32,
        ascii_bytes[2] as i32,
        ascii_bytes[3] as i32,
        ascii_bytes[4] as i32,
    ]);
    let ascii_offset = i32x8::from_array([
        0,
        0,
        0,
        b'0' as i32,
        b'0' as i32,
        b'0' as i32,
        b'0' as i32,
        b'0' as i32,
    ]);
    let multipliers = i32x8::from_array([0, 0, 0, 10000, 1000, 100, 10, 1]);
    let result = (data - ascii_offset) * multipliers;

    result.reduce_sum()
}

fn parse_5_digit_number_i32x4_simd(ascii_bytes: &[u8]) -> i32 {
    let data = i32x4::from_array([
        ascii_bytes[0] as i32,
        ascii_bytes[1] as i32,
        ascii_bytes[2] as i32,
        ascii_bytes[3] as i32,
    ]);
    let ascii_offset = i32x4::from_array([b'0' as i32, b'0' as i32, b'0' as i32, b'0' as i32]);

    let multipliers = i32x4::from_array([10000, 1000, 100, 10]);
    let result = (data - ascii_offset) * multipliers;
    let simd_sum = result.reduce_sum();

    simd_sum + ascii_bytes[4] as i32 - b'0' as i32
}

fn vec_i64(input: &[u8]) -> i64 {
    let (mut left_column, mut right_column) = input
        .chunks(14)
        .map(|line_bytes| {
            let left = parse_5_digit_number_i64(&line_bytes[0..5]);
            let right = parse_5_digit_number_i64(&line_bytes[8..13]);
            (left, right)
        })
        .collect::<(Vec<i64>, Vec<i64>)>();

    left_column.sort();
    right_column.sort();

    left_column
        .iter()
        .zip(right_column.iter())
        .fold(0_i64, |acc, (left, right)| acc + (left - right).abs())
}

fn vec_i32(input: &[u8]) -> i32 {
    let (mut left_column, mut right_column) = input
        .chunks(14)
        .map(|line_bytes| {
            let left = parse_5_digit_number_i32(&line_bytes[0..5]);
            let right = parse_5_digit_number_i32(&line_bytes[8..13]);
            (left, right)
        })
        .collect::<(Vec<i32>, Vec<i32>)>();

    left_column.sort();
    right_column.sort();

    left_column
        .iter()
        .zip(right_column.iter())
        .fold(0_i32, |acc, (left, right)| acc + (left - right).abs())
}

fn vec_i32_simd_parse(input: &[u8]) -> i32 {
    let (mut left_column, mut right_column) = input
        .chunks(14)
        .map(|line_bytes| {
            let left = parse_5_digit_number_i32x8_simd(&line_bytes[0..5]);
            let right = parse_5_digit_number_i32x8_simd(&line_bytes[8..13]);
            (left, right)
        })
        .collect::<(Vec<i32>, Vec<i32>)>();

    left_column.sort_unstable();
    right_column.sort_unstable();

    left_column
        .iter()
        .zip(right_column.iter())
        .fold(0_i32, |acc, (left, right)| acc + (left - right).abs())
}

fn vec_i32_unstable(input: &[u8]) -> i32 {
    let (mut left_column, mut right_column) = input
        .chunks(14)
        .map(|line_bytes| {
            let left = parse_5_digit_number_i32(&line_bytes[0..5]);
            let right = parse_5_digit_number_i32(&line_bytes[8..13]);
            (left, right)
        })
        .collect::<(Vec<i32>, Vec<i32>)>();

    left_column.sort_unstable();
    right_column.sort_unstable();

    left_column
        .iter()
        .zip(right_column.iter())
        .fold(0_i32, |acc, (left, right)| acc + (left - right).abs())
}

fn vec_i32_unstable_simd(input: &[u8]) -> i32 {
    let (mut left_column, mut right_column) = input
        .chunks(14)
        .map(|line_bytes| {
            let left = parse_5_digit_number_i32x4_simd(&line_bytes[0..5]);
            let right = parse_5_digit_number_i32x4_simd(&line_bytes[8..13]);
            (left, right)
        })
        .collect::<(Vec<i32>, Vec<i32>)>();

    left_column.sort_unstable();
    right_column.sort_unstable();

    const LANES: usize = 4;

    let mut sum = 0;
    let chunks = left_column.len() / LANES;

    for i in 0..chunks {
        let left = i32x4::from_slice(&left_column[i * LANES..i * LANES + LANES]);
        let right = i32x4::from_slice(&right_column[i * LANES..i * LANES + LANES]);
        let diff = left - right;
        let abs_diff = diff.abs();
        sum += abs_diff.reduce_sum();
    }

    // Handle remaining elements
    for i in (chunks * 4)..left_column.len() {
        sum += (left_column[i] - right_column[i]).abs();
    }

    sum
}

// Heap version, actually about 2x slower than going into a vector
// and then sorting the vector
fn heap_i32(input: &[u8]) -> i32 {
    let (left_column, right_column) = input
        .chunks(14)
        .map(|line_bytes| {
            let left = parse_5_digit_number_i32(&line_bytes[0..5]);
            let right = parse_5_digit_number_i32(&line_bytes[8..13]);
            (left, right)
        })
        .collect::<(BinaryHeap<i32>, BinaryHeap<i32>)>();

    left_column
        .into_iter_sorted()
        .zip(right_column.into_iter_sorted())
        .fold(0_i32, |acc, (left, right)| acc + (left - right).abs())
}

extern crate test;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    // running 9 tests
    // test tests::bench_output_heap_i32              ... bench:      26,341.04 ns/iter (+/- 2,374.00)
    // test tests::bench_output_vec_i32               ... bench:      13,452.26 ns/iter (+/- 377.84)
    // test tests::bench_output_vec_i32_simd_parse    ... bench:      11,901.90 ns/iter (+/- 369.81)
    // test tests::bench_output_vec_i32_unstable      ... bench:      11,767.19 ns/iter (+/- 283.09)
    // test tests::bench_output_vec_i32_unstable_simd ... bench:      11,757.39 ns/iter (+/- 335.71)
    // test tests::bench_output_vec_i64               ... bench:      14,277.75 ns/iter (+/- 584.71)
    // test tests::bench_parser                       ... bench:           0.31 ns/iter (+/- 0.01)
    // test tests::bench_parser_simd_4                ... bench:           0.31 ns/iter (+/- 0.02)
    // test tests::bench_parser_simd_8                ... bench:           0.31 ns/iter (+/- 0.01)
    //
    // test result: ok. 0 passed; 0 failed; 0 ignored; 9 measured; 0 filtered out; finished in 4.99s

    #[bench]
    fn bench_parser(b: &mut Bencher) {
        b.iter(|| parse_5_digit_number_i32(b"12345"));
    }

    #[bench]
    fn bench_parser_simd_8(b: &mut Bencher) {
        b.iter(|| parse_5_digit_number_i32x8_simd(b"12345"));
    }

    #[bench]
    fn bench_parser_simd_4(b: &mut Bencher) {
        b.iter(|| parse_5_digit_number_i32x4_simd(b"12345"));
    }

    #[bench]
    fn bench_output_vec_i64(b: &mut Bencher) {
        assert_eq!(936063, vec_i64(INPUT));
        b.iter(|| vec_i64(INPUT));
    }

    #[bench]
    fn bench_output_vec_i32(b: &mut Bencher) {
        assert_eq!(936063, vec_i32(INPUT));
        b.iter(|| vec_i32(INPUT));
    }
    #[bench]
    fn bench_output_vec_i32_simd_parse(b: &mut Bencher) {
        assert_eq!(936063, vec_i32_simd_parse(INPUT));
        b.iter(|| vec_i32_simd_parse(INPUT));
    }

    #[bench]
    fn bench_output_vec_i32_unstable(b: &mut Bencher) {
        assert_eq!(936063, vec_i32_unstable(INPUT));
        b.iter(|| vec_i32_unstable(INPUT));
    }

    #[bench]
    fn bench_output_vec_i32_unstable_simd(b: &mut Bencher) {
        assert_eq!(936063, vec_i32_unstable_simd(INPUT));
        b.iter(|| vec_i32_unstable_simd(INPUT));
    }

    #[bench]
    fn bench_output_heap_i32(b: &mut Bencher) {
        assert_eq!(936063, heap_i32(INPUT));
        b.iter(|| heap_i32(INPUT));
    }
}
