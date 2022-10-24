use instant::Instant;

use crate::{
    app_config::AppConfig,
    cell_type::{CellType, CellTypeMap},
    color::Color,
    fade::Fader,
    matrix::{traits::Matrix, vec_matrix::VecMatrix},
    quad_tree::{Node, QuadTree},
    rules::Rules,
    FieldType,
};

pub mod main;
pub mod ui;

pub struct RugolState<M: Matrix + Clone, C: Matrix, N: Matrix<Output = Color>> {
    conv_kernels: [C; 9],
    pub cell_type_map: CellTypeMap,
    rules: Rules<FieldType>,
    /// Vec of matrices with `FieldType` elements
    fields_vec: Vec<M>,
    /// Vec of matrices with `CellType` elements
    cell_type_vec: Vec<VecMatrix<CellType>>,
    /// Accumulator matrices with `FieldType` elements
    acc_vec: Vec<VecMatrix<FieldType>>,
    /// Index to `fields_vec` and `cell_type_vec`
    vec_ix: usize,
    /// Index of the field the mouse is currently over
    /// None if the mouse pointer is outside the window or over the ui
    pub hover_ix: Option<(usize, usize)>,
    pub fader: Fader<N>,
    pub config: AppConfig,
    quad_tree: QuadTree<Node>,
    pub inst: Instant,
    pub frame_time: f64,
}
