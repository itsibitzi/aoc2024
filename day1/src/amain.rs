use std::time::Instant;

const INPUT: &[u8] = include_bytes!("../input.txt");

fn parse_5_digit_number_i32(ascii_bytes: &[u8]) -> i32 {
    (ascii_bytes[0] - b'0') as i32 * 10000
        + (ascii_bytes[1] - b'0') as i32 * 1000
        + (ascii_bytes[2] - b'0') as i32 * 100
        + (ascii_bytes[3] - b'0') as i32 * 10
        + (ascii_bytes[4] - b'0') as i32
}

fn main() {
    let start = Instant::now();

    let (mut left_column, mut right_column) = INPUT
        .chunks(14)
        .map(|line_bytes| {
            let left = parse_5_digit_number_i32(&line_bytes[0..5]);
            let right = parse_5_digit_number_i32(&line_bytes[8..13]);
            (left, right)
        })
        .collect::<(Vec<i32>, Vec<i32>)>();

    left_column.sort_unstable();
    right_column.sort_unstable();

    let total = left_column
        .iter()
        .zip(right_column.iter())
        .fold(0_i32, |acc, (left, right)| acc + (left - right).abs());

    let end = Instant::now();
    println!("Part 1: {} in {}ns", total, (end - start).as_nanos());
}
