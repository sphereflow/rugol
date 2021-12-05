use egui::{Button, DragValue, Separator, Slider, Ui, Window};
use instant::{Duration, Instant};
use macroquad::prelude::*;
use num_traits::{zero, Num};
use std::{
    iter::{repeat, Sum},
    ops::{Add, Mul, RangeInclusive},
};

use matrix::traits::*;
use matrix::const_matrix::*;
use matrix::convolution::*;
use rules::*;

pub mod matrix;
pub mod rules;

const CELLS_X: usize = 400;
const CELLS_Y: usize = 200;

// type RState = RugolState<ConstMatrix<u8, CELLS_X, CELLS_Y>, ConstMatrix<u8, 3, 3>>;
type RState5 = RugolState<Convolution<u8, 5>, ConstMatrix<u8, 5, 5>>;

struct RugolState<M: Matrix + Clone, C: Matrix> {
    conv_matrix: C,
    rules: Rules,
    fields: M,
    timer: u64,
    tick: Instant,
    elapsed: Duration,
    wait: u64,
}

impl RState5 {
    fn new() -> Self {
        let tick = Instant::now();
        let conv_matrix = ConstMatrix::new_std_conv_matrix(3, 3);
        let fields = Convolution::new_random(CELLS_X, CELLS_Y);
        RugolState {
            conv_matrix,
            rules: classic_rules(),
            fields,
            timer: 0,
            tick,
            elapsed: Duration::new(0, 0),
            wait: 1,
        }
    }

    fn step(&mut self) {
        self.tick = Instant::now();
        self.fields
            .convolution(&self.conv_matrix.data.concat(), &self.rules);
        // let mut res = Convolution::new(CELLS_X, CELLS_Y);

        // for ixx in 0..CELLS_X {
        //     for ixy in 0..CELLS_Y {
        //         let conv = self.fields.index((ixx, ixy));
        //         let val = self.rules.apply(conv);
        //         self.fields.set_at_index((ixx, ixy), val);
        //     }
        // }

        // self.fields = res;
        self.elapsed = self.tick.elapsed();
    }

    // xxxxxxxxxxxxxxxxxxxxxxxxxxxx
    // xxxxxxxxxxxxx*CCxxxxxxxxxxxx
    // xxxxxxxxxxxxxCECxxxxxxxxxxxx
    // xxxxxxxxxxxxxCCCxxxxxxxxxxxx
    // xxxxxxxxxxxxxxxxxxxxxxxxxxxx
    // xxxxxxxxxxxxxxxxxxxxxxxxxxxx
    // xxxxxxxxxxxxxxxxxxxxxxxxxxxx
    // C = convolution
    // E = element
    // * = ix_cut{x/y}
    fn convolution(&mut self, ixx: usize, ixy: usize) -> u8 {
        let cut_x: i32 = ixx as i32 - (self.conv_matrix.width() / 2) as i32;
        let cut_y: i32 = ixy as i32 - (self.conv_matrix.height() / 2) as i32;
        let fields_old = self.fields.clone();
        let mut acc = 0;
        for (conv_x, ix_cut_x) in (cut_x.max(0)
            ..(cut_x + self.conv_matrix.width() as i32).min(self.fields.width() as i32))
            .enumerate()
        {
            for (conv_y, ix_cut_y) in (cut_y.max(0)
                ..(cut_y + self.conv_matrix.height() as i32).min(self.fields.height() as i32))
                .enumerate()
            {
                acc += self.conv_matrix.index((conv_x, conv_y))
                    * fields_old.index((ix_cut_x as usize, ix_cut_y as usize));
            }
        }

        acc
    }

    fn edit_rules_ui(&mut self, ui: &mut Ui) {
        if ui.add(Button::new("Add rule")).clicked() {
            self.rules.rules.push(Rule {
                state: 0,
                range: 0..=0,
                transition: 0,
            });
        }
        let mut o_delete_ix = None;
        for (del_ix, rule) in self.rules.rules.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut rule.state).clamp_range(0..=7));
                ui.label("->");
                ui.add(DragValue::new(&mut rule.transition).clamp_range(0..=7));
                ui.add(Separator::default());
                rule.range = RugolState::edit_range(ui, rule.range.clone());
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
    let mut gol = RugolState::new();
    loop {
        clear_background(BLACK);

        egui_macroquad::ui(|ctx| {
            Window::new("Rugol").show(ctx, |ui| {
                ui.label(format!(
                    "frame_time: {} ms",
                    (gol.elapsed.as_micros() as f64) * 0.001
                ));
                gol.edit_rules_ui(ui);
                gol.edit_conv_matrix_ui(ui);
            });
        });

        gol.timer += 1;
        if (gol.timer % gol.wait) == 0 {
            gol.step();
        }
        for ixx in 0..gol.fields.width() {
            for ixy in 0..gol.fields.height() {
                let x = (ixx as f32 * screen_width()) / (gol.fields.width() as f32);
                let y = (ixy as f32 * screen_height()) / (gol.fields.height() as f32);
                let w = screen_width() / (gol.fields.width() as f32);
                let h = screen_height() / (gol.fields.height() as f32);
                match gol.fields.index((ixx, ixy)) {
                    0 => draw_rectangle(x, y, w, h, color_u8!(50, 20, 20, 255)),
                    1 => draw_rectangle(x, y, w, h, color_u8!(170, 40, 40, 255)),
                    2 => draw_rectangle(x, y, w, h, color_u8!(120, 80, 40, 255)),
                    3 => draw_rectangle(x, y, w, h, color_u8!(70, 120, 40, 255)),
                    4 => draw_rectangle(x, y, w, h, color_u8!(20, 160, 40, 255)),
                    5 => draw_rectangle(x, y, w, h, color_u8!(0, 100, 70, 255)),
                    6 => draw_rectangle(x, y, w, h, color_u8!(0, 40, 120, 255)),
                    7 => draw_rectangle(x, y, w, h, color_u8!(0, 0, 170, 255)),
                    _ => draw_rectangle(x, y, w, h, color_u8!(255, 255, 255, 255)),
                }
            }
        }
        egui_macroquad::draw();
        next_frame().await
    }
}
