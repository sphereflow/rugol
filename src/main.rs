use cell_type::{CellType, CellTypeMap};
use egui::emath::Numeric;
use egui::{Align, Button, Color32, DragValue, Label, Layout, Rgba, Sense, Separator, Ui, Window};
use egui::{RadioButton, RichText, Slider};
use fade::Fader;
use instant::{Duration, Instant};
use macroquad::prelude::*;
use std::ops::RangeInclusive;

use matrix::convolution::*;
use matrix::traits::*;
use matrix::{const_matrix::*, vec_matrix::VecMatrix};
use quad_tree::*;
use rules::*;

pub mod cell_type;
pub mod fade;
pub mod matrix;
pub mod quad_tree;
pub mod rules;

const CELLS: [(usize, usize); 5] = [(10, 5), (100, 50), (200, 100), (400, 200), (800, 400)];

// type RState = RugolState<ConstMatrix<u8, CELLS_X, CELLS_Y>, ConstMatrix<u8, 3, 3>>;
type FieldType = i8;
type BaseMatrix<const CW: usize> = Convolution<FieldType, CW>;
type RState<const CW: usize> =
    RugolState<BaseMatrix<CW>, ConstMatrix<FieldType, CW, CW>, VecMatrix<[f32; 4]>>;

struct RugolState<M: Matrix + Clone, C: Matrix, N: Matrix<Output = [f32; 4]>> {
    conv_kernels: [C; 9],
    cell_type_map: CellTypeMap,
    rules: Rules<FieldType>,
    /// Vec of matrices with `FieldType` elements
    fields_vec: Vec<M>,
    /// Vec of matrices with `CellType` elements
    cell_type_vec: Vec<VecMatrix<CellType>>,
    /// Accumulator matrices with `FieldType` elements
    acc_vec: Vec<VecMatrix<FieldType>>,
    /// Index to `fields_vec` and `cell_type_vec`
    vec_ix: usize,
    fader: Fader<N>,
    config: AppConfig,
    quad_tree: QuadTree<Node>,
}

struct AppConfig {
    tick: Instant,
    elapsed: Duration,
    paused: bool,
    bfade: bool,
    bsingle_kernel: bool,
    ui_contains_pointer: bool,
    randomize_range: RangeInclusive<CellType>,
    conv_matrix_copy_range: RangeInclusive<CellType>,
    clear_val: CellType,
    sym_editting: bool,
    symmetry: Symmetry,
    cell_size_factor: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            tick: Instant::now(),
            elapsed: Duration::new(0, 0),
            paused: true,
            bfade: false,
            bsingle_kernel: true,
            ui_contains_pointer: false,
            randomize_range: CellType::NoCell..=CellType::A,
            conv_matrix_copy_range: CellType::NoCell..=CellType::A,
            clear_val: CellType::NoCell,
            sym_editting: false,
            symmetry: Symmetry::XY,
            cell_size_factor: 0.9,
        }
    }
}

impl<const CW: usize> RState<CW> {
    fn new() -> Self {
        let conv_matrix = ConstMatrix::new_std_conv_matrix(3, 3);
        let cell_type_map = CellTypeMap::new();
        let fields_vec_ix = 0;
        let (fields_vec, cell_type_vec, acc_vec) = {
            let mut f = Vec::new();
            let mut ct = Vec::new();
            let mut acc = Vec::new();
            for (cw, ch) in CELLS {
                let mut field_type_matrix = Convolution::new(cw, ch);
                let cell_type_matrix: VecMatrix<CellType> = VecMatrix::new_random(cw, ch);
                for x in 0..cw {
                    for y in 0..ch {
                        field_type_matrix
                            .set_at_index((x, y), cell_type_map[cell_type_matrix.index((x, y))].1);
                    }
                }
                f.push(field_type_matrix);
                ct.push(cell_type_matrix);
                acc.push(VecMatrix::new(cw, ch));
            }
            (f, ct, acc)
        };
        RugolState {
            conv_kernels: [conv_matrix; 9],
            cell_type_map,
            rules: classic_rules(),
            fields_vec,
            cell_type_vec,
            acc_vec,
            vec_ix: fields_vec_ix,
            fader: Fader::new(CELLS[fields_vec_ix].0, CELLS[fields_vec_ix].1),
            config: AppConfig::default(),
            quad_tree: QuadTree::new(CELLS[fields_vec_ix].0, CELLS[fields_vec_ix].1, 5),
        }
    }

