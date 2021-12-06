use egui::Ui;
use macroquad::{color_u8, prelude::Color};

pub struct ColorMap {
    map: Vec<Color>,
}

pub trait ColorMapT<T> {
    fn new() -> Self;
    fn map(&self, val: T) -> Color;
}

impl ColorMapT<u8> for ColorMap {
    fn new() -> Self {
        let map = vec![
            color_u8!(50, 20, 20, 255),
            color_u8!(170, 40, 40, 255),
            color_u8!(120, 80, 40, 255),
            color_u8!(70, 120, 40, 255),
            color_u8!(20, 160, 40, 255),
            color_u8!(0, 100, 70, 255),
            color_u8!(0, 40, 120, 255),
            color_u8!(0, 0, 170, 255),
        ];
        ColorMap { map }
    }

    fn map(&self, val: u8) -> Color {
        match self.map.get(val as usize) {
            Some(color) => *color,
            _ => {
                color_u8!(255, 255, 255, 255)
            }
        }
    }
}

impl ColorMapT<i8> for ColorMap {
    fn new() -> Self {
        let map = vec![
            color_u8!(50, 20, 20, 255),
            color_u8!(170, 40, 40, 255),
            color_u8!(120, 80, 40, 255),
            color_u8!(70, 120, 40, 255),
            color_u8!(20, 160, 40, 255),
            color_u8!(0, 100, 70, 255),
            color_u8!(0, 40, 120, 255),
            color_u8!(0, 0, 170, 255),
        ];
        ColorMap { map }
    }

    fn map(&self, val: i8) -> Color {
        let val = val + 3;
        if val < 0 {
            return color_u8!(255, 255, 255, 255);
        }
        match self.map.get(val as usize) {
            Some(color) => *color,
            _ => {
                color_u8!(255, 255, 255, 255)
            }
        }
    }
}

impl ColorMap {
    pub fn edit(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for color in self.map.iter_mut() {
                let mut edit_color = [color.r, color.g, color.b];
                if ui.color_edit_button_rgb(&mut edit_color).changed() {
                    color.r = edit_color[0];
                    color.g = edit_color[1];
                    color.b = edit_color[2];
                }
            }
        });
    }
}
