use egui_miniquad::EguiMq;
use instant::Instant;
use miniquad::*;

use crate::{fade::Fader, matrix::traits::Matrix, quad_tree::QuadTree, RState, CELLS};

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
    gol: RState<7>,
    bdraw: bool,
}

impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
        let index_buffer = Buffer::immutable::<u16>(ctx, BufferType::IndexBuffer, &vec![]);
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
        let mut gol = <RState<7>>::new();
        gol.donut_all_kernels(0..=1, 0);
        let mut res = Stage {
            pipeline,
            bindings,
            vertices: vec![],
            vertex_buffers: vec![],
            index_buffer,
            gol,
            bdraw: false,
            egui_mini: EguiMq::new(ctx),
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
}

impl EventHandler for Stage {
    fn update(&mut self, ctx: &mut Context) {
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
            let (win_width, win_height) = ctx.screen_size();
            let (field_width, field_height) = (
                self.gol.get_fields().width() as f32,
                self.gol.get_fields().height() as f32,
            );
            let ixx = (field_width * x_pos / win_width) as usize;
            let ixy = (field_height * y_pos / win_height) as usize;
            let val = self.gol.cell_type_map.get_selected_rules_val();
            let cell = self.gol.cell_type_map.get_selected_rules_cell();
            self.gol.fields_vec[self.gol.vec_ix].set_at_index((ixx, ixy), val);
            self.gol.cell_type_vec[self.gol.vec_ix].set_at_index((ixx, ixy), cell);
            self.gol.quad_tree.insert(ixx, ixy, 0, 0);
            self.gol.config.bupdate = true;
            self.bdraw = true;
        }
        self.egui_mini
            .mouse_button_down_event(ctx, button, x_pos, y_pos);
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.bdraw = false;
        }
        self.egui_mini.mouse_button_up_event(ctx, button, x, y);
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        self.egui_mini.mouse_motion_event(ctx, x, y);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, dx: f32, dy: f32) {
        self.egui_mini.mouse_wheel_event(ctx, dx, dy);
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
            ctx.apply_bindings(&binding);

            ctx.draw(0, 6 * self.gol.get_fields().width() as i32, 1);
        }
        ctx.end_render_pass();
        self.egui_mini.run(ctx, |egui_ctx| {
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
    miniquad::start(conf::Conf::default(), |mut ctx| {
        UserData::owning(Stage::new(&mut ctx), ctx)
    });
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
