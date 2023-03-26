use miniquad::Context;

use crate::render_mini::Vec2;

pub struct ZoomWindow {
    pub midpoint: Vec2,
    pub horizontal: f32,
    pub vertical: f32,
}

impl Default for ZoomWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoomWindow {
    pub fn new() -> ZoomWindow {
        ZoomWindow {
            midpoint: Vec2 { x: 0.0, y: 0.0 },
            horizontal: 1.0,
            vertical: 1.0,
        }
    }

    pub fn zoom(&mut self, factor: f32) {
        assert_ne!(factor, 0.0);
        self.horizontal *= factor;
        self.vertical *= factor;
    }

    pub fn pan_x(&mut self, x: f32) {
        self.midpoint.x += x / self.horizontal;
    }

    pub fn pan_y(&mut self, y: f32) {
        self.midpoint.y += y / self.vertical;
    }

    pub fn reset(&mut self) {
        self.midpoint = Vec2 { x: 0.0, y: 0.0 };
        self.vertical = 1.0;
        self.horizontal = 1.0;
    }

    pub fn mouse_to_screen(&self, ctx: &mut Context, mouse_x: f32, mouse_y: f32) -> (f32, f32) {
        let (width, height) = ctx.screen_size();
        let viewport_corner_x = ((self.midpoint.x - 0.5) * self.horizontal + 0.5) * width;
        let viewport_corner_y = ((self.midpoint.y - 0.5) * self.vertical + 0.5) * height;
        let x = (mouse_x - viewport_corner_x) / self.horizontal;
        let y = (mouse_y - viewport_corner_y) / self.vertical;
        (x, y)
    }

    pub fn set_window(&self, ctx: &mut Context) {
        let (width, height) = ctx.screen_size();
        let viewport_corner_x = ((-self.midpoint.x - 0.5) * self.horizontal + 0.5) * width;
        let viewport_corner_y = ((-self.midpoint.y - 0.5) * self.vertical + 0.5) * height;
        let viewport_width = width * self.horizontal;
        let viewport_height = height * self.vertical;
        ctx.apply_viewport(
            viewport_corner_x as i32,
            viewport_corner_y as i32,
            viewport_width as i32,
            viewport_height as i32,
        );
    }
}
