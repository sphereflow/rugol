use std::ops::RangeInclusive;

use instant::Instant;

use super::*;

use crate::{
    app_config::AppConfig,
    cell_type::{CellType, CellTypeMap},
    fade::Fader,
    matrix::{
        const_matrix::ConstMatrix, convolution::Convolution, traits::Matrix, vec_matrix::VecMatrix,
    },
    quad_tree::QuadTree,
    rules::classic_rules,
    BaseMatrix, FieldType, RState, CELLS,
};

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
                let mut field_type_matrix = Convolution::new(cw, ch);
                let cell_type_matrix: VecMatrix<CellType> = VecMatrix::new_random(cw, ch);
                for x in 0..cw {
                    for y in 0..ch {
                        field_type_matrix
                            .set_at_index((x, y), cell_type_map[cell_type_matrix.index((x, y))].1);
                    }
                }
                f.push(field_type_matrix);
                ct.push(cell_type_matrix);
                acc.push(VecMatrix::new(cw, ch));
            }
            (f, ct, acc)
        };
        RugolState {
            conv_kernels: [conv_matrix; 9],
            cell_type_map,
            rules: classic_rules(),
            fields_vec,
            cell_type_vec,
            acc_vec,
            vec_ix: fields_vec_ix,
            fader: Fader::new(CELLS[fields_vec_ix].0, CELLS[fields_vec_ix].1),
            config: AppConfig::default(),
            quad_tree: QuadTree::new(CELLS[fields_vec_ix].0, CELLS[fields_vec_ix].1, 5),
            inst: Instant::now(),
            frame_time: 0.,
        }
    }

    pub fn step(&mut self) {
        self.config.bupdate = true;
        self.config.tick = Instant::now();
        let field_type_matrix = &mut self.fields_vec[self.vec_ix];
        let cell_type_matrix = &mut self.cell_type_vec[self.vec_ix];
        let acc_matrix = &mut self.acc_vec[self.vec_ix];
        let indices = {
            let mut res = Vec::new();
            let mut range_vec = Vec::new();
            self.quad_tree.get_changed_ranges(CW, 0, 0, &mut range_vec);
            //dbg!(&range_vec);
            for range in range_vec.iter_mut() {
                let (x_start, x_end) = *range.start();
                let (y_start, y_end) = *range.end();
                for x in x_start..=x_end {
                    for y in y_start..=y_end {
                        res.push((x, y));
                    }
                }
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
        self.quad_tree.clear();
        // map the accumulated values to the cell matrix
        // field_type_matrix -> self.rules.apply(...) -> self.cell_type_vec[self.vec_ix]
        // self.cell_type_vec[self.vec_ix] -> self.map.lookup(...) -> self.fields_vec[self.vec_ix]
        for (ixx, ixy) in indices {
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
}
