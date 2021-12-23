use super::*;

#[derive(Debug, Clone)]
pub struct VecMatrix<T: Copy + Clone> {
    pub data: Vec<T>,
    width: usize,
    height: usize,
}

impl Matrix for VecMatrix<u8> {
    type Output = u8;
    fn new(width: usize, height: usize) -> VecMatrix<u8> {
        VecMatrix {
            data: vec![0; width * height],
            width,
            height,
        }
    }

    fn new_std_conv_matrix(width: usize, height: usize) -> Self {
        let mut data = vec![1; width * height];
        data[width / 2 + (height / 2) * width] = 0;
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn new_random(width: usize, height: usize) -> Self {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            data.push(gen_range::<u8>(0, 2));
        }
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn index(&self, (ixx, ixy): (usize, usize)) -> u8 {
        self.data[ixx + ixy * self.width]
    }

    fn set_at_index(&mut self, (ixx, ixy): (usize, usize), value: u8) {
        self.data[ixx + ixy * self.width] = value;
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl Matrix for VecMatrix<[f32; 4]> {
    type Output = [f32; 4];
    fn new(width: usize, height: usize) -> Self {
        VecMatrix {
            data: vec![[0.; 4]; width * height],
            width,
            height,
        }
    }

    fn new_std_conv_matrix(width: usize, height: usize) -> Self {
        let mut data = vec![[1.; 4]; width * height];
        data[width / 2 + (height / 2) * width] = [0.; 4];
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn new_random(width: usize, height: usize) -> Self {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            let mut color = [0.; 4];
            for ix in 0..4 {
                color[ix] = gen_range::<f32>(0., 1.);
            }
            data.push(color);
        }
        VecMatrix {
            data,
            width,
            height,
        }
    }

    fn index(&self, (ixx, ixy): (usize, usize)) -> [f32; 4] {
        self.data[ixx + ixy * self.width]
    }

    fn set_at_index(&mut self, (ixx, ixy): (usize, usize), value: [f32; 4]) {
        self.data[ixx + ixy * self.width] = value;
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}
