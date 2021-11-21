use ::egui_tetra::egui;
use egui::{Button, Layout, Slider, Ui, Window};
use std::{iter::repeat, ops::RangeInclusive};
use tetra::{
    graphics::{
        self,
        mesh::{BorderRadii, GeometryBuilder, ShapeStyle},
        Color, DrawParams, Rectangle,
    },
    Context, ContextBuilder, State,
};

const WIDTH: i32 = 1280;
const HEIGHT: i32 = 720;

type RState = RugolState<ConstMatrix<u8, 100, 80>, MatrixPacked>;

struct Rule {
    state: u8,
    range: RangeInclusive<u8>,
    transition: u8,
}

struct RugolState<M: Matrix, C: Matrix> {
    width: usize,
    height: usize,
    conv_matrix: C,
    rules: Vec<Rule>,
    fields: M,
    timer: u64,
    wait: u64,
}

impl RState {
    fn new(_context: &mut Context) -> Result<RState, Box<dyn std::error::Error>> {
        let width = 100;
        let height = 80;
        let mut conv_matrix = MatrixPacked::new_std_conv_matrix(3, 3);
        conv_matrix.set_at_index((0, 0), 2);
        conv_matrix.set_at_index((2, 2), 0);
        Ok(RugolState {
            width,
            height,
            conv_matrix,
            rules: classic_rules(),
            fields: ConstMatrix::new_random(width, height),
            timer: 0,
            wait: 1,
        })
    }

    fn step(&mut self) {
        let mut res = ConstMatrix::new(self.width, self.height);
        for ixx in 0..self.width {
            for ixy in 0..self.height {
                let conv = self.convolution(ixx, ixy);
                res.set_at_index((ixx, ixy), self.apply_rules(ixx, ixy, conv));
            }
        }
        self.fields = res;
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
        let cut_x: i32 = ixx as i32 - (self.conv_matrix.width / 2) as i32;
        let cut_y: i32 = ixy as i32 - (self.conv_matrix.height / 2) as i32;
        let mut acc = 0;
        for (conv_x, ix_cut_x) in (cut_x.max(0)
            ..(cut_x + self.conv_matrix.width as i32).min(self.fields.width() as i32))
            .enumerate()
        {
            for (conv_y, ix_cut_y) in (cut_y.max(0)
                ..(cut_y + self.conv_matrix.height as i32).min(self.fields.height() as i32))
                .enumerate()
            {
                acc += self.conv_matrix.index((conv_x, conv_y))
                    * self.fields.index((ix_cut_x as usize, ix_cut_y as usize));
            }
        }

        acc
    }

    fn apply_rules(&mut self, ixx: usize, ixy: usize, convolution: u8) -> u8 {
        let field = self.fields.index((ixx, ixy));
        for rule in &self.rules {
            if rule.state == field && rule.range.contains(&convolution) {
                return rule.transition;
            }
        }

        field
    }

    fn edit_rules_ui(&mut self, ui: &mut Ui) {
        if ui.add(Button::new("Add rule")).clicked() {
            self.rules.push(Rule {
                state: 0,
                range: 0..=0,
                transition: 0,
            });
        }
        let mut o_delete_ix = None;
        for (del_ix, rule) in self.rules.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(Slider::new(&mut rule.state, 0..=7));
                ui.label("->");
                RugolState::edit_range(ui);
                ui.label("->");
                ui.add(Slider::new(&mut rule.transition, 0..=7));
                if ui.add(Button::new("Delete rule")).clicked() {
                    o_delete_ix = Some(del_ix);
                }
            });
        }
        if let Some(del_ix) = o_delete_ix {
            self.rules.remove(del_ix);
        }
    }

    fn edit_range(ui: &mut Ui) {}
}

impl<M: Matrix, C: Matrix> State for RugolState<M, C> {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ContextBuilder::new("Rugol", WIDTH, HEIGHT)
        .show_mouse(true)
        .quit_on_escape(true)
        .build()?
        .run(|ctx| Ok(egui_tetra::StateWrapper::new(RugolState::new(ctx)?)))
}

fn classic_rules() -> Vec<Rule> {
    vec![
        Rule {
            state: 0,
            range: 0..=0,
            transition: 1,
        },
        Rule {
            state: 1,
            range: 0..=1,
            transition: 0,
        },
        Rule {
            state: 1,
            range: 5..=8,
            transition: 0,
        },
        Rule {
            state: 0,
            range: 3..=3,
            transition: 1,
        },
    ]
}

trait Matrix {
    type Output;
    fn new(width: usize, height: usize) -> Self;
    fn new_random(width: usize, height: usize) -> Self;
    fn new_std_conv_matrix(width: usize, height: usize) -> Self;
    fn index(&self, ix: (usize, usize)) -> Self::Output;
    fn set_at_index(&mut self, ix: (usize, usize), value: Self::Output);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

struct MatrixPacked {
    tiles: Vec<[u64; 8]>,
    width: usize,
    height: usize,
}

impl Matrix for MatrixPacked {
    type Output = u8;

    fn new(width: usize, height: usize) -> MatrixPacked {
        let tiles_x = 1 + (width - 1) / 8;
        let tiles_y = 1 + (height - 1) / 8;
        let tiles = Vec::from_iter(repeat([0; 8]).take(tiles_x * tiles_y));
        MatrixPacked {
            tiles,
            width,
            height,
        }
    }

