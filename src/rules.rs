use std::ops::RangeInclusive;

use crate::FieldType;

pub struct Rule<T: Copy> {
    pub state: T,
    pub range: RangeInclusive<T>,
    pub transition: T,
}

pub struct Rules<T: Copy> {
    pub rules: Vec<Rule<T>>,
}

impl<T: Copy + PartialEq + PartialOrd> Rules<T> {
    pub fn apply(&self, initial_value: T, convolution: T) -> T {
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
                state: 1,
                range: 0..=1,
                transition: 0,
            },
            Rule {
                state: 1,
                range: 4..=8,
                transition: 0,
            },
            Rule {
                state: 0,
                range: 3..=3,
                transition: 1,
            },
        ],
    }
}

pub fn flame_rules() -> Rules<FieldType> {
    Rules {
        rules: vec![
            Rule {
                state: 1,
                range: 0..=3,
                transition: 0,
            },
            Rule {
                state: 1,
                range: 10..=28,
                transition: 0,
            },
            Rule {
                state: 0,
                range: 6..=8,
                transition: 1,
            },
        ],
    }
}
