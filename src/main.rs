use colormap::{ColorMap, ColorMapT};
use egui::{Button, DragValue, Separator, Slider, Ui, Window};
use instant::{Duration, Instant};
use macroquad::prelude::*;
use num_traits::{zero, Num};
use std::{
    iter::{repeat, Sum},
    ops::{Add, Mul, RangeInclusive},
};

use matrix::const_matrix::*;
use matrix::convolution::*;
use matrix::traits::*;
use rules::*;

pub mod colormap;
pub mod fade;
pub mod matrix;
pub mod rules;

const CELLS_X: usize = 400;
const CELLS_Y: usize = 200;

// type RState = RugolState<ConstMatrix<u8, CELLS_X, CELLS_Y>, ConstMatrix<u8, 3, 3>>;
type RState<const CW: usize> = RugolState<Convolution<u8, CW>, ConstMatrix<u8, CW, CW>>;

struct RugolState<M: Matrix + Clone, C: Matrix> {
    conv_matrix: C,
    rules: Rules,
    fields: M,
    tick: Instant,
    elapsed: Duration,
    paused: bool,
}

impl<const CW: usize> RState<CW> {
    fn new() -> Self {
        let tick = Instant::now();
        let conv_matrix = ConstMatrix::new_std_conv_matrix(3, 3);
        let fields = Convolution::new_random(CELLS_X, CELLS_Y);
        RugolState {
            conv_matrix,
            rules: classic_rules(),
            fields,
            tick,
            elapsed: Duration::new(0, 0),
            paused: true,
        }
    }

    fn step(&mut self) {
        self.tick = Instant::now();
        self.fields
            .convolution(&self.conv_matrix.data.concat(), &self.rules);
        self.elapsed = self.tick.elapsed();
    }

    fn control_ui(&mut self, ui: &mut Ui) {
        if self.paused {
            ui.horizontal(|ui| {
                if ui.button("▶").clicked() {
                    self.paused = false;
                }
                if ui.button("›").clicked() {
                    self.step();
                }
            });
        } else {
            if ui.button("⏸").clicked() {
                self.paused = true;
            }
        }
    }

    fn edit_rules_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.add(Button::new("Add rule")).clicked() {
                self.rules.rules.push(Rule {
                    state: 0,
                    range: 0..=0,
                    transition: 0,
                });
            }
            if CW == 5 {
                if ui.button("Flame").clicked() {
                    self.conv_matrix.data = [[0; CW]; CW];
                    for ixy in 0..CW {
                        for ixx in 0..CW {
                            if (ixx + ixy) % 2 == 1 {
                                if (1..4).contains(&ixx) && (1..4).contains(&ixy) {
                                    self.conv_matrix.data[ixx][ixy] = 2;
                                } else {
                                    self.conv_matrix.data[ixx][ixy] = 1;
                                }
                            }
                        }
                    }
                    self.rules = flame_rules();
                }
            }
        });
        let mut o_delete_ix = None;
        for (del_ix, rule) in self.rules.rules.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut rule.state).clamp_range(0..=7));
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

    fn edit_range(ui: &mut Ui, range: RangeInclusive<u8>) -> RangeInclusive<u8> {
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
        for y in 0..self.conv_matrix.height() {
            ui.horizontal(|ui| {
                for x in 0..self.conv_matrix.width() {
                    let mut val = self.conv_matrix.index((x, y));
                    ui.add(DragValue::new(&mut val));
                    self.conv_matrix.set_at_index((x, y), val);
                }
            });
        }
    }
}

#[macroquad::main("Rugol")]
async fn main() {
    let mut gol = <RState<5>>::new();
    let mut color_map = <ColorMap as ColorMapT<u8>>::new();
    loop {
        clear_background(BLACK);

        egui_macroquad::ui(|ctx| {
            Window::new("Rugol").show(ctx, |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                ui.label(format!(
                    "frame_time: {} ms",
                    (gol.elapsed.as_micros() as f64) * 0.001
                ));
                #[cfg(target_arch = "wasm32")]
                ui.label(format!("frame_time: {} ms", gol.elapsed.as_micros()));
                gol.control_ui(ui);
                gol.edit_rules_ui(ui);
                gol.edit_conv_matrix_ui(ui);
                color_map.edit(ui);
            });
        });

        if !gol.paused {
            gol.step();
        }

        // draw the frame
        for ixx in 0..gol.fields.width() {
            for ixy in 0..gol.fields.height() {
                let x = (ixx as f32 * screen_width()) / (gol.fields.width() as f32);
                let y = (ixy as f32 * screen_height()) / (gol.fields.height() as f32);
                let w = screen_width() / (gol.fields.width() as f32);
                let h = screen_height() / (gol.fields.height() as f32);
                draw_rectangle(x, y, w, h, color_map.map(gol.fields.index((ixx, ixy))));
            }
        }
        egui_macroquad::draw();
        next_frame().await
    }
}
