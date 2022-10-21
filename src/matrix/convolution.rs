use std::collections::HashSet;

use macroquad::rand::gen_range;

use crate::CellType;

use super::{
    traits::{ConvolutionT, Matrix},
    vec_matrix::VecMatrix,
};

// the kernel should be a square matrix
#[derive(Debug, Clone)]
pub struct Convolution<T: Copy + Clone, const KW: usize> {
    width: usize,
    height: usize,
    base: Vec<Vec<T>>,
}

impl<Conv: Matrix<Output = u8>, Acc: Matrix<Output = u8>, const KW: usize>
    ConvolutionT<Conv, u8, Acc> for Convolution<u8, KW>
{
    fn convolution(
        &self,
        kernels: &[Conv],
        single_kernel: bool,
        cell_type_matrix: &VecMatrix<CellType>,
        acc_matrix: &mut Acc,
        indices: &HashSet<(usize, usize)>,
    ) {
        for (ixx, ixy) in indices.iter().copied() {
            let mut acc: u8 = 0;
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

impl<Conv: Matrix<Output = i8>, Acc: Matrix<Output = i8>, const KW: usize>
    ConvolutionT<Conv, i8, Acc> for Convolution<i8, KW>
{
    fn convolution(
        &self,
        kernels: &[Conv],
        single_kernel: bool,
        cell_type_matrix: &VecMatrix<CellType>,
        acc_matrix: &mut Acc,
        indices: &HashSet<(usize, usize)>,
    ) {
        for (ixx, ixy) in indices.iter().copied() {
            let mut acc: i8 = 0;
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
        base: &mut Vec<Vec<T>>,
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
impl<const KW: usize> Matrix for Convolution<u8, KW> {
    type Output = u8;
    fn new(width: usize, height: usize) -> Self {
        let base: Vec<Vec<u8>> = vec![vec![0; KW.pow(2)]; width * height];
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
                let random_value = gen_range(0, 2);
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
        let mut base = vec![vec![1; KW.pow(2)]; width * height];
        let wh = KW / 2;
        Self::set_base_at_index(&mut base, KW, KW, (wh, wh), 0);
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

// KW : width of the convolution kernel
impl<const KW: usize> Matrix for Convolution<i8, KW> {
    type Output = i8;
    fn new(width: usize, height: usize) -> Self {
        let base: Vec<Vec<i8>> = vec![vec![0; KW.pow(2)]; width * height];
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
                let random_value = gen_range::<i16>(0, 2) as i8;
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
                let random_value = gen_range(*range.start() as i32, *range.end() as i32) as i8;
                Self::set_base_at_index(&mut res.base, width, height, (ixx, ixy), random_value);
            }
        }
        res
    }

    fn new_std_conv_matrix(width: usize, height: usize) -> Self {
        let mut base = vec![vec![1; KW.pow(2)]; width * height];
        let wh = KW / 2;
        Self::set_base_at_index(&mut base, KW, KW, (wh, wh), 0);
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
            res.push_str("\n");
        }
        res
    }
}
