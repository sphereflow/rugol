use instant::Instant;

use crate::{
    app_config::AppConfig,
    cell_type::{CellType, CellTypeMap},
    fade::Fader,
    matrix::{
        traits::{ConvolutionT, Matrix},
        vec_matrix::VecMatrix,
    },
    quad_tree::{Node, QuadTree},
    rules::Rules,
    FieldType,
};

pub mod rugol_main;
pub mod rugol_ui;

pub struct RugolState<M: Matrix + Clone, C: Matrix, N: Matrix<Output = [f32; 4]>> {
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
    pub fader: Fader<N>,
    pub config: AppConfig,
    pub quad_tree: QuadTree<Node>,
    pub inst: Instant,
    pub frame_time: f64,
}
