use super::conv_tabs::ConvWrapper;
#[cfg(not(target_arch = "wasm32"))]
use crate::save_file::*;
use crate::{
    cell_type::{CellType, CellTypeMap},
    fade::Fader,
    quad_tree::QuadTree,
    rules::{flame_rules, Rule},
    RState, UiMode, CELLS, WARN_TEXT,
};
use egui::emath::Numeric;
use egui::*;
use egui_dock::DockArea;
use matrices::traits::Symmetry;
use num_traits::{AsPrimitive, One, Zero};
#[cfg(not(target_arch = "wasm32"))]
use rfd::{AsyncFileDialog, FileDialog};
use std::ops::RangeInclusive;

impl<const CW: usize> RState<CW> {
    pub fn ui(&mut self, ctx: &Context) {
        Window::new("Rugol").show(ctx, |ui| match self.config.mode {
            UiMode::Warn => {
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.add(Label::new(RichText::new(WARN_TEXT).heading().strong()));
                    if ui.button("continue").clicked() {
                        self.config.mode = UiMode::Main;
                    }
                });
            }

            UiMode::Main => {
                self.main_ui(ui);
            }

            #[cfg(not(target_arch = "wasm32"))]
            UiMode::OpenFile => {
                self.load_file_controls(ui);
            }

            #[cfg(not(target_arch = "wasm32"))]
            UiMode::SaveFile => {
                self.save_file(ui);
            }

            UiMode::Help => {
                ui.label("A description of how Rugol works can be found in the following link:");
                ui.hyperlink("https://github.com/sphereflow/rugol#how-it-works");
                if ui.button("<-- back").clicked() {
                    self.config.mode = UiMode::Main;
                }
            }
        });
        self.config.ui_contains_pointer = ctx.is_pointer_over_area();
    }

    fn main_ui(&mut self, ui: &mut Ui) {
        self.timings_ui(ui);
        if let Some(ix) = self.hover_ix {
            ui.label(format!(
                "hovered field: {}",
                self.fields_vec[self.vec_ix].display_element(ix)
            ));
        }
        self.control_ui(ui);
        self.sections_ui(ui);
        if self.config.ui_sections.show_reset_fields() {
            self.clear_ui(ui);
            self.randomize_ui(ui);
        }
        if self.config.ui_sections.show_edit_rules() {
            self.edit_rules_ui(ui);
        }
        if self.config.ui_sections.show_settings() {
            self.settings_ui(ui);
        }
        if self.config.ui_sections.show_edit_conv_matrix() {
            self.edit_conv_matrix_ui(ui);
        }
        if self.config.ui_sections.show_edit_colors() {
            CellTypeMap::edit(&mut self.cell_type_map, ui);
        }
        if ui.button("Help").clicked() {
            self.config.mode = UiMode::Help;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Open file").clicked() {
            let file_dialog = AsyncFileDialog::new().pick_file();
            self.config.mode = UiMode::OpenFile;
            self.save_file = pollster::block_on(async {
                if let Some(file) = file_dialog.await {
                    let bytes = file.read().await;
                    SaveFile::<CW>::load_from_bytes(&bytes).ok()
                } else {
                    None
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Save file").clicked() {
            self.config.mode = UiMode::SaveFile;
        }
    }

    fn timings_ui(&mut self, ui: &mut Ui) {
        #[cfg(not(target_arch = "wasm32"))]
        ui.label(format!(
            "calc_time: {:.1} ms",
            (self.config.elapsed.as_micros() as f64) * 0.001
        ));
        #[cfg(target_arch = "wasm32")]
        ui.label(format!("calc_time: {} ms", self.config.elapsed.as_micros()));
        ui.label(format!("frame_time: {:.1} ms", self.frame_time));
    }

    fn sections_ui(&mut self, ui: &mut Ui) {
        let sections = &mut self.config.ui_sections;
        ui.horizontal(|ui| {
            Self::select_section_and_hover_ui(
                ui,
                &mut sections.settings,
                &mut sections.hover_settings,
                "Settings",
            );
            Self::select_section_and_hover_ui(
                ui,
                &mut sections.reset_fields,
                &mut sections.hover_reset_fields,
                "Reset controls",
            );
            Self::select_section_and_hover_ui(
                ui,
                &mut sections.edit_rules,
                &mut sections.hover_edit_rules,
                "Edit rules",
            );
            Self::select_section_and_hover_ui(
                ui,
                &mut sections.edit_conv_matrix,
                &mut sections.hover_edit_conv_matrix,
                "Edit convolution matrix",
            );
            Self::select_section_and_hover_ui(
                ui,
                &mut sections.edit_colors,
                &mut sections.hover_edit_colors,
                "Edit colors and values",
            );
        });
    }

    fn select_section_and_hover_ui(
        ui: &mut Ui,
        b: &mut bool,
        hover: &mut bool,
        text: impl Into<WidgetText>,
    ) {
        let label = ui.selectable_label(*b, text);
        *hover = label.hovered();
        if label.clicked() {
            *b = !*b;
        }
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
            self.config.randomize_range =
                Self::edit_cell_type_range(ui, self.config.randomize_range.clone());
        });
    }

    fn clear_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("clear").clicked() {
                self.clear();
            }
            ui.label("clear value:");
            Self::edit_cell_type(ui, &mut self.config.clear_val);
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
        ui.checkbox(
            &mut self.config.ui_sections.hover_preview,
            "Preview UI Sections on hover",
        );
        ui.add(Slider::new(&mut self.config.draw_line_thickness, 1_u8..=10));
    }

    fn edit_rules_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.add(Button::new("Add rule")).clicked() {
                self.rules.rules.push(Rule {
                    state: CellType::NoCell,
                    range: Zero::zero()..=Zero::zero(),
                    transition: CellType::NoCell,
                });
            }
            if CW == 5 && ui.button("Flame").clicked() {
                self.conv_kernels[0].data = [[Zero::zero(); CW]; CW];
                for ixy in 0..CW {
                    for ixx in 0..CW {
                        if (ixx + ixy) % 2 == 1 {
                            if (1..4).contains(&ixx) && (1..4).contains(&ixy) {
                                self.conv_kernels[0].data[ixx][ixy] = 2_u8.as_();
                            } else {
                                self.conv_kernels[0].data[ixx][ixy] = One::one();
                            }
                        }
                    }
                }
                self.rules = flame_rules();
            }
        });
        let mut o_delete_ix = None;
        let mut changed = false;
        for (del_ix, rule) in self.rules.rules.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                changed |= Self::edit_cell_type(ui, &mut rule.state);
                ui.label("->");
                changed |= Self::edit_cell_type(ui, &mut rule.transition);
                ui.add(Separator::default());
                if let Some(range) = <RState<CW>>::edit_range(ui, rule.range.clone()) {
                    rule.range = range;
                    changed = true;
                }
                if ui.add(Button::new("Delete rule")).clicked() {
                    o_delete_ix = Some(del_ix);
                }
            });
        }
        if let Some(del_ix) = o_delete_ix {
            self.rules.rules.remove(del_ix);
        }
        if changed {
            self.everything_changed();
        }
    }

    fn edit_cell_type(ui: &mut Ui, cell: &mut CellType) -> bool {
        ui.add(
            DragValue::new(cell).custom_formatter(|num, _| format!("{}", CellType::from_f64(num))),
        )
        .changed()
    }

    fn edit_range<T: Numeric>(ui: &mut Ui, range: RangeInclusive<T>) -> Option<RangeInclusive<T>> {
        let mut start = *range.start();
        let mut end = *range.end();
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("range: ");
            changed |= ui.add(DragValue::new(&mut start).speed(0.01)).changed();
            ui.label("..=");
            changed |= ui.add(DragValue::new(&mut end).speed(0.01)).changed();
        });
        if changed {
            Some(start..=end)
        } else {
            None
        }
    }

    fn edit_cell_type_range(
        ui: &mut Ui,
        range: RangeInclusive<CellType>,
    ) -> RangeInclusive<CellType> {
        let mut start = *range.start();
        let mut end = *range.end();
        ui.horizontal(|ui| {
            ui.label("range: ");
            Self::edit_cell_type(ui, &mut start);
            ui.label("..=");
            Self::edit_cell_type(ui, &mut end);
        });
        start..=end
    }

    fn edit_conv_matrix_ui(&mut self, ui: &mut Ui) {
        let mut convolution_wrapper = ConvWrapper {
            inner: &mut self.conv_kernels,
            config: &mut self.config,
            cell_type_map: &mut self.cell_type_map,
            copy_indices: Vec::new(),
            copy_to_all: None,
        };
        let mut style = egui_dock::Style::from_egui(ui.ctx().style().as_ref());
        style.show_close_buttons = false;
        Window::new("convolution matrices").show(ui.ctx(), |ui| {
            ui.vertical(|ui| {
                DockArea::new(&mut self.tree)
                    .style(style)
                    .show_inside(ui, &mut convolution_wrapper);
            });
        });
        convolution_wrapper.copy_kernels();
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

    #[cfg(not(target_arch = "wasm32"))]
    fn load_file_controls(&mut self, ui: &mut Ui) {
        if let Some(save_file) = self.save_file.as_mut() {
            if save_file.convolution.is_some() {
                Self::select_bool_ui(ui, &mut save_file.include_convolution, "convolution");
            }
            if save_file.rules.is_some() {
                Self::select_bool_ui(ui, &mut save_file.include_rules, "rules");
            }
            if save_file.cell_type_map.is_some() {
                Self::select_bool_ui(ui, &mut save_file.include_cell_type_map, "cell type map");
            }
            if save_file.cells.is_some() {
                Self::select_bool_ui(ui, &mut save_file.include_cells, "cells");
            }
            if ui.button("Load").clicked() {
                self.load_save_file();
                self.config.mode = UiMode::Main;
            }
        } else {
            self.config.mode = UiMode::Main;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_file(&mut self, ui: &mut Ui) {
        let save_file = self.save_file.get_or_insert_with(|| {
            let convolution = if self.config.bsingle_kernel {
                ConvMatrixE::Single(self.conv_kernels[0])
            } else {
                ConvMatrixE::Multiple(self.conv_kernels)
            };
            SaveFile {
                convolution: Some(convolution),
                rules: Some(self.rules.clone()),
                cell_type_map: Some(self.cell_type_map.clone()),
                cells: Some(self.cell_type_vec.clone()),
                include_convolution: true,
                include_rules: true,
                include_cell_type_map: true,
                include_cells: true,
            }
        });
        Self::save_file_controls(&mut self.config.mode, save_file, ui);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_file_controls(mode: &mut UiMode, save_file: &mut SaveFile<CW>, ui: &mut Ui) {
        Self::select_bool_ui(ui, &mut save_file.include_convolution, "convolution");
        Self::select_bool_ui(ui, &mut save_file.include_rules, "rules");
        Self::select_bool_ui(ui, &mut save_file.include_cell_type_map, "cell_type_map");
        Self::select_bool_ui(ui, &mut save_file.include_cells, "cells");
        if ui.button("Save as ...").clicked() {
            if let Some(path_buf) = FileDialog::new().save_file() {
                if let Some(file_path) = path_buf.to_str() {
                    if let Err(e) = save_file.save_to(file_path) {
                        println!("save_file_controls: {:?}", e);
                    }
                }
            }
            *mode = UiMode::Main;
        }
    }
}
