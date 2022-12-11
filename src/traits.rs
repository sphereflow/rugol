use crate::{index_set::IndexSet, CellType};
use matrices::traits::Matrix;
use matrices::vec_matrix::VecMatrix;

pub trait ConvolutionT<Conv: Matrix<T>, T: Copy, Acc: Matrix<T>> {
    /// places accumulated values in self
    fn convolution(
        &self,
        kernels: &[Conv],
        single_kernel: bool,
        cell_type_matrix: &VecMatrix<CellType>,
        acc_matrix: &mut Acc,
        indices: &IndexSet,
    );
}