    fn new_random(width: usize, height: usize) -> MatrixPacked {
        let tiles_x = 1 + (width - 1) / 8;
        let tiles_y = 1 + (height - 1) / 8;
        let mut tiles = Vec::new();
        for _ in 0..tiles_x {
            for _ in 0..tiles_y {
                let mut tile = [0; 8];
                for tix in &mut tile {
                    for _ in 0..8 {
                        *tix <<= 8;
                        *tix += (rand::random::<u8>() / 128) as u64;
                    }
                }
                tiles.push(tile);
            }
        }
        MatrixPacked {
            tiles,
            width,
            height,
        }
    }

    fn new_std_conv_matrix(width: usize, height: usize) -> MatrixPacked {
        let rep: u64 = u64::from_be_bytes([1; 8]);
        let mut tiles = Vec::from_iter(repeat([rep; 8]).take((width * height) as usize));

        // set the middle elemnt to 0
        let ix_tile_x = ((width / 2).max(1) - 1) / 8;
        let ix_tile_y = ((height / 2).max(1) - 1) / 8;
        let num_tiles_y = (height - 1) / 8;
        let mask: u64 = 0xFF << ((width / 2) % 8);
        let v_mask = 1 << ((width / 2) % 8);
        let tile_ix = ix_tile_x + ix_tile_y * num_tiles_y;
        tiles[tile_ix][(height / 2) % 8] &= !mask;
        tiles[tile_ix][(height / 2) % 8] |= v_mask;
        MatrixPacked {
            tiles,
            width,
            height,
        }
    }

    fn index(&self, (ixx, ixy): (usize, usize)) -> Self::Output {
        let ix_tile_x = (ixx.max(1) - 1) / 8;
        let ix_tile_y = (ixy.max(1) - 1) / 8;
        let num_tiles_y = (self.height - 1) / 8;
        (self.tiles[ix_tile_x + ix_tile_y * num_tiles_y][ixy % 8].to_be_bytes())[ixx % 8]
    }

    fn set_at_index(&mut self, (ixx, ixy): (usize, usize), value: u8) {
        let ix_tile_x = (ixx.max(1) - 1) / 8;
        let ix_tile_y = (ixy.max(1) - 1) / 8;
        let num_tiles_y = (self.height - 1) / 8;
        let mask: u64 = 0xFF << (8 * (ixx % 8));
        let v_mask = (value as u64) << (8 * (ixx % 8));
        self.tiles[ix_tile_x + ix_tile_y * num_tiles_y][ixy % 8] &= !mask;
        self.tiles[ix_tile_x + ix_tile_y * num_tiles_y][ixy % 8] |= v_mask;
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

struct ConstMatrix<T: Copy, const M: usize, const N: usize> {
    data: [[T; N]; M],
}

impl<const M: usize, const N: usize> Matrix for ConstMatrix<u8, M, N> {
    fn new(_width: usize, _height: usize) -> ConstMatrix<u8, M, N> {
        ConstMatrix { data: [[0; N]; M] }
    }

    fn new_std_conv_matrix(_width: usize, _height: usize) -> ConstMatrix<u8, M, N> {
        let mut data = [[1; N]; M];
        data[N / 2][M / 2] = 0;
        ConstMatrix { data }
    }

    fn new_random(_width: usize, _height: usize) -> ConstMatrix<u8, M, N> {
        let mut data = [[0; N]; M];
        for x in 0..M {
            for y in 0..N {
                data[x][y] = rand::random::<u8>() / 128;
            }
        }
        ConstMatrix { data }
    }

    type Output = u8;

    fn index(&self, (x, y): (usize, usize)) -> Self::Output {
        self.data[x][y]
    }

    fn set_at_index(&mut self, (x, y): (usize, usize), value: Self::Output) {
        self.data[x][y] = value;
    }

    fn width(&self) -> usize {
        M
    }

    fn height(&self) -> usize {
        N
    }
}

impl egui_tetra::State<Box<dyn std::error::Error>> for RState {
    fn ui(
        &mut self,
        _ctx: &mut tetra::Context,
        egui_context: &egui_tetra::egui::CtxRef,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Window::new("Rugol").show(egui_context, |ui| {
            ui.label("yo");
            self.edit_rules_ui(ui);
        });
        Ok(())
    }

    fn draw(
        &mut self,
        ctx: &mut Context,
        _egui_context: &egui::CtxRef,
    ) -> Result<(), Box<(dyn std::error::Error)>> {
        graphics::clear(ctx, Color::BLACK);
        for ixx in 0..self.width {
            for ixy in 0..self.height {
                let x = (ixx as f32 * WIDTH as f32) / (self.width as f32);
                let y = (ixy as f32 * HEIGHT as f32) / (self.height as f32);
                let mut geo_builder = GeometryBuilder::new();
                geo_builder.rounded_rectangle(
                    ShapeStyle::Fill,
                    Rectangle {
                        x,
                        y,
                        width: WIDTH as f32 / (self.width as f32),
                        height: HEIGHT as f32 / (self.height as f32),
                    },
                    BorderRadii::new(4.0),
                )?;
                let mesh = geo_builder.build_mesh(ctx)?;
                if self.fields.index((ixx, ixy)) == 0 {
                    mesh.draw(ctx, DrawParams::new().color(Color::rgb8(80, 20, 20)));
                } else {
                    mesh.draw(ctx, DrawParams::new().color(Color::rgb8(40, 128, 70)));
                }
            }
        }
        Ok(())
    }

    fn update(
        &mut self,
        _context: &mut Context,
        _egui_context: &egui::CtxRef,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.timer += 1;
        if (self.timer % self.wait) == 0 {
            self.step();
        }
        Ok(())
    }
}
