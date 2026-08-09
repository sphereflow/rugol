use miniquad::Context;

use crate::{render_mini::Vec2, selection::Selection};

pub struct DragEvent {
    pub mouse_down: bool,
    pub dragging: bool,
    pub drag_move_dist: f32,
    pub drag_start: Option<Vec2>,
    pub drag_end: Option<Vec2>,
}

impl DragEvent {
    pub fn mouse_down(&mut self, mouse_pos: Vec2) {
        self.drag_start = Some(mouse_pos);
        self.drag_end = None;
        self.dragging = false;
        self.mouse_down = true;
    }

    pub fn mouse_up(&mut self, mouse_pos: Vec2) {
        if self.dragging {
            self.drag_end = Some(mouse_pos);
        } else {
            // behave like it was a click
            self.drag_start = None;
            self.drag_end = None;
        }
        self.dragging = false;
        self.mouse_down = false;
    }

    pub fn mouse_move(&mut self, mouse_pos: Vec2) {
        if let Some(start) = self.drag_start {
            if !self.dragging && self.mouse_down {
                let dist_squared =
                    (mouse_pos.x - start.x).powi(2) + (mouse_pos.y - start.y).powi(2);
                let drag_dist_crossed = dist_squared > self.drag_move_dist.powi(2);
                if drag_dist_crossed {
                    self.dragging = true;
                }
            } else if self.dragging {
                self.drag_end = Some(mouse_pos);
            }
        }
    }

    pub fn get_drag(&self) -> Option<(Vec2, Vec2)> {
        if let (Some(s), Some(e)) = (self.drag_start, self.drag_end) {
            Some((s, e))
        } else {
            None
        }
    }

    pub fn as_selection(&self, ctx: &mut Context) -> Option<Selection> {
        if let Some(s) = self.drag_start {
            let mut selection = Selection::new(ctx, s);
            selection.end = self.drag_end;
            Some(selection)
        } else {
            None
        }
    }
}
