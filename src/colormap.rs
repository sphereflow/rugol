use std::fmt::Display;

use egui::{RadioButton, Ui};
use macroquad::{color_u8, prelude::Color};

pub struct ColorMap {
    map: Vec<Color>,
    selected_idx: usize,
}

pub trait ColorMapT<T: Copy + Display> {
    fn new() -> Self;
    fn map(&self, rules_val: &T) -> Color;
    fn inv_lookup(&self, idx: usize) -> T;
    fn get_map_mut(&mut self) -> &mut Vec<Color>;
    fn get_map(&self) -> &Vec<Color>;
    fn get_selected_idx(&self) -> usize;
    fn set_selected_idx(&mut self, idx: usize);
    fn get_selected_rules_val(&self) -> T {
        self.inv_lookup(self.get_selected_idx())
    }
    fn get_selected_color(&self) -> Color;
    fn edit(&mut self, ui: &mut Ui) {
        let mut selected_idx = self.get_selected_idx();
        for chunk in (0..self.get_map().len())
            .collect::<Vec<usize>>()
            .chunks_mut(4)
        {
            ui.horizontal(|ui| {
                for &idx in chunk.iter() {
                    if ui
                        .add(RadioButton::new(
                            idx == selected_idx,
                            format!("{}", self.inv_lookup(idx)),
                        ))
                        .clicked()
                    {
                        selected_idx = idx;
                    }
                    let map = self.get_map_mut();
                    let mut edit_color = [map[idx].r, map[idx].g, map[idx].b];
                    if ui.color_edit_button_rgb(&mut edit_color).changed() {
                        map[idx].r = edit_color[0];
                        map[idx].g = edit_color[1];
                        map[idx].b = edit_color[2];
                    }
                }
            });
        }
        self.set_selected_idx(selected_idx);
    }
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
        ColorMap {
            map,
            selected_idx: 0,
        }
    }

    fn map(&self, rules_val: &u8) -> Color {
        match self.map.get(*rules_val as usize) {
            Some(color) => *color,
            _ => {
                color_u8!(255, 255, 255, 255)
            }
        }
    }

    fn inv_lookup(&self, idx: usize) -> u8 {
        idx as u8
    }

    fn get_map_mut(&mut self) -> &mut Vec<Color> {
        &mut self.map
    }

    fn get_map(&self) -> &Vec<Color> {
        &self.map
    }

    fn get_selected_idx(&self) -> usize {
        self.selected_idx
    }

    fn set_selected_idx(&mut self, idx: usize) {
        self.selected_idx = idx.clamp(0, 7);
    }

    fn get_selected_color(&self) -> Color {
        self.map[self.selected_idx]
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
        ColorMap {
            map,
            selected_idx: 0,
        }
    }

    fn map(&self, rules_val: &i8) -> Color {
        let val = rules_val + 3;
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

    fn inv_lookup(&self, idx: usize) -> i8 {
        idx as i8 - 3
    }

    fn get_map_mut(&mut self) -> &mut Vec<Color> {
        &mut self.map
    }

    fn get_map(&self) -> &Vec<Color> {
        &self.map
    }

    fn get_selected_idx(&self) -> usize {
        self.selected_idx
    }

    fn set_selected_idx(&mut self, idx: usize) {
        self.selected_idx = idx.clamp(0, 7);
    }

    fn get_selected_color(&self) -> Color {
        self.map[self.selected_idx]
    }
}
