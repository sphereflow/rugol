use std::ops::RangeInclusive;

use crate::{CellType, FieldType};

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
                range: 0..=1,
                transition: CellType::NoCell,
            },
            Rule {
                state: CellType::A,
                range: 4..=8,
                transition: CellType::NoCell,
            },
            Rule {
                state: CellType::NoCell,
                range: 3..=3,
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
                range: 0..=3,
                transition: CellType::NoCell,
            },
            Rule {
                state: CellType::A,
                range: 10..=28,
                transition: CellType::NoCell,
            },
            Rule {
                state: CellType::NoCell,
                range: 6..=8,
                transition: CellType::A,
            },
        ],
    }
}
