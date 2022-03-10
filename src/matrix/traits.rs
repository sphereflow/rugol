use super::vec_matrix::VecMatrix;
use crate::CellType;
use std::ops::RangeInclusive;

pub trait Matrix {
    type Output: Copy;
    fn new(width: usize, height: usize) -> Self;
    fn new_random(width: usize, height: usize) -> Self;
    fn new_random_range(width: usize, height: usize, range: RangeInclusive<Self::Output>) -> Self;
    fn new_std_conv_matrix(width: usize, height: usize) -> Self;
    fn index(&self, ix: (usize, usize)) -> Self::Output;
    fn set_at_index(&mut self, ix: (usize, usize), value: Self::Output);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn clear(&mut self, val: Self::Output) {
        for y in 0..self.height() {
            for x in 0..self.width() {
                self.set_at_index((x, y), val);
            }
        }
    }

    /// form a ring around the middle element in a convolution kernel
    /// e. g. :
    /// 0 0 0 0 0
    /// 0 1 1 1 0
    /// 0 1 0 1 0
    /// 0 1 1 1 0
    /// 0 0 0 0 0
    /// make sure the matrix is a square
    fn donut(&mut self, range: RangeInclusive<usize>, val: Self::Output) {
        let wh = self.width();
        let whhalf = wh / 2;
        for ixx in 0..wh {
            let x = if ixx > whhalf { wh - ixx - 1 } else { ixx };
            for ixy in 0..wh {
                let y = if ixy > whhalf { wh - ixy - 1 } else { ixy };
                if (range.contains(&x) || range.contains(&y))
                    && (x >= *range.start() && y >= *range.start())
                {
                    self.set_at_index((ixx, ixy), val);
                }
            }
        }
    }

    fn set_at_index_sym(&mut self, sym: Symmetry, (ixx, ixy): (usize, usize), val: Self::Output) {
        // zero based indexing width and height values
        let w = self.width() - 1;
        let h = self.height() - 1;
        let wh: i64 = w as i64 / 2;
        let hh: i64 = h as i64 / 2;
        match sym {
            Symmetry::X => {
                self.set_at_index((ixx, ixy), val);
                self.set_at_index((w - ixx, ixy), val);
            }
            Symmetry::Y => {
                self.set_at_index((ixx, ixy), val);
                self.set_at_index((ixx, h - ixy), val);
            }
            Symmetry::XY => {
                self.set_at_index((ixx, ixy), val);
                self.set_at_index((w - ixx, ixy), val);
                self.set_at_index((ixx, h - ixy), val);
                self.set_at_index((w - ixx, h - ixy), val);
            }
            Symmetry::ROT90 => {
                self.set_at_index((ixx, ixy), val);
                if ixx > h || ixy > w {
                    return;
                }
                let relx: i64 = wh - ixx as i64;
                let rely: i64 = hh - ixy as i64;
                // 90 degrees
                let nx = wh + rely;
                if (0..=w as i64).contains(&nx) && (0..=h).contains(&ixx) {
                    self.set_at_index((nx as usize, ixx), val);
                }
                // 180 degrees
                self.set_at_index((w - ixx, h - ixy), val);
                // 270 degrees
                let ny = hh + relx;
                if (0..=w).contains(&ixy) && (0..=h as i64).contains(&ny) {
                    self.set_at_index((ixy, ny as usize), val);
                }
            }
            Symmetry::ROT180 => {
                self.set_at_index((ixx, ixy), val);
                self.set_at_index((w - ixx, h - ixy), val);
            }
        }
    }
}

pub trait ConvolutionT<Conv: Matrix<Output = T>, T: Copy> {
    /// places accumulated values in self
    fn convolution(&mut self, kernels: &[Conv], single_kernel: bool, cell_type_matrix: &VecMatrix<CellType>);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symmetry {
    X,
    Y,
    XY,
    ROT90,
    ROT180,
}
