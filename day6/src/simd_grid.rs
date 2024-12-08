// use std::{cmp::min, env::current_dir, simd::Simd};

// use crate::{point::Point, rectangle::Rectangle, SimdBlock};

// pub struct SimdGrid {
//     // Ugh, we should have the top, left coordinates in world space
//     // since the provided extents could have a non-0,0 origin.
//     //
//     // But I can't be bothered.
//     rows: Vec<SimdDimension>,
//     columns: Vec<SimdDimension>,
// }

// struct SimdDimension(Vec<SimdBlock>);

// impl SimdDimension {
//     pub fn empty() -> Self {
//         Self(vec![])
//     }
// }

// impl SimdGrid {
//     pub fn from_bytes(input: &[u8], extents: Rectangle) -> Self {
//         let width = extents.width as usize;
//         let height = extents.height as usize;

//         let mut rows = Vec::with_capacity(height);
//         let mut columns = Vec::with_capacity(width);

//         let simd_blocks_per_row = width / 64 + 1;

//         // A vector containing the current dimension we're building
//         // will be reused for rows and columns
//         let mut current_dim = Vec::with_capacity(simd_blocks_per_row);

//         for row_bytes in input.chunks(width + 1) {
//             for block in 0..simd_blocks_per_row {
//                 let start = block * 64;
//                 let end = min(start + 64, row_bytes.len());
//                 current_dim.push(SimdBlock::load_or_default(&row_bytes[start..end]))
//             }
//             rows.push(SimdDimension(current_dim.clone()));
//             current_dim.clear();
//         }

//         // We have to fill this block one byte at a time striding over `width` bytes each time
//         let mut current_block_idx = 0;
//         let mut current_block = [0; 64];

//         for column in 0..width {
//             for column_cell in 0..height {
//                 let cell = input[column + column_cell * (width + 1)]; /* skip over newlines */
//                 current_block[current_block_idx] = cell;
//                 current_block_idx += 1;

//                 if current_block_idx == 64 {
//                     current_dim.push(SimdBlock::load_or_default(&current_block[0..64]));
//                     current_block_idx = 0;
//                 }
//             }

//             // Push any remaining cells into the dim
//             if current_block_idx > 0 {
//                 current_dim.push(SimdBlock::load_or_default(
//                     &current_block[0..current_block_idx],
//                 ))
//             }

//             columns.push(SimdDimension(current_dim.clone()));
//             current_dim.clear();
//         }

//         Self { rows, columns }
//     }

//     pub fn to_vec_by_row(&self) -> Vec<u8> {
//         let mut bytes = Vec::new();

//         for SimdDimension(simd_blocks) in &self.rows {
//             for block in simd_blocks {
//                 for byte in block.as_array() {
//                     if *byte == 0 {
//                         break;
//                     }
//                     bytes.push(*byte);
//                 }
//             }
//         }

//         bytes
//     }

//     pub fn byte_at_point_by_row(&self, point: Point) -> u8 {
//         // Would normally have to convert into grid space here
//         // since point is world space.
//         let row = &self.rows[point.y as usize];
//         let dim_block = point.x / 64;

//         let block = row.0[dim_block as usize];

//         let block_idx = point.x % 64;

//         block.as_array()[block_idx as usize]
//     }

//     pub fn byte_at_point_by_column(&self, point: Point) -> u8 {
//         // Would normally have to convert into grid space here
//         // since point is world space.
//         let row = &self.columns[point.x as usize];
//         let dim_block = point.y / 64;

//         let block = row.0[dim_block as usize];

//         let block_idx = point.y % 64;

//         block.as_array()[block_idx as usize]
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::{point::Point, rectangle::Rectangle};

//     use super::SimdGrid;

//     const INPUT: &[u8] = include_bytes!("input.txt");

//     #[test]
//     fn equal() {
//         let grid = SimdGrid::from_bytes(INPUT, Rectangle::from_origin(130, 130));
//         assert_eq!(grid.to_vec_by_row().len(), INPUT.len());
//     }

//     #[test]
//     fn rows_equals_columns() {
//         let grid = SimdGrid::from_bytes(INPUT, Rectangle::from_origin(130, 130));

//         for y in 0..130 {
//             for x in 0..130 {
//                 let p = Point::new(x, y);
//                 let r = grid.byte_at_point_by_row(p);
//                 let c = grid.byte_at_point_by_column(p);
//                 let diff = if r != c { "DIFFERENT" } else { "" };

//                 println!("@ x: {:3} y: {:3}, r: {}, c: {} {}", x, y, r, c, diff);
//                 // assert_eq!(r, c);
//             }
//         }
//     }
// }
