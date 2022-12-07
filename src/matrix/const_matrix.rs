use super::*;
use crate::CellType;
use num_traits::{AsPrimitive, One, Zero};
use quad_rand::RandomRange;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

/// row major array of arrays
/// N: width, M: height
#[serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConstMatrix<
    T: Copy + Clone + Serialize + DeserializeOwned,
    const M: usize,
    const N: usize,
> {
    #[serde_as(as = "[[_;N];M]")]
    pub data: [[T; N]; M],
}

impl<
        T: Copy + Zero + One + RandomRange + Serialize + DeserializeOwned + 'static,
        const M: usize,
        const N: usize,
    > Matrix for ConstMatrix<T, M, N>
where
    u8: AsPrimitive<T>,
{
    type Output = T;

    fn new(_width: usize, _height: usize) -> ConstMatrix<T, M, N> {
        ConstMatrix {
            data: [[Zero::zero(); N]; M],
        }
    }

    fn new_std_conv_matrix(_width: usize, _height: usize) -> ConstMatrix<T, M, N> {
        let mut data = [[One::one(); N]; M];
        data[N / 2][M / 2] = Zero::zero();
        ConstMatrix { data }
    }

    fn new_random(_width: usize, _height: usize) -> ConstMatrix<T, M, N> {
        let mut data = [[Zero::zero(); N]; M];
        for slice in data.iter_mut() {
            for item in slice {
                *item = gen_range(Zero::zero(), 2_u8.as_());
            }
        }
        ConstMatrix { data }
    }

    fn new_random_range(
        _width: usize,
        _height: usize,
        range: RangeInclusive<Self::Output>,
    ) -> ConstMatrix<T, M, N> {
        let mut data = [[Zero::zero(); N]; M];
        for slice in data.iter_mut() {
            for item in slice.iter_mut() {
                *item = gen_range(*range.start(), *range.end() + One::one());
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
        for slice in data.iter_mut() {
            for item in slice.iter_mut() {
                *item = gen_range(CellType::NoCell, CellType::H);
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
        for slice in data.iter_mut() {
            for item in slice.iter_mut() {
                *item = CellType::random_range(&range);
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
