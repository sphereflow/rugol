use crate::{cell_type::CellTypeMap, color::Color, CellType};
use matrices::traits::Matrix;

pub struct Fader<M: Matrix<Color>> {
    color_matrix: M,
    pub mix_factor: f32,
}

impl<M: Matrix<Color>> Fader<M> {
    pub fn new(width: usize, height: usize) -> Fader<M> {
        Fader {
            color_matrix: M::new(width, height, Color::from_rgba(0, 0, 0, 0)),
            mix_factor: 0.7,
        }
    }

    pub fn add<N: Matrix<CellType>>(&mut self, val_matrix: &N, cmap: &CellTypeMap) {
        if val_matrix.height() != self.color_matrix.height()
            || val_matrix.width() != self.color_matrix.width()
        {
            panic!("Fader::add(...) : Matrices have different dimensions");
        }
        for ixx in 0..self.color_matrix.width() {
            for ixy in 0..self.color_matrix.height() {
                let old_color: [f32; 4] = self.color_matrix.index((ixx, ixy)).into();
                let mut new_color: [f32; 4] = cmap[val_matrix.index((ixx, ixy))].0.into();
                for ix in 0..4 {
                    new_color[ix] =
                        new_color[ix] * self.mix_factor + old_color[ix] * (1. - self.mix_factor);
                }
                self.color_matrix
                    .set_at_index((ixx, ixy), Color::from(new_color));
            }
        }
    }

    pub fn index(&self, ixx: usize, ixy: usize) -> Color {
        self.color_matrix.index((ixx, ixy)).into()
    }
}
