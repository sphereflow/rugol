use std::ops::RangeInclusive;

use fixedbitset::{FixedBitSet, Ones};

#[derive(Clone)]
pub struct IndexSet {
    /// indices in sorted order
    indices: FixedBitSet,
    matrix_width: usize,
}

impl IndexSet {
    pub fn new(matrix_width: usize, matrix_height: usize) -> Self {
        IndexSet {
            indices: FixedBitSet::with_capacity(matrix_width * matrix_height),
            matrix_width,
        }
    }

    pub fn insert(&mut self, (ixx, ixy): (usize, usize)) {
        // if the item is not already in the set of indices -> insert it
        self.indices.set(ixy * self.matrix_width + ixx, true);
    }

    pub fn insert_rect(&mut self, range: &RangeInclusive<(usize, usize)>) {
        let (x_start, x_end) = range.start();
        let (y_start, y_end) = range.end();
        for ixy in *y_start..(y_end + 1) {
            let base_ix = ixy * self.matrix_width;
            self.indices
                .set_range((base_ix + x_start)..(base_ix + x_end + 1), true);
        }
    }

    pub fn contains(&self, (ixx, ixy): (usize, usize)) -> bool {
        self.indices[ixx + ixy * self.matrix_width]
    }

    pub fn iter(&self) -> IndexSetIter {
        IndexSetIter {
            iter: self.indices.ones(),
            matrix_width: self.matrix_width,
        }
    }
}

pub struct IndexSetIter<'a> {
    iter: Ones<'a>,
    matrix_width: usize,
}

impl Iterator for IndexSetIter<'_> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(raw_index) => Some((raw_index % self.matrix_width, raw_index / self.matrix_width)),
            None => None,
        }
    }
}
