use cell_type::CellType;
use color::Color;
use matrix::{const_matrix::*, vec_matrix::VecMatrix, convolution::Convolution};
use render_mini::mini_main;
use rugol::RugolState;

pub mod app_config;
pub mod cell_type;
pub mod fade;
pub mod matrix;
pub mod quad_tree;
pub mod render_mini;
pub mod rugol;
pub mod rules;
pub mod index_set;
pub mod color;

const CELLS: [(usize, usize); 5] = [(10, 5), (100, 50), (200, 100), (400, 200), (800, 400)];

pub enum ConvolutionWidth {
    Three,
    Five,
}

// type RState = RugolState<ConstMatrix<u8, CELLS_X, CELLS_Y>, ConstMatrix<u8, 3, 3>>;
type FieldType = f32;
type BaseMatrix<const CW: usize> = Convolution<FieldType, CW>;
type RState<const CW: usize> =
    RugolState<BaseMatrix<CW>, ConstMatrix<FieldType, CW, CW>, VecMatrix<Color>>;

fn main() {
    mini_main();
}

pub enum UiMode {
    Warn,
    Main,
    Help,
}
