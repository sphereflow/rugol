use super::*;
use crate::CellType;

/// row major array of arrays
/// N: width, M: height
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
        for y in 0..M {
            for x in 0..N {
                data[y][x] = gen_range(0_u8, 2);
            }
        }
        ConstMatrix { data }
    }

    fn new_random_range(
        _width: usize,
        _height: usize,
        range: RangeInclusive<Self::Output>,
    ) -> ConstMatrix<u8, M, N> {
        let mut data = [[0; N]; M];
        for x in 0..M {
            for y in 0..N {
                data[y][x] = gen_range(*range.start(), range.end() + 1);
            }
        }
        ConstMatrix { data }
    }

    type Output = u8;

    fn index(&self, (x, y): (usize, usize)) -> Self::Output {
        self.data[y][x]
    }

    fn set_at_index(&mut self, (x, y): (usize, usize), value: Self::Output) {
        self.data[y][x] = value;
    }

    fn width(&self) -> usize {
        N
    }

    fn height(&self) -> usize {
        M
    }
}

impl<const M: usize, const N: usize> Matrix for ConstMatrix<i8, M, N> {
    type Output = i8;
    fn new(_width: usize, _height: usize) -> ConstMatrix<i8, M, N> {
        ConstMatrix { data: [[0; N]; M] }
    }

    fn new_std_conv_matrix(_width: usize, _height: usize) -> ConstMatrix<i8, M, N> {
        let mut data = [[1; N]; M];
        data[N / 2][M / 2] = 0;
        ConstMatrix { data }
    }

    fn new_random(_width: usize, _height: usize) -> ConstMatrix<i8, M, N> {
        let mut data = [[0; N]; M];
        for y in 0..M {
            for x in 0..N {
                data[y][x] = gen_range::<i16>(0, 2) as i8;
            }
        }
        ConstMatrix { data }
    }

    fn new_random_range(
        _width: usize,
        _height: usize,
        range: RangeInclusive<Self::Output>,
    ) -> ConstMatrix<i8, M, N> {
        let mut data = [[0; N]; M];
        for y in 0..M {
            for x in 0..N {
                data[y][x] = gen_range::<i16>(*range.start() as i16, *range.end() as i16 + 1) as i8;
            }
        }
        ConstMatrix { data }
    }

    fn index(&self, (x, y): (usize, usize)) -> Self::Output {
        self.data[y][x]
    }

    fn set_at_index(&mut self, (x, y): (usize, usize), value: Self::Output) {
        self.data[y][x] = value;
    }

    fn width(&self) -> usize {
        N
    }

    fn height(&self) -> usize {
        M
    }
}

impl<const M: usize, const N: usize> Matrix for ConstMatrix<CellType, M, N> {
    type Output = CellType;

    fn new(_width: usize, _height: usize) -> ConstMatrix<CellType, M, N> {
        ConstMatrix {
            data: [[CellType::NoCell; N]; M],
        }
    }

    fn new_std_conv_matrix(_width: usize, _height: usize) -> ConstMatrix<CellType, M, N> {
        let mut data = [[CellType::A; N]; M];
        data[N / 2][M / 2] = CellType::NoCell;
        ConstMatrix { data }
    }

    fn new_random(_width: usize, _height: usize) -> ConstMatrix<CellType, M, N> {
        let mut data: [[CellType; N]; M] = [[CellType::NoCell; N]; M];
        for y in 0..M {
            for x in 0..N {
                data[y][x] = gen_range(CellType::NoCell, CellType::H);
            }
        }
        ConstMatrix { data }
    }

    fn new_random_range(
        _width: usize,
        _height: usize,
        range: RangeInclusive<Self::Output>,
    ) -> Self {
        let mut data: [[CellType; N]; M] = [[CellType::NoCell; N]; M];
        for y in 0..M {
            for x in 0..N {
                data[y][x] = CellType::random_range(&range);
            }
        }
        ConstMatrix { data }
    }

    fn index(&self, (x, y): (usize, usize)) -> Self::Output {
        self.data[y][x]
    }

    fn set_at_index(&mut self, (x, y): (usize, usize), value: Self::Output) {
        self.data[y][x] = value;
    }

    fn width(&self) -> usize {
        N
    }

    fn height(&self) -> usize {
        M
    }
}
