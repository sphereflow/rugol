use std::ops::RangeInclusive;

use egui_dock::Tree;
use instant::Instant;

use super::*;
use crate::{
    app_config::AppConfig,
    cell_type::{CellType, CellTypeMap},
    convolution::Convolution,
    fade::Fader,
    index_set::IndexSet,
    quad_tree::QuadTree,
    rules::classic_rules,
    save_file::ConvMatrixE,
    traits::ConvolutionT,
    BaseMatrix, FieldType, RState, CELLS,
};
use matrices::traits::*;
use matrices::{const_matrix::ConstMatrix, vec_matrix::VecMatrix};

impl<const CW: usize> RState<CW> {
    pub fn new() -> Self {
        let conv_matrix = ConstMatrix::new_std_conv_matrix(3, 3);
        let cell_type_map = CellTypeMap::new();
        let fields_vec_ix = 0;
        let (fields_vec, cell_type_vec, acc_vec) = {
            let mut f = Vec::new();
            let mut ct = Vec::new();
            let mut acc = Vec::new();
            for (cw, ch) in CELLS {
                let mut field_type_matrix = Convolution::new(cw, ch, 0.);
                let cell_type_matrix: VecMatrix<CellType> = VecMatrix::new_random(cw, ch);
                for x in 0..cw {
                    for y in 0..ch {
                        field_type_matrix
                            .set_at_index((x, y), cell_type_map[cell_type_matrix.index((x, y))].1);
                    }
                }
                f.push(field_type_matrix);
                ct.push(cell_type_matrix);
                acc.push(VecMatrix::new(cw, ch, 0.));
            }
            (f, ct, acc)
        };
        let mut tabs = Vec::new();
        for i in 0..9 {
            tabs.push(i);
        }
        let tree = Tree::new(tabs);
        RugolState {
            conv_kernels: [conv_matrix; 9],
            cell_type_map,
            rules: classic_rules(),
            fields_vec,
            cell_type_vec,
            acc_vec,
            vec_ix: fields_vec_ix,
            hover_ix: None,
            fader: Fader::new(CELLS[fields_vec_ix].0, CELLS[fields_vec_ix].1),
            config: AppConfig::default(),
            quad_tree: QuadTree::new(CELLS[fields_vec_ix].0, CELLS[fields_vec_ix].1, 5),
            inst: Instant::now(),
            frame_time: 0.,
            save_file: None,
            tree,
        }
    }

    pub fn step(&mut self) {
        self.config.bupdate = true;
        self.config.tick = Instant::now();
        let field_type_matrix = &mut self.fields_vec[self.vec_ix];
        let cell_type_matrix = &mut self.cell_type_vec[self.vec_ix];
        let acc_matrix = &mut self.acc_vec[self.vec_ix];
        let indices = {
            let mut res = IndexSet::new(acc_matrix.width(), acc_matrix.height());
            let mut range_vec = Vec::new();
            self.quad_tree.get_changed_ranges(CW, 0, 0, &mut range_vec);
            // dbg!(&range_vec);
            for range in range_vec.iter() {
                res.insert_rect(range);
            }
            res
        };
        field_type_matrix.convolution(
            &self.conv_kernels,
            self.config.bsingle_kernel,
            cell_type_matrix,
            acc_matrix,
            &indices,
        );
        // dbg!(&indices);
        // self.quad_tree.print_levels();
        // println!("Acc:\n{acc_matrix}");
        self.quad_tree.clear();
        // map the accumulated values to the cell matrix
        // field_type_matrix -> self.rules.apply(...) -> self.cell_type_vec[self.vec_ix]
        // self.cell_type_vec[self.vec_ix] -> self.map.lookup(...) -> self.fields_vec[self.vec_ix]
        // stores indices that have already had rules applied to them
        for (ixx, ixy) in indices.iter() {
            let acc = acc_matrix.index((ixx, ixy));
            let initial_cell = cell_type_matrix.index((ixx, ixy));
            let cell = self.rules.apply(initial_cell, acc);
            let field = self.cell_type_map[cell].1;
            if cell != initial_cell {
                cell_type_matrix.set_at_index((ixx, ixy), cell);
                field_type_matrix.set_at_index((ixx, ixy), field);
                self.quad_tree.insert(ixx, ixy, 0, 0);
            }
        }
        if self.config.bfade {
            self.fader
                .add(&self.cell_type_vec[self.vec_ix], &self.cell_type_map);
        }
        self.config.elapsed = self.config.tick.elapsed();
    }