    fn step(&mut self) {
        self.config.tick = Instant::now();
        let field_type_matrix = &mut self.fields_vec[self.vec_ix];
        let cell_type_matrix = &mut self.cell_type_vec[self.vec_ix];
        let acc_matrix = &mut self.acc_vec[self.vec_ix];
        let indices = {
            let mut res = Vec::new();
            let mut range_vec = Vec::new();
            self.quad_tree.get_changed_ranges(CW, 0, 0, &mut range_vec);
            //dbg!(&range_vec);
            for range in range_vec.iter_mut() {
                let (x_start, x_end) = *range.start();
                let (y_start, y_end) = *range.end();
                for x in x_start..=x_end {
                    for y in y_start..=y_end {
                        res.push((x, y));
                    }
                }
            }
            res
        };
        field_type_matrix.convolution(
            &self.conv_kernels,
            self.config.bsingle_kernel,
            cell_type_matrix,
            acc_matrix,
            &indices,
        );
        // dbg!(&indices);
        // self.quad_tree.print_levels();
        self.quad_tree.clear();
        // map the accumulated values to the cell matrix
        // field_type_matrix -> self.rules.apply(...) -> self.cell_type_vec[self.vec_ix]
        // self.cell_type_vec[self.vec_ix] -> self.map.lookup(...) -> self.fields_vec[self.vec_ix]
        for (ixx, ixy) in indices {
            let acc = acc_matrix.index((ixx, ixy));
            let initial_cell = cell_type_matrix.index((ixx, ixy));
            let cell = self.rules.apply(initial_cell, acc);
            let field = self.cell_type_map[cell].1;
            if cell != initial_cell {
                cell_type_matrix.set_at_index((ixx, ixy), cell);
                field_type_matrix.set_at_index((ixx, ixy), field);
                self.quad_tree.insert(ixx, ixy, 0, 0);
            }
        }
        self.fader
            .add(&self.cell_type_vec[self.vec_ix], &self.cell_type_map);
        self.config.elapsed = self.config.tick.elapsed();
    }

    fn randomize(&mut self, range: RangeInclusive<CellType>) {
        let fields = &mut self.fields_vec[self.vec_ix];
        let cells = &mut self.cell_type_vec[self.vec_ix];
        let w = fields.width();
        let h = fields.height();
        *cells = VecMatrix::new_random_range(w, h, range);
        self.quad_tree.everything_changed();
        for x in 0..w {
            for y in 0..h {
                fields.set_at_index((x, y), self.cell_type_map[cells.index((x, y))].1);
            }
        }
    }

    fn donut_all_kernels(&mut self, range: RangeInclusive<usize>, val: FieldType) {
        for kernel in self.conv_kernels.iter_mut() {
            kernel.donut(range.clone(), val);
        }
    }

    fn clear(&mut self) {
        let fields = &mut self.fields_vec[self.vec_ix];
        let cells = &mut self.cell_type_vec[self.vec_ix];
        let field_value = self.cell_type_map[self.config.clear_val].1;
        cells.clear(self.config.clear_val);
        fields.clear(field_value);
        self.quad_tree.everything_changed();
    }

    fn get_fields(&self) -> &BaseMatrix<CW> {
        &self.fields_vec[self.vec_ix]
    }

    fn get_cells(&self) -> &VecMatrix<CellType> {
        &self.cell_type_vec[self.vec_ix]
    }

    // Ui stuff

