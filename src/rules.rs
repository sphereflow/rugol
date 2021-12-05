use std::ops::RangeInclusive;

pub struct Rule {
    pub state: u8,
    pub range: RangeInclusive<u8>,
    pub transition: u8,
}

pub struct Rules {
    pub rules: Vec<Rule>,
}

impl Rules {
    pub fn apply(&self, initial_value: u8, convolution: u8) -> u8 {
        for rule in &self.rules {
            if rule.state == initial_value && rule.range.contains(&convolution) {
                return rule.transition;
            }
        }

        // you think the rules don't apply to you mister anderson
        initial_value
    }
}

pub fn classic_rules() -> Rules {
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