    pub fn randomize(&mut self, range: RangeInclusive<CellType>) {
        let fields = &mut self.fields_vec[self.vec_ix];
        let cells = &mut self.cell_type_vec[self.vec_ix];
        let w = fields.width();
        let h = fields.height();
        *cells = VecMatrix::new_random_range(w, h, range);
        self.quad_tree.everything_changed();
        for x in 0..w {
            for y in 0..h {
                fields.set_at_index((x, y), self.cell_type_map[cells.index((x, y))].1);
            }
        }
        self.config.bupdate = true;
    }

    pub fn load_save_file(&mut self) {
        if let Some(save_file) = self.save_file.as_mut() {
            if let (Some(convolution), true) =
                (save_file.convolution.take(), save_file.include_convolution)
            {
                match convolution {
                    ConvMatrixE::Single(conv) => self.conv_kernels[0] = conv,
                    ConvMatrixE::Multiple(convs) => self.conv_kernels = convs,
                }
            }
            if let (Some(rules), true) = (save_file.rules.take(), save_file.include_rules) {
                self.rules = rules;
            }
            if let (Some(map), true) = (
                save_file.cell_type_map.take(),
                save_file.include_cell_type_map,
            ) {
                self.cell_type_map = map;
            }
            if let (Some(cells), true) = (save_file.cells.take(), save_file.include_cells) {
                self.cell_type_vec = cells;
            }
            for (fields, cells) in self
                .fields_vec
                .iter_mut()
                .zip(self.cell_type_vec.iter_mut())
            {
                self.quad_tree.everything_changed();
                let w = fields.width();
                let h = fields.height();
                for x in 0..w {
                    for y in 0..h {
                        fields.set_at_index((x, y), self.cell_type_map[cells.index((x, y))].1);
                    }
                }
            }
            self.config.bupdate = true;
            self.save_file = None;
        }
    }

    pub fn donut_all_kernels(&mut self, range: RangeInclusive<usize>, val: FieldType) {
        for kernel in self.conv_kernels.iter_mut() {
            kernel.donut(range.clone(), val);
        }
    }

    pub fn clear(&mut self) {
        let fields = &mut self.fields_vec[self.vec_ix];
        let cells = &mut self.cell_type_vec[self.vec_ix];
        let field_value = self.cell_type_map[self.config.clear_val].1;
        cells.clear(self.config.clear_val);
        fields.clear(field_value);
        self.quad_tree.everything_changed();
        self.config.bupdate = true;
    }

    pub fn value_changed_for(&mut self, cell_type: CellType) {
        let cells = &self.cell_type_vec[self.vec_ix];
        for ixx in 0..cells.width() {
            for ixy in 0..cells.height() {
                if cells.index((ixx, ixy)) == cell_type {
                    self.quad_tree.insert(ixx, ixy, 0, 0);
                }
            }
        }
        self.config.bupdate = true;
    }

    pub fn everything_changed(&mut self) {
        self.quad_tree.everything_changed();
        self.config.bupdate = true;
    }

    pub fn get_fields(&self) -> &BaseMatrix<CW> {
        &self.fields_vec[self.vec_ix]
    }

    pub fn get_cells(&self) -> &VecMatrix<CellType> {
        &self.cell_type_vec[self.vec_ix]
    }

    pub fn set_at_index(&mut self, ixx: usize, ixy: usize, cell: &CellType) {
        self.fields_vec[self.vec_ix].set_at_index((ixx, ixy), self.cell_type_map[*cell].1);
        self.cell_type_vec[self.vec_ix].set_at_index((ixx, ixy), *cell);
        self.quad_tree.insert(ixx, ixy, 0, 0);
    }

    pub fn set_selected_at_index(&mut self, ixx: usize, ixy: usize) {
        self.fields_vec[self.vec_ix]
            .set_at_index((ixx, ixy), self.cell_type_map.get_selected_rules_val());
        self.cell_type_vec[self.vec_ix]
            .set_at_index((ixx, ixy), self.cell_type_map.get_selected_rules_cell());
        self.quad_tree.insert(ixx, ixy, 0, 0);
    }

    pub fn is_valid_index(&self, ixx: usize, ixy: usize) -> bool {
        (0..self.fields_vec[self.vec_ix].width()).contains(&ixx)
            && (0..self.fields_vec[self.vec_ix].height()).contains(&ixy)
    }
}
