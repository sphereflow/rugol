use crate::{cell_type::CellType, UiMode};
use instant::{Duration, Instant};
use matrices::traits::Symmetry;
use std::ops::RangeInclusive;

pub struct AppConfig {
    pub tick: Instant,
    pub elapsed: Duration,
    pub paused: bool,
    pub bupdate: bool,
    pub bfade: bool,
    pub bsingle_kernel: bool,
    pub ui_contains_pointer: bool,
    pub randomize_range: RangeInclusive<CellType>,
    pub conv_matrix_copy_range: RangeInclusive<CellType>,
    pub clear_val: CellType,
    pub sym_editting: bool,
    pub symmetry: Symmetry,
    pub cell_size_factor: f32,
    pub mode: UiMode,
    pub bnew_size: bool,
    pub ui_sections: UiSections,
    pub draw_line_thickness: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            tick: Instant::now(),
            elapsed: Duration::new(0, 0),
            paused: true,
            bupdate: true,
            bfade: false,
            bsingle_kernel: true,
            ui_contains_pointer: false,
            randomize_range: CellType::NoCell..=CellType::A,
            conv_matrix_copy_range: CellType::NoCell..=CellType::A,
            clear_val: CellType::NoCell,
            sym_editting: false,
            symmetry: Symmetry::XY,
            cell_size_factor: 1.0,
            mode: UiMode::Warn,
            bnew_size: false,
            ui_sections: UiSections::default(),
            draw_line_thickness: 1,
        }
    }
}

#[derive(Default)]
pub struct UiSections {
    pub settings: bool,
    pub hover_settings: bool,
    pub reset_fields: bool,
    pub hover_reset_fields: bool,
    pub edit_rules: bool,
    pub hover_edit_rules: bool,
    pub edit_conv_matrix: bool,
    pub hover_edit_conv_matrix: bool,
    pub edit_colors: bool,
    pub hover_edit_colors: bool,
    pub hover_preview: bool,
}

impl UiSections {
    pub fn show_settings(&self) -> bool {
        self.settings || (self.hover_settings && self.hover_preview)
    }
    pub fn show_reset_fields(&self) -> bool {
        self.reset_fields || (self.hover_reset_fields && self.hover_preview)
    }
    pub fn show_edit_rules(&self) -> bool {
        self.edit_rules || (self.hover_edit_rules && self.hover_preview)
    }
    pub fn show_edit_conv_matrix(&self) -> bool {
        self.edit_conv_matrix || (self.hover_edit_conv_matrix && self.hover_preview)
    }
    pub fn show_edit_colors(&self) -> bool {
        self.edit_colors || (self.hover_edit_colors && self.hover_preview)
    }
}
