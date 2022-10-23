use std::{iter::repeat, ops::RangeInclusive};

#[derive(Clone, Debug)]
pub struct QuadTree<T> {
    tree: Vec<Vec<T>>,
    // width and height of the field
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Changed {
    Yes,
    No,
    Partial,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    changed: Changed,
    x_range: RangeInclusive<usize>,
    y_range: RangeInclusive<usize>,
}

impl QuadTree<Node> {
    pub fn new(width: usize, height: usize, levels: usize) -> Self {
        let mut tree = Vec::new();
        let init_node = Node {
            changed: Changed::Yes,
            x_range: 0..=0,
            y_range: 0..=0,
        };
        for level in 0..levels {
            tree.push(Vec::from_iter(
                repeat(init_node.clone()).take(4_usize.pow(level as u32)),
            ));
        }
        let mut res = QuadTree {
            tree,
            width,
            height,
        };
        res.insert_ranges(0, 0, 0..=(width - 1), 0..=(height - 1));
        res
    }

    pub fn insert_ranges(
        &mut self,
        level: usize,
        node_ix: usize,
        x_range: RangeInclusive<usize>,
        y_range: RangeInclusive<usize>,
    ) {
        let x_start = *x_range.start();
        let y_start = *y_range.start();
        let x_end = *x_range.end();
        let y_end = *y_range.end();
        if x_start <= x_end && y_start <= y_end {
            let xh = x_start + (x_end - x_start) / 2;
            let yh = y_start + (y_end - y_start) / 2;
            self.tree[level][node_ix].x_range = x_range;
            self.tree[level][node_ix].y_range = y_range;
            if level < (self.tree.len() - 1) {
                self.insert_ranges(level + 1, node_ix * 4, x_start..=xh, y_start..=yh);
                self.insert_ranges(level + 1, node_ix * 4 + 1, (xh + 1)..=x_end, y_start..=yh);
                self.insert_ranges(level + 1, node_ix * 4 + 2, x_start..=xh, (yh + 1)..=y_end);
                self.insert_ranges(
                    level + 1,
                    node_ix * 4 + 3,
                    (xh + 1)..=x_end,
                    (yh + 1)..=y_end,
                );
            }
        }
    }

    pub fn clear(&mut self) {
        for level in self.tree.iter_mut() {
            for node in level.iter_mut() {
                node.changed = Changed::No;
            }
        }
    }

    pub fn everything_changed(&mut self) {
        for level in self.tree.iter_mut() {
            for node in level.iter_mut() {
                node.changed = Changed::Yes;
            }
        }
    }

    pub fn insert(&mut self, x: usize, y: usize, level: usize, node_ix: usize) {
        match &self.tree[level][node_ix].changed {
            Changed::Yes => {}
            Changed::No | Changed::Partial => {
                // check range
                if self.tree[level][node_ix].x_range.contains(&x)
                    && self.tree[level][node_ix].y_range.contains(&y)
                {
                    if level < (self.tree.len() - 1) {
                        // recursive call
                        self.insert(x, y, level + 1, node_ix * 4);
                        self.insert(x, y, level + 1, node_ix * 4 + 1);
                        self.insert(x, y, level + 1, node_ix * 4 + 2);
                        self.insert(x, y, level + 1, node_ix * 4 + 3);
                        // check if transition is required
                        match (
                            self.tree[level + 1][node_ix * 4].changed,
                            self.tree[level + 1][node_ix * 4 + 1].changed,
                            self.tree[level + 1][node_ix * 4 + 2].changed,
                            self.tree[level + 1][node_ix * 4 + 3].changed,
                        ) {
                            (Changed::Yes, Changed::Yes, Changed::Yes, Changed::Yes) => {
                                self.tree[level][node_ix].changed = Changed::Yes;
                            }
                            (Changed::No, Changed::No, Changed::No, Changed::No) => {
                                self.tree[level][node_ix].changed = Changed::No;
                            }
                            _ => self.tree[level][node_ix].changed = Changed::Partial,
                        }
                    } else {
                        // leaf node
                        self.tree[level][node_ix].changed = Changed::Yes;
                    }
                }
            }
        }
    }

    pub fn parent_ix(&self, level: usize, node_ix: usize) -> (usize, usize) {
        (level - 1, node_ix / 4)
    }

    pub fn get_changed_ranges(
        &self,
        convolution_width: usize,
        level: usize,
        node_ix: usize,
        res: &mut Vec<RangeInclusive<(usize, usize)>>,
    ) {
        let wh = convolution_width / 2;
        let node = &self.tree[level][node_ix];
        match node.changed {
            Changed::Yes => {
                let x_start = clamp_start_index(*node.x_range.start(), wh);
                let y_start = clamp_start_index(*node.y_range.start(), wh);
                let x_end = (node.x_range.end() + wh).min(self.width - 1);
                let y_end = (node.y_range.end() + wh).min(self.height - 1);

                res.push((x_start, x_end)..=(y_start, y_end));
            }
            Changed::No => {}
            Changed::Partial => {
                if level >= (self.tree.len() - 1) {
                    panic!("Partial leaf");
                }
                self.get_changed_ranges(convolution_width, level + 1, node_ix * 4, res);
                self.get_changed_ranges(convolution_width, level + 1, node_ix * 4 + 1, res);
                self.get_changed_ranges(convolution_width, level + 1, node_ix * 4 + 2, res);
                self.get_changed_ranges(convolution_width, level + 1, node_ix * 4 + 3, res);
            }
        }
    }

    pub fn print_levels(&self) {
        for level in 0..self.tree.len() {
            let lines = self.print_level_rec(level, 0, 0);
            for line in &lines {
                println!("{line}");
            }
            println!("---");
        }
    }

    fn print_level_rec(&self, target_level: usize, level: usize, node_ix: usize) -> Vec<String> {
        if level == target_level {
            match self.tree[level][node_ix].changed {
                Changed::Yes => vec!["X".to_string()],
                Changed::No => vec!["_".to_string()],
                Changed::Partial => vec!["*".to_string()],
            }
        } else {
            let mut tl = self.print_level_rec(target_level, level + 1, node_ix * 4);
            let tr = self.print_level_rec(target_level, level + 1, node_ix * 4 + 1);
            let mut bl = self.print_level_rec(target_level, level + 1, node_ix * 4 + 2);
            let br = self.print_level_rec(target_level, level + 1, node_ix * 4 + 3);
            let top = tl.iter_mut().zip(tr.iter()).map(|(start, end)| {
                start.push_str(end);
                start.clone()
            });
            let bottom = bl.iter_mut().zip(br.iter()).map(|(start, end)| {
                start.push_str(end);
                start.clone()
            });
            let mut res = top.collect::<Vec<String>>();
            res.extend(bottom);
            res
        }
    }
}

fn clamp_start_index(ix: usize, wh: usize) -> usize {
    if ix < wh {
        0
    } else {
        ix - wh
    }
}
