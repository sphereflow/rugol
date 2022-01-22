use crate::CellType;

use super::vec_matrix::VecMatrix;

pub trait Matrix {
    type Output;
    fn new(width: usize, height: usize) -> Self;
    fn new_random(width: usize, height: usize) -> Self;
    fn new_std_conv_matrix(width: usize, height: usize) -> Self;
    fn index(&self, ix: (usize, usize)) -> Self::Output;
    fn set_at_index(&mut self, ix: (usize, usize), value: Self::Output);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub trait ConvolutionT<Conv: Matrix<Output = T>, T: Copy> {
    /// places accumulated values in self
    fn convolution(&mut self, kernels: &[Conv], cell_type_matrix: &VecMatrix<CellType>);
}
