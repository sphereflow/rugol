use super::*;
use egui::emath::Numeric;
use egui::*;
use macroquad::prelude::*;
use std::ops::RangeInclusive;

use crate::{
    cell_type::{CellType, CellTypeMap},
    fade::Fader,
    matrix::traits::Symmetry,
    quad_tree::QuadTree,
    rules::{flame_rules, Rule},
    RState, UiMode, CELLS,
};

impl<const CW: usize> RState<CW> {
    pub fn ui(&mut self, ctx: &Context) {
        Window::new("Rugol").show(ctx, |ui| {
                match self.config.mode {
                UiMode::Warn => {
                    ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                        ui.add(Label::new(RichText::new("Warning: Depending on the settings this program may produce bright flashing and/or pulsating images").heading().strong()));
                        if ui.button("continue").clicked() {
                            self.config.mode = UiMode::Main;
                        }
                    });
                }
                UiMode::Main => {
                    #[cfg(not(target_arch = "wasm32"))]
                    ui.label(format!(
                        "calc_time: {:.1} ms",
                        (self.config.elapsed.as_micros() as f64) * 0.001
                    ));
                    #[cfg(target_arch = "wasm32")]
                    ui.label(format!("calc_time: {} ms", self.config.elapsed.as_micros()));
                    ui.label(format!("frame_time: {:.1} ms", self.frame_time));
                    if let Some(ix) = self.hover_ix {
                        ui.label(format!("hovered field: {}", self.fields_vec[self.vec_ix].display_element(ix)));
                    }
                    self.control_ui(ui);
                    self.sections_ui(ui);
                    if self.config.ui_sections.reset_fields {
                    self.clear_ui(ui);
                        self.randomize_ui(ui);
                    }
                    if self.config.ui_sections.edit_rules {
                        self.edit_rules_ui(ui);
                    }
                    if self.config.ui_sections.settings {
                        self.settings_ui(ui);
                    }
                    if self.config.ui_sections.edit_conv_matrix {
                        self.edit_conv_matrix_ui(ui);
                    }
                    if self.config.ui_sections.edit_colors {
                        CellTypeMap::edit(&mut self.cell_type_map, ui);
                    }
                    if ui.button("Help").clicked() {
                        self.config.mode = UiMode::Help;
                    }
                }
                UiMode::Help => {
                    ui.label("A description of how Rugol works can be found in the following link:");
                    ui.hyperlink("https://github.com/sphereflow/rugol#how-it-works");
                    if ui.button("<-- back").clicked() {
                        self.config.mode = UiMode::Main;
                    }
                }
                }
            });
        self.config.ui_contains_pointer = ctx.is_pointer_over_area();
    }

    fn sections_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            Self::select_bool_ui(ui, &mut self.config.ui_sections.settings, "Settings");
            Self::select_bool_ui(
                ui,
                &mut self.config.ui_sections.reset_fields,
                "Reset controls",
            );
            Self::select_bool_ui(ui, &mut self.config.ui_sections.edit_rules, "Edit rules");
            Self::select_bool_ui(
                ui,
                &mut self.config.ui_sections.edit_conv_matrix,
                "Edit convolution matrix",
            );
            Self::select_bool_ui(
                ui,
                &mut self.config.ui_sections.edit_colors,
                "Edit colors and values",
            );
        });
    }

    fn select_bool_ui(ui: &mut Ui, b: &mut bool, text: impl Into<WidgetText>) {
        if ui.selectable_label(*b, text).clicked() {
            *b = !*b;
        }
    }

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

    fn settings_ui(&mut self, ui: &mut Ui) {
        for (ix, (w, h)) in CELLS.iter().enumerate() {
            if ui
                .radio_value(&mut self.vec_ix, ix, format!("{}x{}", w, h))
                .changed()
            {
                self.fader = Fader::new(*w, *h);
                self.quad_tree = QuadTree::new(*w, *h, 5);
                self.config.bnew_size = true;
                self.config.bupdate = true;
            }
        }
        ui.checkbox(&mut self.config.bsingle_kernel, "single kernel");
        ui.checkbox(&mut self.config.bfade, "fade");
        ui.add(Slider::new(&mut self.fader.mix_factor, 0.0_f32..=1.0).text("Fader: mix_factor"));
        ui.checkbox(&mut self.config.sym_editting, "symmetric editting");
        if self.config.sym_editting {
            self.edit_symmetry(ui);
        }
        if ui
            .add(Slider::new(&mut self.config.cell_size_factor, 0.1_f32..=3.0).text("cell size"))
            .changed()
        {
            self.config.bnew_size = true;
            self.config.bupdate = true;
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
                ui.add(
                    DragValue::new(&mut rule.transition)
                        .clamp_range::<CellType>(CellType::NoCell..=CellType::H),
                );
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
                                    let col = match self.cell_type_map.color_for_value(val) {
                                        Some(col) => col,
                                        None => WHITE,
                                    };
                                    let text_col = if (col.r + col.g + col.b) < 0.5 {
                                        Color32::GRAY
                                    } else {
                                        Color32::BLACK
                                    };
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
                            if cix > 0 && ui.button("▲").clicked() {
                                copy_indices = vec![(cix, cix - 1)];
                            }

                            if cix < (kernel_len - 1) && ui.button("⯆").clicked() {
                                copy_indices = vec![(cix, cix + 1)];
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