    fn randomize_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Random").clicked() {
                self.randomize(CellType::NoCell..=CellType::H);
            }
            if ui.button("Random range").clicked() {
                self.randomize(self.config.randomize_range.clone());
            }
            self.config.randomize_range = Self::edit_range(ui, self.config.randomize_range.clone());
        });
    }

    fn clear_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("clear").clicked() {
                self.clear();
            }
            ui.label("clear value:");
            ui.add(DragValue::new(&mut self.config.clear_val));
        });
    }

    fn control_ui(&mut self, ui: &mut Ui) {
        if self.config.paused {
            ui.horizontal(|ui| {
                if ui.button("▶").clicked() {
                    self.config.paused = false;
                }
                if ui.button("›").clicked() {
                    self.step();
                }
            });
        } else if ui.button("⏸").clicked() {
            self.config.paused = true;
        }
    }

    fn edit_rules_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.add(Button::new("Add rule")).clicked() {
                self.rules.rules.push(Rule {
                    state: CellType::NoCell,
                    range: 0..=0,
                    transition: CellType::NoCell,
                });
            }
            if CW == 5 && ui.button("Flame").clicked() {
                self.conv_kernels[0].data = [[0; CW]; CW];
                for ixy in 0..CW {
                    for ixx in 0..CW {
                        if (ixx + ixy) % 2 == 1 {
                            if (1..4).contains(&ixx) && (1..4).contains(&ixy) {
                                self.conv_kernels[0].data[ixx][ixy] = 2;
                            } else {
                                self.conv_kernels[0].data[ixx][ixy] = 1;
                            }
                        }
                    }
                }
                self.rules = flame_rules();
            }
        });
        let mut o_delete_ix = None;
        for (del_ix, rule) in self.rules.rules.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut rule.state));
                ui.label("->");
                ui.add(DragValue::new(&mut rule.transition).clamp_range(0..=7));
                ui.add(Separator::default());
                rule.range = <RState<CW>>::edit_range(ui, rule.range.clone());
                if ui.add(Button::new("Delete rule")).clicked() {
                    o_delete_ix = Some(del_ix);
                }
            });
        }
        if let Some(del_ix) = o_delete_ix {
            self.rules.rules.remove(del_ix);
        }
    }

    fn edit_range<T: Numeric>(ui: &mut Ui, range: RangeInclusive<T>) -> RangeInclusive<T> {
        let mut start = *range.start();
        let mut end = *range.end();
        ui.horizontal(|ui| {
            ui.label("range: ");
            ui.add(DragValue::new(&mut start));
            ui.label("..=");
            ui.add(DragValue::new(&mut end));
        });
        start..=end
    }

    fn edit_conv_matrix_ui(&mut self, ui: &mut Ui) {
        let kernel_len = self.conv_kernels.len();
        let mut copy_indices: Vec<(usize, usize)> = Vec::new();
        let mut copy_to_all = None;
        for (cix, conv_matrix) in self.conv_kernels.iter_mut().enumerate() {
            let collapsing_text = if self.config.bsingle_kernel {
                "Convolution Matrix".into()
            } else {
                format!("Convolution Matrix: {}", cix)
            };
            ui.collapsing(collapsing_text, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for y in 0..conv_matrix.height() {
                            ui.horizontal(|ui| {
                                for x in 0..conv_matrix.width() {
                                    let val = conv_matrix.index((x, y));
                                    //ui.add(DragValue::new(&mut val));
                                    let text_col;
                                    let col = match self.cell_type_map.color_for_value(val) {
                                        Some(col) => col,
                                        None => WHITE,
                                    };
                                    if (col.r + col.g + col.b) < 0.5 {
                                        text_col = Color32::GRAY
                                    } else {
                                        text_col = Color32::BLACK
                                    }
                                    if ui
                                        .add(
                                            Label::new(
                                                RichText::new(format!("{}", val))
                                                    .color(text_col)
                                                    .strong()
                                                    .heading()
                                                    .background_color(Rgba::from_rgb(
                                                        col.r, col.g, col.b,
                                                    )),
                                            )
                                            .sense(Sense::click_and_drag()),
                                        )
                                        .dragged()
                                    {
                                        if self.config.sym_editting {
                                            conv_matrix.set_at_index_sym(
                                                self.config.symmetry,
                                                (x, y),
                                                self.cell_type_map.get_selected_rules_val(),
                                            )
                                        } else {
                                            conv_matrix.set_at_index(
                                                (x, y),
                                                self.cell_type_map.get_selected_rules_val(),
                                            );
                                        }
                                    }
                                }
                            });
                        }
                    });
                    ui.vertical(|ui| {
                        for y in 0..conv_matrix.height() {
                            ui.horizontal(|ui| {
                                for x in 0..conv_matrix.width() {
                                    let mut val = conv_matrix.index((x, y));
                                    ui.add(DragValue::new(&mut val));
                                    conv_matrix.set_at_index((x, y), val);
                                }
                            });
                        }
                    });

                    // copying of kernels
                    if !self.config.bsingle_kernel {
                        ui.vertical(|ui| {
                            // copy up / down
                            if cix > 0 {
                                if ui.button("▲").clicked() {
                                    copy_indices = vec![(cix, cix - 1)];
                                }
                            }
                            if cix < (kernel_len - 1) {
                                if ui.button("⯆").clicked() {
                                    copy_indices = vec![(cix, cix + 1)];
                                }
                            }

                            ui.horizontal(|ui| {
                                // copy to range
                                if ui.button("copy to range").clicked() {
                                    let ix_range =
                                        self.config.conv_matrix_copy_range.start().as_index()
                                            ..=self.config.conv_matrix_copy_range.end().as_index();
                                    for ix in ix_range {
                                        copy_indices.push((cix, ix));
                                    }
                                }
                                self.config.conv_matrix_copy_range = Self::edit_range(
                                    ui,
                                    self.config.conv_matrix_copy_range.clone(),
                                );
                            });

                            // copy to all
                            if ui.button("copy to all").clicked() {
                                copy_to_all = Some(cix);
                            }
                        });
                    }
                });
            });
            if self.config.bsingle_kernel {
                break;
            }
        }
        for (from, to) in copy_indices {
            self.conv_kernels[to] = self.conv_kernels[from];
        }
        if let Some(from) = copy_to_all {
            let from_kernel = self.conv_kernels[from];
            for kernel in self.conv_kernels.iter_mut() {
                *kernel = from_kernel;
            }
        }
    }

    fn edit_symmetry(&mut self, ui: &mut Ui) {
        let symmetries = [
            Symmetry::X,
            Symmetry::Y,
            Symmetry::XY,
            Symmetry::ROT90,
            Symmetry::ROT180,
            Symmetry::DONUT,
        ];
        for sym in symmetries {
            if ui
                .add(RadioButton::new(
                    sym == self.config.symmetry,
                    format!("{:?}", sym),
                ))
                .clicked()
            {
                self.config.symmetry = sym;
            }
        }
    }
}

