use crate::convolution::*;
use cell_type::CellType;
use color::Color;
use matrices::{const_matrix::*, vec_matrix::VecMatrix};
use render_mini::mini_main;
use rugol::RugolState;

pub mod app_config;
pub mod cell_type;
pub mod color;
pub mod convolution;
pub mod fade;
pub mod index_set;
pub mod quad_tree;
pub mod render_mini;
pub mod rugol;
pub mod rules;
pub mod save_file;
pub mod traits;
pub mod zoom_window;

const CELLS: [(usize, usize); 5] = [(10, 5), (100, 50), (200, 100), (400, 200), (800, 400)];
static WARN_TEXT: &str = "Warning: Depending on the settings this program may produce bright flashing and/or pulsating images";

pub enum ConvolutionWidth {
    Three,
    Five,
}

// type RState = RugolState<ConstMatrix<u8, CELLS_X, CELLS_Y>, ConstMatrix<u8, 3, 3>>;
// width of the convolution matrix
const CONVOLUTION_WIDTH: usize = 7;
type FieldType = f32;
// FieldType matrix
type BaseMatrix<const CW: usize> = Convolution<FieldType, CW>;
type ConvolutionMatrix<const CW: usize> = ConstMatrix<FieldType, CW, CW>;
type RState<const CW: usize> =
    RugolState<BaseMatrix<CW>, ConvolutionMatrix<CW>, VecMatrix<Color>, CW>;

fn main() {
    mini_main();
}

pub enum UiMode {
    Warn,
    Main,
    #[cfg(not(target_arch = "wasm32"))]
    OpenFile,
    #[cfg(not(target_arch = "wasm32"))]
    SaveFile,
    Help,
}
