use colormap::{ColorMap, ColorMapT};
use egui::Slider;
use egui::{Align, Button, Color32, DragValue, Label, Layout, Rgba, Sense, Separator, Ui, Window};
use fade::Fader;
use instant::{Duration, Instant};
use macroquad::prelude::*;
use std::ops::RangeInclusive;

use matrix::convolution::*;
use matrix::traits::*;
use matrix::{const_matrix::*, vec_matrix::VecMatrix};
use rules::*;

pub mod colormap;
pub mod fade;
pub mod matrix;
pub mod rules;

const CELLS: [(usize, usize); 4] = [(100, 50), (200, 100), (400, 200), (800, 400)];

// type RState = RugolState<ConstMatrix<u8, CELLS_X, CELLS_Y>, ConstMatrix<u8, 3, 3>>;
type FieldType = i8;
type BaseMatrix<const CW: usize> = Convolution<FieldType, CW>;
type RState<const CW: usize> =
    RugolState<BaseMatrix<CW>, ConstMatrix<FieldType, CW, CW>, VecMatrix<[f32; 4]>>;

struct RugolState<M: Matrix + Clone, C: Matrix, N: Matrix<Output = [f32; 4]>> {
    conv_matrix: C,
    color_map: ColorMap,
    rules: Rules<FieldType>,
    fields_vec: Vec<M>,
    fields_vec_ix: usize,
    tick: Instant,
    elapsed: Duration,
    paused: bool,
    fader: Fader<N>,
    bfade: bool,
}

impl<const CW: usize> RState<CW> {
    fn new() -> Self {
        let tick = Instant::now();
        let conv_matrix = ConstMatrix::new_std_conv_matrix(3, 3);
        let fields_vec_ix = 0;
        let fields_vec = {
            let mut f = Vec::new();
            for (cw, ch) in CELLS {
                f.push(Convolution::new_random(cw, ch));
            }
            f
        };
        let color_map = <ColorMap as ColorMapT<FieldType>>::new();
        RugolState {
            conv_matrix,
            color_map,
            rules: classic_rules(),
            fields_vec,
            fields_vec_ix,
            tick,
            elapsed: Duration::new(0, 0),
            paused: true,
            fader: Fader::new(CELLS[fields_vec_ix].0, CELLS[fields_vec_ix].1),
            bfade: false,
        }
    }

    fn step(&mut self) {
        self.tick = Instant::now();
        self.fields_vec[self.fields_vec_ix]
            .convolution(&self.conv_matrix.data.concat(), &self.rules);
        self.fader
            .add(&self.fields_vec[self.fields_vec_ix], &self.color_map);
        self.elapsed = self.tick.elapsed();
    }

    fn get_fields(&self) -> &BaseMatrix<CW> {
        &self.fields_vec[self.fields_vec_ix]
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

    fn edit_range(ui: &mut Ui, range: RangeInclusive<FieldType>) -> RangeInclusive<FieldType> {
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
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for x in 0..self.conv_matrix.width() {
                    ui.horizontal(|ui| {
                        for y in 0..self.conv_matrix.height() {
                            let val = self.conv_matrix.index((x, y));
                            //ui.add(DragValue::new(&mut val));
                            let col = self.color_map.map(&val);
                            let text_col = if (col.r + col.g + col.b) < 0.5 {
                                Color32::GRAY
                            } else {
                                Color32::BLACK
                            };
                            if ui
                                .add(
                                    Label::new(format!("{}", val))
                                        .text_color(text_col)
                                        .strong()
                                        .heading()
                                        .background_color(Rgba::from_rgb(col.r, col.g, col.b))
                                        .sense(Sense::click_and_drag()),
                                )
                                .dragged()
                            {
                                self.conv_matrix
                                    .set_at_index((x, y), self.color_map.get_selected_rules_val());
                            }
                        }
                    });
                }
            });
            ui.vertical(|ui| {
                for x in 0..self.conv_matrix.width() {
                    ui.horizontal(|ui| {
                        for y in 0..self.conv_matrix.height() {
                            let mut val = self.conv_matrix.index((x, y));
                            ui.add(DragValue::new(&mut val));
                            self.conv_matrix.set_at_index((x, y), val);
                        }
                    });
                }
            });
        });
    }
}

#[macroquad::main("Rugol")]
async fn main() {
    let mut gol = <RState<5>>::new();
    let mut mode = UiMode::Warn;
    let mut inst;
    let mut frame_time = 0.;
    loop {
        inst = Instant::now();
        clear_background(BLACK);

        egui_macroquad::ui(|ctx| {
            Window::new("Rugol").show(ctx, |ui| match mode {
                UiMode::Warn => {
                    ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                        ui.add(Label::new("Warning: Depending on the settings this program may produce bright flashing and/or pulsating images").heading().strong());
                        if ui.button("continue").clicked() {
                            mode = UiMode::Main;
                        }
                    });
                }
                UiMode::Main => {
                    #[cfg(not(target_arch = "wasm32"))]
                    ui.label(format!(
                        "calc_time: {:.1} ms",
                        (gol.elapsed.as_micros() as f64) * 0.001
                    ));
                    #[cfg(target_arch = "wasm32")]
                    ui.label(format!("calc_time: {} ms", gol.elapsed.as_micros()));
                    ui.label(format!("frame_time: {:.1} ms", frame_time));
                    gol.control_ui(ui);
                    gol.edit_rules_ui(ui);
                    gol.edit_conv_matrix_ui(ui);
                    if ui.button("Settings").clicked() {
                        mode = UiMode::Settings;
                    }
                    <ColorMap as ColorMapT<FieldType>>::edit(&mut gol.color_map, ui);
                }
                UiMode::Settings => {
                    for ix in 0..CELLS.len() {
                            if ui.radio_value(&mut gol.fields_vec_ix, ix, format!("{}x{}", CELLS[ix].0, CELLS[ix].1)).changed() {
                                let (w, h) = CELLS[gol.fields_vec_ix];
                                gol.fader = Fader::new(w, h);
                            }
                    }
                    ui.checkbox(&mut gol.bfade, "fade");
                    ui.label( "Fader: mix_factor");
                    ui.add(Slider::new(&mut gol.fader.mix_factor, 0.0..=1.0));
                    if ui.button("<-- back").clicked() {
                        mode = UiMode::Main;
                    }
                }
            });
        });

        if !gol.paused {
            gol.step();
        }

        // draw the frame
        for ixx in 0..gol.get_fields().width() {
            for ixy in 0..gol.get_fields().height() {
                let x = (ixx as f32 * screen_width()) / (gol.get_fields().width() as f32);
                let y = (ixy as f32 * screen_height()) / (gol.get_fields().height() as f32);
                let w = screen_width() / (gol.get_fields().width() as f32);
                let h = screen_height() / (gol.get_fields().height() as f32);

                // handle drawing with the mouse pointer on the screen
                let mouse_pos = mouse_position();
                if is_mouse_button_down(MouseButton::Left)
                    && (x..(x + w)).contains(&mouse_pos.0)
                    && (y..(y + h)).contains(&mouse_pos.1)
                {
                    gol.fields_vec[gol.fields_vec_ix]
                        .set_at_index((ixx, ixy), gol.color_map.get_selected_rules_val());
                }

                if gol.bfade {
                    draw_rectangle(x, y, w, h, gol.fader.index(ixx, ixy));
                } else {
                    draw_rectangle(
                        x,
                        y,
                        w,
                        h,
                        gol.color_map.map(&gol.get_fields().index((ixx, ixy))),
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
}
