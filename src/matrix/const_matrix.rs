use super::*;

#[derive(Debug, Clone, Copy)]
pub struct ConstMatrix<T: Copy + Clone, const M: usize, const N: usize> {
    pub data: [[T; N]; M],
}

impl<const M: usize, const N: usize> Matrix for ConstMatrix<u8, M, N> {
    fn new(_width: usize, _height: usize) -> ConstMatrix<u8, M, N> {
        ConstMatrix { data: [[0; N]; M] }
    }

    fn new_std_conv_matrix(_width: usize, _height: usize) -> ConstMatrix<u8, M, N> {
        let mut data = [[1; N]; M];
        data[N / 2][M / 2] = 0;
        ConstMatrix { data }
    }

    fn new_random(_width: usize, _height: usize) -> ConstMatrix<u8, M, N> {
        let mut data = [[0; N]; M];
        for x in 0..M {
            for y in 0..N {
                data[x][y] = gen_range(0_u8, 2);
            }
        }
        ConstMatrix { data }
    }

    type Output = u8;

    fn index(&self, (x, y): (usize, usize)) -> Self::Output {
        self.data[x][y]
    }

    fn set_at_index(&mut self, (x, y): (usize, usize), value: Self::Output) {
        self.data[x][y] = value;
    }

    fn width(&self) -> usize {
        M
    }

    fn height(&self) -> usize {
        N
    }
}

