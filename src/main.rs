use cell_type::CellType;
use instant::Instant;
use macroquad::prelude::*;

use matrix::convolution::*;
use matrix::traits::Matrix;
use matrix::{const_matrix::*, vec_matrix::VecMatrix};
use render_mini::mini_main;
use rugol::RugolState;

pub mod app_config;
pub mod cell_type;
pub mod fade;
pub mod matrix;
pub mod quad_tree;
pub mod render_mini;
pub mod rugol;
pub mod rules;
pub mod index_set;

const CELLS: [(usize, usize); 5] = [(10, 5), (100, 50), (200, 100), (400, 200), (800, 400)];

pub enum ConvolutionWidth {
    Three,
    Five,
}

// type RState = RugolState<ConstMatrix<u8, CELLS_X, CELLS_Y>, ConstMatrix<u8, 3, 3>>;
type FieldType = i8;
type BaseMatrix<const CW: usize> = Convolution<FieldType, CW>;
type RState<const CW: usize> =
    RugolState<BaseMatrix<CW>, ConstMatrix<FieldType, CW, CW>, VecMatrix<[f32; 4]>>;

fn main() {
    mini_main();
}

// #[macroquad::main("Rugol")]
// async fn main() {
//     macro_main().await;
// }

pub async fn macro_main() {
    let mut gol = <RState<7>>::new();
    gol.donut_all_kernels(0..=0, 0);
    loop {
        gol.inst = Instant::now();
        clear_background(BLACK);

        egui_macroquad::ui(|ctx| {
            gol.ui(ctx);
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
                    gol.set_selected_at_index(ixx, ixy);
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
            gol.frame_time = (gol.inst.elapsed().as_micros() as f64) * 0.001;
        }
        #[cfg(target_arch = "wasm32")]
        {
            gol.frame_time = gol.inst.elapsed().as_micros() as f64;
        }

        next_frame().await
    }
}

pub enum UiMode {
    Warn,
    Main,
    Help,
}
