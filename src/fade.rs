use crate::matrix::traits::Matrix;
use std::collections::VecDeque;

pub struct Fader<M> {
    values: VecDeque<M>,
}

impl<M: Matrix> Fader<M> {}
