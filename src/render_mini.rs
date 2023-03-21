use egui::{Color32, ColorImage, ImageData, TextureOptions};
use egui_miniquad::EguiMq;
use instant::Instant;
use matrices::traits::Matrix;
use miniquad::*;
use num_traits::Zero;

use crate::{RState, CONVOLUTION_WIDTH};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Vertex {
    pos: Vec2,
    color: [f32; 4],
}

struct Stage {
    egui_mini: EguiMq,
    pipeline: Pipeline,
    bindings: Vec<Bindings>,
    vertices: Vec<Vec<Vertex>>,
    vertex_buffers: Vec<Buffer>,
    index_buffer: Buffer,
    gol: RState<CONVOLUTION_WIDTH>,
    bdraw: bool,
    last_draw_index: Option<(usize, usize)>,
}

impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
        let index_buffer = Buffer::immutable::<u16>(ctx, BufferType::IndexBuffer, &[]);
        let bindings = vec![];

        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta()).unwrap();

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("color", VertexFormat::Float4),
            ],
            shader,
        );
        let mut gol = <RState<CONVOLUTION_WIDTH>>::new();
        gol.donut_all_kernels(0..=1, Zero::zero());

        // arrows
        let egui_mini = EguiMq::new(ctx);
        let mut image_up_arrow = ColorImage::new([20, 20], Color32::DARK_GRAY);
        let w = image_up_arrow.width();
        let h = image_up_arrow.height();
        for y in 0..w {
            for x in 0..h {
                if ((y >= (h - 2 * x)) && (2 * x < w)) || ((y >= (2 * x - h)) && (2 * x >= w)) {
                    image_up_arrow.pixels[x + y * w] = Color32::LIGHT_GRAY;
                }
            }
        }
        let mut image_down_arrow = ColorImage::new([20, 20], Color32::DARK_GRAY);
        let w = image_down_arrow.width();
        let h = image_down_arrow.height();
        for y in 0..w {
            for x in 0..h {
                if ((y < (2 * x)) && (2 * x < w)) || ((y < (2 * h - 2 * x)) && (2 * x >= w)) {
                    image_down_arrow.pixels[x + y * w] = Color32::LIGHT_GRAY;
                }
            }
        }
        gol.ui_up_arrow = Some(egui_mini.egui_ctx().load_texture(
            "",
            ImageData::Color(image_up_arrow),
            TextureOptions::default(),
        ));
        gol.ui_down_arrow = Some(egui_mini.egui_ctx().load_texture(
            "",
            ImageData::Color(image_down_arrow),
            TextureOptions::default(),
        ));

        let mut res = Stage {
            pipeline,
            bindings,
            vertices: vec![],
            vertex_buffers: vec![],
            index_buffer,
            gol,
            bdraw: false,
            last_draw_index: None,
            egui_mini,
        };
        res.new_size_selected(ctx);
        res
    }

    fn new_size_selected(&mut self, ctx: &mut Context) {
        self.vertices.clear();
        self.vertex_buffers.clear();
        self.bindings.clear();
        let width = self.gol.get_fields().width();
        let height = self.gol.get_fields().height();
        // set up indices
        let mut indices: Vec<u16> = Vec::new();
        for index in 0..width {
            let index_base = 4 * index as u16;
            indices.extend_from_slice(&[
                index_base,
                index_base + 1,
                index_base + 2,
                index_base,
                index_base + 2,
                index_base + 3,
            ]);
        }
        self.index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);
        for ixy in 0..height {
            self.vertices.push(vec![]);
            for ixx in 0..width {
                let matrix_width = self.gol.get_fields().width() as f32;
                let matrix_height = self.gol.get_fields().height() as f32;
                let x = (ixx as f32 * 2.0) / matrix_width;
                let y = (ixy as f32 * 2.0) / matrix_height;
                let w = 2.0 / (width as f32);
                let h = 2.0 / (height as f32);
                let f = self.gol.config.cell_size_factor;
                let x_adjust = 0.5 * w * (1.0 - f) - 1.0;
                let y_adjust = 0.5 * h * (1.0 - f) - 1.0;
                let x = x + x_adjust;
                let y = y + y_adjust;
                let w = w * f;
                let h = h * f;

                let color: [f32; 4] = if self.gol.config.bfade {
                    self.gol.fader.index(ixx, ixy).into()
                } else {
                    self.gol.cell_type_map[self.gol.get_cells().index((ixx, ixy))]
                        .0
                        .into()
                };
                self.vertices[ixy].extend_from_slice(&[
                    Vertex {
                        pos: Vec2 { x, y },
                        color,
                    },
                    Vertex {
                        pos: Vec2 { x, y: y + h },
                        color,
                    },
                    Vertex {
                        pos: Vec2 { x: x + w, y: y + h },
                        color,
                    },
                    Vertex {
                        pos: Vec2 { x: x + w, y },
                        color,
                    },
                ]);
            }
            //self.vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &self.vertices);
            self.vertex_buffers.push(Buffer::stream(
                ctx,
                BufferType::VertexBuffer,
                self.vertices[ixy].len() * std::mem::size_of::<Vertex>(),
            ));
            self.bindings.push(Bindings {
                vertex_buffers: vec![self.vertex_buffers[ixy]],
                index_buffer: self.index_buffer,
                images: vec![],
            });
        }
    }

    fn mouse_pos_to_index(&self, ctx: &mut Context, mouse_x: f32, mouse_y: f32) -> (usize, usize) {
        let (win_width, win_height) = ctx.screen_size();
        let (field_width, field_height) = (
            self.gol.get_fields().width(),
            self.gol.get_fields().height(),
        );
        let ixx = (field_width as f32 * mouse_x / win_width) as usize;
        let ixy = (field_height as f32 * mouse_y / win_height) as usize;
        (ixx.min(field_width - 1), ixy.min(field_height - 1))
    }

    fn draw_circle_at_index(&mut self, ixx: usize, ixy: usize) {
        let thickness = (self.gol.config.draw_line_thickness - 1) as usize;
        for (xoff, tx) in (ixx - thickness.min(ixx)..(ixx + thickness)).enumerate() {
            for (yoff, ty) in (ixy - thickness.min(ixy)..(ixy + thickness)).enumerate() {
                let xsq = (xoff as i32 - thickness as i32).pow(2);
                let ysq = (yoff as i32 - thickness as i32).pow(2);
                if (((xsq + ysq) as usize) < (thickness * thickness))
                    && self.gol.is_valid_index(tx, ty)
                {
                    self.gol.set_selected_at_index(tx, ty);
                }
            }
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self, ctx: &mut Context) {
        ctx.clear(Some((0., 0., 0., 1.)), None, None);
        if self.gol.config.bnew_size {
            self.new_size_selected(ctx);
            self.gol.config.bnew_size = false;
        }
        if !self.gol.config.paused {
            self.gol.step();
        }
        if self.gol.config.bupdate {
            for ixy in 0..self.gol.get_fields().height() {
                for ixx in 0..self.gol.get_fields().width() {
                    let color: [f32; 4] = if self.gol.config.bfade {
                        self.gol.fader.index(ixx, ixy).into()
                    } else {
                        self.gol.cell_type_map[self.gol.get_cells().index((ixx, ixy))]
                            .0
                            .into()
                    };
                    self.vertices[ixy][ixx * 4].color = color;
                    self.vertices[ixy][ixx * 4 + 1].color = color;
                    self.vertices[ixy][ixx * 4 + 2].color = color;
                    self.vertices[ixy][ixx * 4 + 3].color = color;
                }

                self.vertex_buffers[ixy].update(ctx, &self.vertices[ixy]);
                //self.vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &self.vertices);
                self.bindings[ixy] = Bindings {
                    vertex_buffers: vec![self.vertex_buffers[ixy]],
                    index_buffer: self.index_buffer,
                    images: vec![],
                };
            }
            self.gol.config.bupdate = false;
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x_pos: f32,
        y_pos: f32,
    ) {
        // handle drawing with the mouse pointer on the screen
        if button == MouseButton::Left && !self.gol.config.ui_contains_pointer {
            let (ixx, ixy) = self.mouse_pos_to_index(ctx, x_pos, y_pos);
            self.last_draw_index = Some((ixx, ixy));
            self.gol.set_selected_at_index(ixx, ixy);
            self.draw_circle_at_index(ixx, ixy);
            self.gol.config.bupdate = true;
            self.bdraw = true;
        }
        self.egui_mini
            .mouse_button_down_event(ctx, button, x_pos, y_pos);
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            if self.bdraw && !self.gol.config.ui_contains_pointer {
                let (ixx, ixy) = self.mouse_pos_to_index(ctx, x, y);
                self.draw_circle_at_index(ixx, ixy);
                self.gol.config.bupdate = true;
            }
            self.last_draw_index = None;
            self.bdraw = false;
        }
        self.egui_mini.mouse_button_up_event(ctx, button, x, y);
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        self.egui_mini.mouse_motion_event(x, y);
        if self.gol.config.ui_contains_pointer {
            self.last_draw_index = None;
            self.bdraw = false;
        }
        if !self.gol.config.ui_contains_pointer {
            let (ixx, ixy) = self.mouse_pos_to_index(ctx, x, y);
            self.gol.hover_ix = Some((ixx, ixy));
            if self.bdraw {
                self.draw_circle_at_index(ixx, ixy);
                match self.last_draw_index {
                    Some((from_ixx, from_ixy)) => {
                        let (to_ixx, to_ixy) = (ixx, ixy);
                        let up = to_ixy as f32 - from_ixy as f32;
                        let right = to_ixx as f32 - from_ixx as f32;
                        if right == 0. {
                            let start = ixy.min(from_ixy).min(self.gol.get_fields().height() - 1);
                            let end = ixy.max(from_ixy).min(self.gol.get_fields().height() - 1);
                            if from_ixx >= self.gol.get_fields().width() {
                                return;
                            }
                            for y in start..=end {
                                self.gol.set_selected_at_index(from_ixx, y);
                                // we went up or down, so paint with thickness right and left
                                for x in 1..(self.gol.config.draw_line_thickness as isize) {
                                    let tx = from_ixx + x as usize;
                                    if self.gol.is_valid_index(tx, y) {
                                        self.gol.set_selected_at_index(tx, y)
                                    }
                                    let tx = ((from_ixx as isize - x).max(0)) as usize;
                                    if self.gol.is_valid_index(tx, y) {
                                        self.gol.set_selected_at_index(tx, y)
                                    }
                                }
                            }
                        } else {
                            let ratio = up / right;
                            let mut go_up = 0_isize;
                            let mut go_right = 0_isize;
                            let (mut current_x, mut current_y) = (from_ixx, from_ixy);
                            while (to_ixx, to_ixy) != (current_x, current_y) {
                                let lr = right.signum();
                                let ud = up.signum();
                                // ratio difference if we went right or left
                                let r = ((go_up as f32) / (go_right as f32 + lr) - ratio).abs();
                                // ratio difference if we went up or down
                                let u = ((go_up as f32 + ud) / (go_right as f32) - ratio).abs();
                                // pick the smallest ratio difference
                                if r < u {
                                    go_right += lr as isize;
                                } else {
                                    go_up += ud as isize;
                                }
                                if ((go_right < 0) && ((from_ixx as isize) < go_right.abs()))
                                    || ((go_up < 0) && ((from_ixy as isize) < go_up.abs()))
                                {
                                    break;
                                }
                                current_x = (from_ixx as isize + go_right) as usize;
                                current_y = (from_ixy as isize + go_up) as usize;
                                if self.gol.is_valid_index(current_x, current_y) {
                                    self.gol.set_selected_at_index(current_x, current_y);

                                    // draw thickness of line
                                    let thickness = self.gol.config.draw_line_thickness as isize;
                                    if r < u {
                                        // we went right or left so paint with thickness up and down
                                        for y in 1..thickness {
                                            let ty = current_y + y as usize;
                                            if self.gol.is_valid_index(current_x, ty) {
                                                self.gol.set_selected_at_index(current_x, ty)
                                            }
                                            let ty = ((current_y as isize - y).max(0)) as usize;
                                            if self.gol.is_valid_index(current_x, ty) {
                                                self.gol.set_selected_at_index(current_x, ty)
                                            }
                                        }
                                    } else {
                                        // we went up or down, so paint with thickness right and left
                                        for x in 1..thickness {
                                            let tx = current_x + x as usize;
                                            if self.gol.is_valid_index(tx, current_y) {
                                                self.gol.set_selected_at_index(tx, current_y)
                                            }
                                            let tx = ((current_x as isize - x).max(0)) as usize;
                                            if self.gol.is_valid_index(tx, current_y) {
                                                self.gol.set_selected_at_index(tx, current_y)
                                            }
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    None => self.gol.set_selected_at_index(ixx, ixy),
                }
                self.gol.config.bupdate = true;
                self.last_draw_index = Some((ixx, ixy));
            }
        } else {
            self.gol.hover_ix = None;
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, dx: f32, dy: f32) {
        self.egui_mini.mouse_wheel_event(dx, dy);
    }

    fn char_event(
        &mut self,
        _ctx: &mut Context,
        character: char,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        self.egui_mini.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        _repeat: bool,
    ) {
        self.egui_mini.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        self.egui_mini.key_up_event(keycode, keymods);
    }

    fn draw(&mut self, ctx: &mut Context) {
        self.gol.inst = Instant::now();
        ctx.set_blend(
            Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            None,
        );
        ctx.begin_default_pass(Default::default());

        ctx.apply_pipeline(&self.pipeline);
        for binding in &self.bindings {
            ctx.apply_bindings(binding);

            ctx.draw(0, 6 * self.gol.get_fields().width() as i32, 1);
        }
        ctx.end_render_pass();
        self.egui_mini.run(ctx, |_mq_ctx, egui_ctx| {
            self.gol.ui(egui_ctx);
        });
        self.egui_mini.draw(ctx);
        ctx.commit_frame();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.gol.frame_time = (self.gol.inst.elapsed().as_micros() as f64) * 0.001;
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.gol.frame_time = self.gol.inst.elapsed().as_micros() as f64;
        }
    }
}

pub fn mini_main() {
    miniquad::start(conf::Conf::default(), |ctx| Box::new(Stage::new(ctx)));
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec4 color;
    varying lowp vec4 fragment_color;
    void main() {
        gl_Position = vec4(pos.x, -pos.y, 0, 1);
        fragment_color = color;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 fragment_color;
    void main() {
        gl_FragColor = fragment_color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout { uniforms: vec![] },
        }
    }
}
