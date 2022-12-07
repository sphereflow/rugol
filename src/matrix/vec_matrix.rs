use super::*;
use crate::{color::Color, CellType};
use num_traits::{AsPrimitive, One, Zero};
use quad_rand::RandomRange;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VecMatrix<T: Copy + Clone> {
    pub data: Vec<T>,
    width: usize,
    height: usize,
}

impl<T: Copy + Zero + One + RandomRange + 'static> Matrix for VecMatrix<T>
where
    u8: AsPrimitive<T>,
{
    type Output = T;
    fn new(width: usize, height: usize) -> VecMatrix<T> {
        VecMatrix {
            data: vec![Zero::zero(); width * height],
            width,
            height,
        }
    }

    fn new_std_conv_matrix(width: usize, height: usize) -> Self {
        let mut data = vec![One::one(); width * height];
        data[width / 2 + (height / 2) * width] = Zero::zero();
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn new_random(width: usize, height: usize) -> Self {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            data.push(gen_range::<T>(Zero::zero(), 2_u8.as_()));
        }
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn new_random_range(width: usize, height: usize, range: RangeInclusive<Self::Output>) -> Self {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            data.push(gen_range(*range.start(), *range.end()));
        }
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn index(&self, (ixx, ixy): (usize, usize)) -> T {
        self.data[ixx + ixy * self.width]
    }

    fn set_at_index(&mut self, (ixx, ixy): (usize, usize), value: T) {
        self.data[ixx + ixy * self.width] = value;
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl Matrix for VecMatrix<Color> {
    type Output = Color;
    fn new(width: usize, height: usize) -> Self {
        VecMatrix {
            data: vec![Color::from([0.; 4]); width * height],
            width,
            height,
        }
    }

    fn new_std_conv_matrix(width: usize, height: usize) -> Self {
        let mut data = vec![Color::from([1.; 4]); width * height];
        data[width / 2 + (height / 2) * width] = Color::from([0.; 4]);
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn new_random(width: usize, height: usize) -> Self {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            let mut color = [0.; 4];
            for rgb in &mut color {
                *rgb = gen_range::<f32>(0., 1.);
            }
            data.push(color.into());
        }
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn new_random_range(width: usize, height: usize, _range: RangeInclusive<Self::Output>) -> Self {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            let mut color = [0.; 4];
            for rgb in &mut color {
                *rgb = gen_range::<f32>(0., 1.);
            }
            data.push(color.into());
        }
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn index(&self, (ixx, ixy): (usize, usize)) -> Color {
        self.data[ixx + ixy * self.width]
    }

    fn set_at_index(&mut self, (ixx, ixy): (usize, usize), value: Color) {
        self.data[ixx + ixy * self.width] = value;
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl Matrix for VecMatrix<CellType> {
    type Output = CellType;
    fn new(width: usize, height: usize) -> Self {
        VecMatrix {
            data: vec![CellType::A; width * height],
            width,
            height,
        }
    }

    fn new_std_conv_matrix(width: usize, height: usize) -> Self {
        let mut data = vec![CellType::A; width * height];
        data[width / 2 + (height / 2) * width] = CellType::NoCell;
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn new_random(width: usize, height: usize) -> Self {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            data.push(gen_range(CellType::NoCell, CellType::H));
        }
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn new_random_range(width: usize, height: usize, range: RangeInclusive<Self::Output>) -> Self {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            data.push(CellType::random_range(&range));
        }
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn index(&self, (ixx, ixy): (usize, usize)) -> CellType {
        self.data[ixx + ixy * self.width]
    }

    fn set_at_index(&mut self, (ixx, ixy): (usize, usize), value: CellType) {
        self.data[ixx + ixy * self.width] = value;
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<T: Copy + Display> Display for VecMatrix<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ixy in 0..self.height {
            for ixx in 0..self.width {
                write!(f, "{}, ", self.data[ixy * self.width + ixx])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
