use crate::{app_config::AppConfig, cell_type::CellTypeMap, color::WHITE, ConvolutionMatrix};
use egui::{
    emath::Numeric, Color32, DragValue, ImageButton, Label, Rgba, RichText, Sense, TextureHandle,
    Ui, Vec2, WidgetText,
};
use egui_dock::TabViewer;
use matrices::traits::Matrix;
use std::ops::RangeInclusive;

pub struct ConvWrapper<'a, const CW: usize> {
    pub inner: &'a mut [ConvolutionMatrix<CW>; 9],
    pub config: &'a mut AppConfig,
    pub cell_type_map: &'a CellTypeMap,
    pub ui_down_arrow: &'a Option<TextureHandle>,
    pub ui_up_arrow: &'a Option<TextureHandle>,
    pub copy_indices: Vec<(usize, usize)>,
    pub copy_to_all: Option<usize>,
}

impl<'a, const CW: usize> TabViewer for ConvWrapper<'a, CW> {
    type Tab = usize;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        let convolution_index = *tab;
        ui.horizontal(|ui| {
            self.colored_matrix_ui(ui, convolution_index);
            ui.separator();
            ui.vertical(|ui| {
                self.drag_value_matrix_ui(ui, convolution_index);
            });
        });
        ui.separator();
        self.copy_kernels_ui(ui, convolution_index);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        (tab.to_string()).into()
    }
}

impl<'a, const CW: usize> ConvWrapper<'a, CW> {
    fn edit_range<T: Numeric>(ui: &mut Ui, range: RangeInclusive<T>) -> RangeInclusive<T> {
        let mut start = *range.start();
        let mut end = *range.end();
        ui.horizontal(|ui| {
            ui.label("range: ");
            ui.add(DragValue::new(&mut start).speed(0.01));
            ui.label("..=");
            ui.add(DragValue::new(&mut end).speed(0.01));
        });
        start..=end
    }

    fn colored_matrix_ui(&mut self, ui: &mut Ui, matrix_index: usize) {
        let w = self.inner[matrix_index].width();
        let h = self.inner[matrix_index].height();
        if self.config.bmirror_conv_kernels {
            ui.horizontal(|ui| {
                for x in (0..w).rev() {
                    ui.vertical(|ui| {
                        for y in (0..h).rev() {
                            self.edit_colored_value(ui, matrix_index, x, y);
                        }
                    });
                }
            });
        } else {
            ui.horizontal(|ui| {
                for x in 0..w {
                    ui.vertical(|ui| {
                        for y in 0..h {
                            self.edit_colored_value(ui, matrix_index, x, y);
                        }
                    });
                }
            });
        }
    }

    fn edit_colored_value(&mut self, ui: &mut Ui, matrix_index: usize, x: usize, y: usize) {
        let conv_matrix = &mut self.inner[matrix_index];
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
                        .background_color(Rgba::from_rgb(col.r, col.g, col.b)),
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
                conv_matrix.set_at_index((x, y), self.cell_type_map.get_selected_rules_val());
            }
        }
    }

    fn drag_value_matrix_ui(&mut self, ui: &mut Ui, convolution_index: usize) {
        let conv_matrix = &mut self.inner[convolution_index];
        if self.config.bmirror_conv_kernels {
            ui.horizontal(|ui| {
                for x in (0..conv_matrix.width()).rev() {
                    ui.vertical(|ui| {
                        for y in (0..conv_matrix.height()).rev() {
                            Self::edit_drag_value(conv_matrix, x, y, ui);
                        }
                    });
                }
            });
        } else {
            ui.horizontal(|ui| {
                for x in 0..conv_matrix.width() {
                    ui.vertical(|ui| {
                        for y in 0..conv_matrix.height() {
                            Self::edit_drag_value(conv_matrix, x, y, ui);
                        }
                    });
                }
            });
        }
    }

    fn edit_drag_value(conv_matrix: &mut ConvolutionMatrix<CW>, x: usize, y: usize, ui: &mut Ui) {
        let mut val = conv_matrix.index((x, y));
        ui.add(DragValue::new(&mut val).speed(0.01));
        conv_matrix.set_at_index((x, y), val);
    }

    fn copy_kernels_ui(&mut self, ui: &mut Ui, convolution_index: usize) {
        let kernel_len = self.inner.len();
        if !self.config.bsingle_kernel {
            ui.vertical(|ui| {
                // copy up / down
                if let Some(up) = self.ui_up_arrow {
                    if convolution_index > 0
                        && ui
                            .add(ImageButton::new(up, Vec2::new(10.0, 10.0)))
                            .clicked()
                    {
                        self.copy_indices = vec![(convolution_index, convolution_index - 1)];
                    }
                }

                if let Some(down) = self.ui_down_arrow {
                    if convolution_index < (kernel_len - 1)
                        && ui
                            .add(ImageButton::new(down, Vec2::new(10.0, 10.0)))
                            .clicked()
                    {
                        self.copy_indices = vec![(convolution_index, convolution_index + 1)];
                    }
                }

                ui.horizontal(|ui| {
                    // copy to range
                    if ui.button("copy to range").clicked() {
                        let ix_range = self.config.conv_matrix_copy_range.start().as_index()
                            ..=self.config.conv_matrix_copy_range.end().as_index();
                        for ix in ix_range {
                            self.copy_indices.push((convolution_index, ix));
                        }
                    }
                    self.config.conv_matrix_copy_range =
                        Self::edit_range(ui, self.config.conv_matrix_copy_range.clone());
                });

                // copy to all
                if ui.button("copy to all").clicked() {
                    self.copy_to_all = Some(convolution_index);
                }
            });
        }
    }

    pub fn copy_kernels(&mut self) {
        for (from, to) in self.copy_indices.iter().copied() {
            self.inner[to] = self.inner[from];
        }
        if let Some(from) = self.copy_to_all {
            let from_kernel = self.inner[from];
            for kernel in self.inner.iter_mut() {
                *kernel = from_kernel;
            }
        }
    }
}
