use super::{
    traits::{ConvolutionT, Matrix},
    vec_matrix::VecMatrix,
};
use crate::{index_set::IndexSet, CellType};
use num_traits::{AsPrimitive, One, Zero};
use quad_rand::{gen_range, RandomRange};
use std::ops::{AddAssign, Mul};

// the kernel should be a square matrix
#[derive(Debug, Clone)]
pub struct Convolution<T: Copy + Clone, const KW: usize> {
    width: usize,
    height: usize,
    base: Vec<Vec<T>>,
}

impl<
        T: Copy + Zero + Mul + AddAssign + Mul<Output = T>,
        Conv: Matrix<Output = T>,
        Acc: Matrix<Output = T>,
        const KW: usize,
    > ConvolutionT<Conv, T, Acc> for Convolution<T, KW>
{
    fn convolution(
        &self,
        kernels: &[Conv],
        single_kernel: bool,
        cell_type_matrix: &VecMatrix<CellType>,
        acc_matrix: &mut Acc,
        indices: &IndexSet,
    ) {
        for (ixx, ixy) in indices.iter() {
            let mut acc: T = Zero::zero();
            let kernel_ix = cell_type_matrix.index((ixx, ixy)).as_index();
            let slice = &self.base[ixx + self.width * ixy];
            for kixx in 0..KW {
                for kixy in 0..KW {
                    if single_kernel {
                        acc += slice[kixy * KW + kixx] * kernels[0].index((kixx, kixy));
                    } else {
                        acc += slice[kixy * KW + kixx] * kernels[kernel_ix].index((kixx, kixy));
                    }
                }
            }
            acc_matrix.set_at_index((ixx, ixy), acc);
        }
    }
}

impl<T: Copy, const KW: usize> Convolution<T, KW> {
    fn set_base_at_index(
        base: &mut [Vec<T>],
        base_width: usize,
        base_height: usize,
        (ixx, ixy): (usize, usize),
        val: T,
    ) {
        let wh = KW / 2;
        for y_offset in 0..KW {
            if ixy + wh < y_offset {
                continue;
            }
            if ixy + wh - y_offset >= base_height {
                continue;
            }
            for x_offset in 0..KW {
                if ixx + wh < x_offset {
                    continue;
                }
                if ixx + wh - x_offset >= base_width {
                    continue;
                }
                base[(ixx + wh - x_offset) + (ixy + wh - y_offset) * base_width]
                    [x_offset + y_offset * KW] = val;
            }
        }
    }
}

// KW : width of the convolution kernel
impl<T: Copy + Zero + One + RandomRange + 'static, const KW: usize> Matrix for Convolution<T, KW>
where
    u8: AsPrimitive<T>,
{
    type Output = T;
    fn new(width: usize, height: usize) -> Self {
        let base: Vec<Vec<T>> = vec![vec![Zero::zero(); KW.pow(2)]; width * height];
        Convolution {
            width,
            height,
            base,
        }
    }

    fn new_random(width: usize, height: usize) -> Self {
        let mut res = Self::new(width, height);
        for ixx in 0..width {
            for ixy in 0..height {
                let random_value = gen_range(Zero::zero(), 2_u8.as_());
                Self::set_base_at_index(&mut res.base, width, height, (ixx, ixy), random_value);
            }
        }
        res
    }

    fn new_random_range(
        width: usize,
        height: usize,
        range: std::ops::RangeInclusive<Self::Output>,
    ) -> Self {
        let mut res = Self::new(width, height);
        for ixx in 0..width {
            for ixy in 0..height {
                let random_value = gen_range(*range.start(), *range.end());
                Self::set_base_at_index(&mut res.base, width, height, (ixx, ixy), random_value);
            }
        }
        res
    }

    fn new_std_conv_matrix(width: usize, height: usize) -> Self {
        let mut base = vec![vec![One::one(); KW.pow(2)]; width * height];
        let wh = KW / 2;
        Self::set_base_at_index(&mut base, KW, KW, (wh, wh), Zero::zero());
        Convolution {
            width,
            height,
            base,
        }
    }

    fn index(&self, (ixx, ixy): (usize, usize)) -> Self::Output {
        let wh = KW / 2;
        self.base[ixy * self.width + ixx][wh * KW + wh]
    }

    fn set_at_index(&mut self, ix: (usize, usize), value: Self::Output) {
        Self::set_base_at_index(&mut self.base, self.width, self.height, ix, value);
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<T: Copy + ToString, const KW: usize> Convolution<T, KW> {
    pub fn display_element(&self, (ixx, ixy): (usize, usize)) -> String {
        let mut res = String::new();
        let conv = &self.base[ixy * self.width + ixx];
        for y in 0..KW {
            for x in 0..KW {
                res.push_str(&conv[y * KW + x].to_string());
                res.push_str(", ");
            }
            res.push('\n');
        }
        res
    }
}
