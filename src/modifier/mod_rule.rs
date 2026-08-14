use crate::modifier::mod_error::ModifierError;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Rule {
    Min(u32),
    Max(u32),
    Default(u32),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RuleKind {
    Min,
    Max,
    Default,
}

impl Rule {
    pub fn kind(self) -> RuleKind {
        match self {
            Self::Min(_) => RuleKind::Min,
            Self::Max(_) => RuleKind::Max,
            Self::Default(_) => RuleKind::Default,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) struct SelectionBounds {
    pub(super) min: u32,
    pub(super) max: Option<u32>,
}

impl SelectionBounds {
    pub(super) fn from_rules(rules: &[Rule]) -> Result<Self, ModifierError> {
        let mut min = 0_u32;
        let mut max: Option<u32> = None;
        let mut has_min = false;
        let mut has_max = false;

        for rule in rules {
            match *rule {
                Rule::Min(value) => {
                    if has_min {
                        return Err(ModifierError::DuplicateRule(RuleKind::Min));
                    }

                    has_min = true;
                    min = value;
                }
                Rule::Max(value) => {
                    if has_max {
                        return Err(ModifierError::DuplicateRule(RuleKind::Max));
                    }

                    has_max = true;
                    max = Some(value);
                }
                Rule::Default(_) => {}
            }
        }

        if let Some(max_select) = max
            && min > max_select
        {
            return Err(ModifierError::InvalidConstraints {
                min_select: min,
                max_select,
            });
        }

        Ok(Self { min, max })
    }
}