#[macroquad::main("Rugol")]
async fn main() {
    let mut gol = <RState<7>>::new();
    gol.donut_all_kernels(0..=0, 0);
    let mut mode = UiMode::Warn;
    let mut inst;
    let mut frame_time = 0.;
    loop {
        inst = Instant::now();
        clear_background(BLACK);

        egui_macroquad::ui(|ctx| {
            Window::new("Rugol").show(ctx, |ui| {
                match mode {
                UiMode::Warn => {
                    ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                        ui.add(Label::new(RichText::new("Warning: Depending on the settings this program may produce bright flashing and/or pulsating images").heading().strong()));
                        if ui.button("continue").clicked() {
                            mode = UiMode::Main;
                        }
                    });
                }
                UiMode::Main => {
                    #[cfg(not(target_arch = "wasm32"))]
                    ui.label(format!(
                        "calc_time: {:.1} ms",
                        (gol.config.elapsed.as_micros() as f64) * 0.001
                    ));
                    #[cfg(target_arch = "wasm32")]
                    ui.label(format!("calc_time: {} ms", gol.config.elapsed.as_micros()));
                    ui.label(format!("frame_time: {:.1} ms", frame_time));
                    gol.clear_ui(ui);
                    gol.randomize_ui(ui);
                    gol.control_ui(ui);
                    gol.edit_rules_ui(ui);
                    gol.edit_conv_matrix_ui(ui);
                    if ui.button("Settings").clicked() {
                        mode = UiMode::Settings;
                    }
                    if ui.button("Help").clicked() {
                        mode = UiMode::Help;
                    }
                    CellTypeMap::edit(&mut gol.cell_type_map, ui);
                }
                UiMode::Settings => {
                    for (ix, (w, h)) in CELLS.iter().enumerate() {
                            if ui.radio_value(&mut gol.vec_ix, ix, format!("{}x{}", w, h)).changed() {
                                gol.fader = Fader::new(*w, *h);
                                gol.quad_tree = QuadTree::new(*w, *h, 5);
                            }
                    }
                    ui.checkbox(&mut gol.config.bsingle_kernel, "single kernel");
                    ui.checkbox(&mut gol.config.bfade, "fade");
                    ui.add(Slider::new(&mut gol.fader.mix_factor, 0.0..=1.0).text("Fader: mix_factor"));
                    ui.checkbox(&mut gol.config.sym_editting, "symmetric editting");
                    if gol.config.sym_editting {
                        gol.edit_symmetry(ui);
                    }
                    ui.add(Slider::new(&mut gol.config.cell_size_factor, 0.1..=3.0).text("cell size"));
                    if ui.button("<-- back").clicked() {
                        mode = UiMode::Main;
                    }
                }
                UiMode::Help => {
                    ui.label("A description of how Rugol works can be found in the following link:");
                    ui.hyperlink("https://github.com/sphereflow/rugol#how-it-works");
                    if ui.button("<-- back").clicked() {
                        mode = UiMode::Main;
                    }
                }
                }
            });
            gol.config.ui_contains_pointer = ctx.is_pointer_over_area();
        });

        if !gol.config.paused {
            gol.step();
        }

        // draw the frame
        for ixx in 0..gol.get_fields().width() {
            for ixy in 0..gol.get_fields().height() {
                let x = (ixx as f32 * screen_width()) / (gol.get_fields().width() as f32);
                let y = (ixy as f32 * screen_height()) / (gol.get_fields().height() as f32);
                let w = screen_width() / (gol.get_fields().width() as f32);
                let h = screen_height() / (gol.get_fields().height() as f32);
                let f = gol.config.cell_size_factor;
                let x_adjust = 0.5 * w * (1.0 - f);
                let y_adjust = 0.5 * h * (1.0 - f);
                let x = x + x_adjust;
                let y = y + y_adjust;
                let w = w * f;
                let h = h * f;

                // handle drawing with the mouse pointer on the screen
                let mouse_pos = mouse_position();
                if is_mouse_button_down(MouseButton::Left)
                    && !gol.config.ui_contains_pointer
                    && (x..(x + w)).contains(&mouse_pos.0)
                    && (y..(y + h)).contains(&mouse_pos.1)
                {
                    let val = gol.cell_type_map.get_selected_rules_val();
                    let cell = gol.cell_type_map.get_selected_rules_cell();
                    gol.fields_vec[gol.vec_ix].set_at_index((ixx, ixy), val);
                    gol.cell_type_vec[gol.vec_ix].set_at_index((ixx, ixy), cell);
                    gol.quad_tree.insert(ixx, ixy, 0, 0);
                }

                if gol.config.bfade {
                    draw_rectangle(x, y, w, h, gol.fader.index(ixx, ixy));
                } else {
                    draw_rectangle(
                        x,
                        y,
                        w,
                        h,
                        gol.cell_type_map[gol.get_cells().index((ixx, ixy))].0,
                    );
                }
            }
        }
        egui_macroquad::draw();

        #[cfg(not(target_arch = "wasm32"))]
        {
            frame_time = (inst.elapsed().as_micros() as f64) * 0.001;
        }
        #[cfg(target_arch = "wasm32")]
        {
            frame_time = inst.elapsed().as_micros() as f64;
        }

        next_frame().await
    }
}

enum UiMode {
    Warn,
    Main,
    Settings,
    Help,
}
