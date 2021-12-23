use macroquad::prelude::Color;

use crate::{colormap::ColorMap, matrix::traits::Matrix, ColorMapT, FieldType};

pub struct Fader<M: Matrix> {
    color_matrix: M,
    pub mix_factor: f32,
}

impl<M: Matrix<Output = [f32; 4]>> Fader<M> {
    pub fn new(width: usize, height: usize) -> Fader<M> {
        Fader {
            color_matrix: M::new(width, height),
            mix_factor: 0.7,
        }
    }

    pub fn add<N: Matrix<Output = FieldType>>(&mut self, val_matrix: &N, cmap: &ColorMap) {
        if val_matrix.height() != self.color_matrix.height()
            || val_matrix.width() != self.color_matrix.width()
        {
            panic!("Fader::add(...) : Matrices have different dimensions");
        }
        for ixx in 0..self.color_matrix.width() {
            for ixy in 0..self.color_matrix.height() {
                let old_color = self.color_matrix.index((ixx, ixy));
                let mut new_color: [f32; 4] = cmap.map(&val_matrix.index((ixx, ixy))).into();
                for ix in 0..4 {
                    new_color[ix] =
                        new_color[ix] * self.mix_factor + old_color[ix] * (1. - self.mix_factor);
                }
                self.color_matrix.set_at_index((ixx, ixy), new_color);
            }
        }
    }

    pub fn index(&self, ixx: usize, ixy: usize) -> Color {
        self.color_matrix.index((ixx, ixy)).into()
    }
}
