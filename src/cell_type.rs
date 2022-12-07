use std::{
    fmt::Display,
    ops::{Add, Index, Mul, RangeInclusive},
};

use egui::{emath::Numeric, DragValue, RadioButton, Ui};
use num_traits::{AsPrimitive, Bounded, One, Zero};
use quad_rand::{gen_range, RandomRange};
use serde::{Deserialize, Serialize};

use crate::{color::Color, color_u8, FieldType};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum CellType {
    NoCell,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl Numeric for CellType {
    const MIN: Self = CellType::NoCell;
    const MAX: Self = CellType::H;
    const INTEGRAL: bool = true;
    fn to_f64(self) -> f64 {
        self as u8 as f64
    }

    fn from_f64(num: f64) -> Self {
        let unum = num as usize;
        match CellType::try_from(unum) {
            Ok(cell) => cell,
            Err(_) => CellType::NoCell,
        }
    }
}

impl CellType {
    pub fn as_index(&self) -> usize {
        *self as usize
    }

    pub fn random_range(range: &RangeInclusive<CellType>) -> CellType {
        Self::try_from(gen_range::<usize>(
            *range.start() as usize,
            *range.end() as usize + 1,
        ))
        .expect("CellType::try_from(usize) failed")
    }
}

impl TryFrom<usize> for CellType {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CellType::NoCell),
            1 => Ok(CellType::A),
            2 => Ok(CellType::B),
            3 => Ok(CellType::C),
            4 => Ok(CellType::D),
            5 => Ok(CellType::E),
            6 => Ok(CellType::F),
            7 => Ok(CellType::G),
            8 => Ok(CellType::H),
            _ => Err(()),
        }
    }
}

impl Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellType::NoCell => write!(f, "0"),
            CellType::A => write!(f, "A"),
            CellType::B => write!(f, "B"),
            CellType::C => write!(f, "C"),
            CellType::D => write!(f, "D"),
            CellType::E => write!(f, "E"),
            CellType::F => write!(f, "F"),
            CellType::G => write!(f, "G"),
            CellType::H => write!(f, "H"),
        }
    }
}

impl RandomRange for CellType {
    fn gen_range(low: Self, high: Self) -> Self {
        Self::random_range(&(low..=high))
    }
}

impl Zero for CellType {
    fn zero() -> Self {
        CellType::NoCell
    }

    fn is_zero(&self) -> bool {
        *self == CellType::NoCell
    }
}

impl Add for CellType {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::try_from((self.as_index() + rhs.as_index()) % 9).unwrap()
    }
}

impl One for CellType {
    fn one() -> Self {
        CellType::A
    }
}

impl Mul for CellType {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::try_from((self.as_index() * rhs.as_index()) % 9).unwrap()
    }
}

impl Bounded for CellType {
    fn min_value() -> Self {
        CellType::NoCell
    }

    fn max_value() -> Self {
        CellType::H
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CellTypeMap {
    map: Vec<(Color, FieldType)>,
    selected_idx: usize,
    default: (Color, FieldType),
}

impl Default for CellTypeMap {
    fn default() -> Self {
        Self::new()
    }
}

impl CellTypeMap {
    pub fn new() -> Self {
        let map = vec![
            (color_u8!(50, 20, 20, 200), 0_u8.as_()),    // NoCell
            (color_u8!(236, 55, 90, 200), 1_u8.as_()),   // A
            (color_u8!(229, 117, 35, 200), 2_u8.as_()),  // B
            (color_u8!(89, 229, 39, 200), 3_u8.as_()),   // C
            (color_u8!(228, 228, 32, 200), 4_u8.as_()),  // D
            (color_u8!(105, 216, 205, 200), 5_u8.as_()), // E
            (color_u8!(0, 94, 229, 200), 6_u8.as_()),    // F
            (color_u8!(190, 52, 219, 200), 7_u8.as_()),  // G
            (color_u8!(229, 152, 152, 200), 8_u8.as_()), // H
        ];
        dbg!(map.clone());
        CellTypeMap {
            map,
            selected_idx: 0,
            default: (color_u8!(255, 255, 255, 255), 0_u8.as_()),
        }
    }

    pub fn color_for_value(&self, value: FieldType) -> Option<Color> {
        for (color, v) in &self.map {
            if *v == value {
                return Some(*color);
            }
        }
        None
    }

    pub fn inv_lookup(&self, idx: usize) -> CellType {
        idx.try_into()
            .expect("Index does not correspond to any CellType")
    }

    fn get_map_mut(&mut self) -> &mut Vec<(Color, FieldType)> {
        &mut self.map
    }

    fn get_map(&self) -> &Vec<(Color, FieldType)> {
        &self.map
    }

    fn get_selected_idx(&self) -> usize {
        self.selected_idx
    }

    fn set_selected_idx(&mut self, idx: usize) {
        self.selected_idx = idx.clamp(0, 8);
    }

    pub fn get_selected_rules_cell(&self) -> CellType {
        self.inv_lookup(self.get_selected_idx())
    }

    pub fn get_selected_rules_val(&self) -> FieldType {
        self.map[self.get_selected_idx()].1
    }

    pub fn edit(&mut self, ui: &mut Ui) {
        let mut selected_idx = self.get_selected_idx();
        for chunk in (0..self.get_map().len())
            .collect::<Vec<usize>>()
            .chunks_mut(4)
        {
            ui.horizontal(|ui| {
                for &idx in chunk.iter() {
                    if ui
                        .add(RadioButton::new(
                            idx == selected_idx,
                            format!("{}", self.inv_lookup(idx)),
                        ))
                        .clicked()
                    {
                        selected_idx = idx;
                    }
                    let map = self.get_map_mut();
                    let mut edit_color_u8: [u8; 4] = map[idx].0.into();
                    if ui
                        .color_edit_button_srgba_unmultiplied(&mut edit_color_u8)
                        .changed()
                    {
                        map[idx].0 = edit_color_u8.into();
                    }
                    ui.horizontal(|ui| {
                        ui.label("value:");
                        ui.add(DragValue::new(&mut map[idx].1).speed(0.01));
                    });
                }
            });
        }
        self.set_selected_idx(selected_idx);
    }
}

impl Index<CellType> for CellTypeMap {
    type Output = (Color, FieldType);
    fn index(&self, cell_type: CellType) -> &Self::Output {
        match self.map.get(cell_type.as_index()) {
            Some(val) => val,
            _ => &self.default,
        }
    }
}
