use super::*;

// the kernel should be a square matrix
#[derive(Debug, Clone)]
pub struct Convolution<T: Copy + Clone, const KW: usize> {
    width: usize,
    height: usize,
    base: Vec<Vec<T>>,
}

impl<const KW: usize> ConvolutionT<u8> for Convolution<u8, KW> {
    fn convolution(&mut self, kernel: &[u8], rules: &Rules) {
        let mut new_base: Vec<Vec<u8>> = Vec::from_iter(
            repeat(Vec::from_iter(repeat(0).take(KW.pow(2)))).take(self.width * self.height),
        );
        let wh = KW / 2;
        for (slice_ix, slice) in self.base.iter().enumerate() {
            let mut acc: u8 = 0;
            for ix in 0..KW.pow(2) {
                acc += slice[ix] * kernel[ix];
            }
            let initial_value = slice[wh * KW + wh];
            Self::set_base_at_index(
                &mut new_base,
                self.width,
                self.height,
                (slice_ix % self.width, slice_ix / self.width),
                rules.apply(initial_value, acc),
            );
        }
        self.base = new_base;
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