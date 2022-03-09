use super::{
    traits::{ConvolutionT, Matrix},
    vec_matrix::VecMatrix,
};
use crate::CellType;
use macroquad::rand::gen_range;
use std::iter::repeat;

#[derive(Clone, Debug)]
pub struct MatrixPacked {
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
                        *tix += gen_range(0_u8, 1) as u64;
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

    fn new_random_range(
        width: usize,
        height: usize,
        range: std::ops::RangeInclusive<Self::Output>,
    ) -> Self {
        let tiles_x = 1 + (width - 1) / 8;
        let tiles_y = 1 + (height - 1) / 8;
        let mut tiles = Vec::new();
        for _ in 0..tiles_x {
            for _ in 0..tiles_y {
                let mut tile = [0; 8];
                for tix in &mut tile {
                    for _ in 0..8 {
                        *tix <<= 8;
                        *tix += gen_range(*range.start(), *range.end()) as u64;
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

    // FIXME: buggy
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
impl<Conv: Matrix<Output = u8>> ConvolutionT<Conv, u8> for MatrixPacked {
    fn convolution(&mut self, kernels: &[Conv], cell_type_matrix: &VecMatrix<CellType>) {
        let kernel_width = kernels[0].width();
        let fields_old = self.clone();
        for ixy in 0..self.height {
            for ixx in 0..self.width {
                let cut_x: i32 = ixx as i32 - (kernel_width / 2) as i32;
                let cut_y: i32 = ixy as i32 - (kernel_width / 2) as i32;
                let mut acc = 0;
                let kernel_ix = cell_type_matrix.index((ixx, ixy)).as_index();
                for (conv_x, ix_cut_x) in
                    (cut_x.max(0)..(cut_x + kernel_width as i32).min(self.width as i32)).enumerate()
                {
                    for (conv_y, ix_cut_y) in (cut_y.max(0)
                        ..(cut_y + kernel_width as i32).min(self.height as i32))
                        .enumerate()
                    {
                        acc += kernels[kernel_ix].index((conv_x, conv_y))
                            * fields_old.index((ix_cut_x as usize, ix_cut_y as usize));
                    }
                }

                self.set_at_index((ixx, ixy), acc);
            }
        }
    }
}
