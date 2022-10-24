use crate::{CellType, FieldType};
use num_traits::{AsPrimitive, One, Zero};
use std::ops::RangeInclusive;

pub struct Rule<T: Copy> {
    pub state: CellType,
    pub range: RangeInclusive<T>,
    pub transition: CellType,
}

pub struct Rules<T: Copy> {
    pub rules: Vec<Rule<T>>,
}

impl<T: Copy + PartialEq + PartialOrd> Rules<T> {
    pub fn apply(&self, initial_value: CellType, convolution: T) -> CellType {
        for rule in &self.rules {
            if rule.state == initial_value && rule.range.contains(&convolution) {
                return rule.transition;
            }
        }

        // you think the rules don't apply to you Mr. Anderson
        initial_value
    }
}

pub fn classic_rules() -> Rules<FieldType> {
    Rules {
        rules: vec![
            Rule {
                state: CellType::A,
                range: Zero::zero()..=One::one(),
                transition: CellType::NoCell,
            },
            Rule {
                state: CellType::A,
                range: 4_u8.as_()..=8_u8.as_(),
                transition: CellType::NoCell,
            },
            Rule {
                state: CellType::NoCell,
                range: 3_u8.as_()..=3_u8.as_(),
                transition: CellType::A,
            },
        ],
    }
}

pub fn flame_rules() -> Rules<FieldType> {
    Rules {
        rules: vec![
            Rule {
                state: CellType::A,
                range: 0_u8.as_()..=3_u8.as_(),
                transition: CellType::NoCell,
            },
            Rule {
                state: CellType::A,
                range: 10_u8.as_()..=28_u8.as_(),
                transition: CellType::NoCell,
            },
            Rule {
                state: CellType::NoCell,
                range: 6_u8.as_()..=8_u8.as_(),
                transition: CellType::A,
            },
        ],
    }
}
