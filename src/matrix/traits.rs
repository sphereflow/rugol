use crate::rules::Rules;

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

pub trait ConvolutionT<T: Copy> {
    fn convolution(&mut self, kernel: &[T], rules: &Rules<T>);
}
