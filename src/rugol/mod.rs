use crate::{
    app_config::AppConfig,
    cell_type::{CellType, CellTypeMap},
    color::Color,
    fade::Fader,
    quad_tree::{Node, QuadTree},
    rules::RuleSet,
    save_file::SaveFile,
    FieldType,
};
use egui::TextureHandle;
use egui_dock::Tree;
use instant::Instant;
use matrices::{traits::Matrix, vec_matrix::VecMatrix};

pub mod conv_tabs;
pub mod main;
pub mod ui;

/// <Field type matrix, Convolution matrix, Color matrix>
pub struct RugolState<
    M: Matrix<FieldType> + Clone,
    C: Matrix<FieldType>,
    N: Matrix<Color>,
    const CW: usize,
> {
    conv_kernels: [C; 9],
    pub cell_type_map: CellTypeMap,
    rules: RuleSet<FieldType>,
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
    pub save_file: Option<SaveFile<CW>>,
    pub ui_down_arrow: Option<TextureHandle>,
    pub ui_up_arrow: Option<TextureHandle>,
    tree: Tree<usize>,
}
