use std::fmt;

use crate::modifier::mod_rule::RuleKind;
use crate::primitives::ids::ComponentId;
use crate::primitives::label::LabelError;
use crate::primitives::money::MoneyError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ModifierError {
    EmptyPromptTitle,
    EmptyChoiceTitle,
    ChoiceInput(Box<ChoiceInputError>),
    DefaultRuleOnPrompt,
    InvalidConstraints {
        min_select: u32,
        max_select: u32,
    },
    DuplicateRule(RuleKind),
    RequiredPromptHasNoChoices {
        prompt_id: ComponentId,
        min_select: u32,
    },
    DuplicateChoice(ComponentId),
    DuplicateDefault(ComponentId),
    ZeroDefaultQuantity(ComponentId),
    ZeroQuantity(ComponentId),
    DuplicateSelection(ComponentId),
    UnknownPromptSelection(ComponentId),
    UnknownSelection(ComponentId),
    InapplicablePromptSelection(ComponentId),
    InapplicableChoiceSelection(ComponentId),
    ScheduledChoiceRequiresEvaluationTime(ComponentId),
    UnavailableChoiceSelection(ComponentId),
    UnexpectedNestedSelections(ComponentId),
    NegativeChoicePrice,
    UnconsumedPriceFactor(ComponentId),
    Label(LabelError),
    Money(MoneyError),
    SelectionCountOverflow,
    BelowMinimum {
        min_select: u32,
        actual: u32,
    },
    AboveMaximum {
        max_select: u32,
        actual: u32,
    },
    ChoiceBelowMinimum {
        choice_id: ComponentId,
        min_select: u32,
        actual: u32,
    },
    ChoiceAboveMaximum {
        choice_id: ComponentId,
        max_select: u32,
        actual: u32,
    },
}

impl fmt::Display for ModifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPromptTitle => f.write_str("prompt title cannot be empty"),
            Self::EmptyChoiceTitle => f.write_str("choice title cannot be empty"),
            Self::ChoiceInput(error) => write!(f, "{error}"),
            Self::DefaultRuleOnPrompt => {
                f.write_str("default rules belong on choices, not prompts")
            }
            Self::InvalidConstraints {
                min_select,
                max_select,
            } => write!(
                f,
                "selection constraints are invalid: min {min_select} exceeds max {max_select}"
            ),
            Self::DuplicateRule(kind) => write!(f, "duplicate {kind:?} rule"),
            Self::RequiredPromptHasNoChoices {
                prompt_id,
                min_select,
            } => write!(
                f,
                "prompt `{prompt_id}` requires {min_select} selections but has no choices"
            ),
            Self::DuplicateChoice(choice_id) => write!(f, "duplicate choice `{choice_id}`"),
            Self::DuplicateDefault(choice_id) => {
                write!(f, "duplicate default rule on choice `{choice_id}`")
            }
            Self::ZeroDefaultQuantity(choice_id) => {
                write!(
                    f,
                    "default quantity for choice `{choice_id}` cannot be zero"
                )
            }
            Self::ZeroQuantity(choice_id) => {
                write!(f, "choice selection `{choice_id}` has zero quantity")
            }
            Self::DuplicateSelection(choice_id) => {
                write!(f, "duplicate choice selection `{choice_id}`")
            }
            Self::UnknownPromptSelection(prompt_id) => {
                write!(f, "unknown prompt selection `{prompt_id}`")
            }
            Self::UnknownSelection(choice_id) => {
                write!(f, "unknown choice selection `{choice_id}`")
            }
            Self::InapplicablePromptSelection(prompt_id) => {
                write!(f, "prompt selection `{prompt_id}` is not applicable")
            }
            Self::InapplicableChoiceSelection(choice_id) => {
                write!(f, "choice selection `{choice_id}` is not applicable")
            }
            Self::ScheduledChoiceRequiresEvaluationTime(choice_id) => {
                write!(
                    f,
                    "choice `{choice_id}` has a schedule and requires an evaluation time"
                )
            }
            Self::UnavailableChoiceSelection(choice_id) => {
                write!(f, "choice selection `{choice_id}` is unavailable")
            }
            Self::UnexpectedNestedSelections(choice_id) => {
                write!(f, "choice `{choice_id}` does not accept nested selections")
            }
            Self::NegativeChoicePrice => f.write_str("choice price cannot be negative"),
            Self::UnconsumedPriceFactor(choice_id) => write!(
                f,
                "choice `{choice_id}` defines a price factor without a priced ancestor"
            ),
            Self::Label(error) => write!(f, "{error}"),
            Self::Money(error) => write!(f, "{error}"),
            Self::SelectionCountOverflow => f.write_str("choice selection count overflow"),
            Self::BelowMinimum { min_select, actual } => {
                write!(f, "choice count {actual} is below min {min_select}")
            }
            Self::AboveMaximum { max_select, actual } => {
                write!(f, "choice count {actual} exceeds max {max_select}")
            }
            Self::ChoiceBelowMinimum {
                choice_id,
                min_select,
                actual,
            } => write!(
                f,
                "choice `{choice_id}` quantity {actual} is below min {min_select}"
            ),
            Self::ChoiceAboveMaximum {
                choice_id,
                max_select,
                actual,
            } => write!(
                f,
                "choice `{choice_id}` quantity {actual} exceeds max {max_select}"
            ),
        }
    }
}

impl std::error::Error for ModifierError {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ChoiceInputError {
    EmptyTitle,
    InvalidLengthConstraints {
        input_id: ComponentId,
        min_length: u32,
        max_length: u32,
    },
    DuplicateDefinition(ComponentId),
    UnknownInput {
        choice_id: ComponentId,
        input_id: ComponentId,
    },
    UnexpectedUnit {
        choice_id: ComponentId,
        input_id: ComponentId,
    },
    UnitRequired {
        choice_id: ComponentId,
        input_id: ComponentId,
    },
    UnitOutOfRange {
        choice_id: ComponentId,
        input_id: ComponentId,
        unit: u32,
        quantity: u32,
    },
    DuplicateValue {
        choice_id: ComponentId,
        input_id: ComponentId,
        unit: Option<u32>,
    },
    MissingRequiredValue {
        choice_id: ComponentId,
        input_id: ComponentId,
        expected: u32,
        actual: usize,
    },
    BelowMinimumLength {
        choice_id: ComponentId,
        input_id: ComponentId,
        min_length: u32,
        actual: usize,
    },
    AboveMaximumLength {
        choice_id: ComponentId,
        input_id: ComponentId,
        max_length: u32,
        actual: usize,
    },
}

impl fmt::Display for ChoiceInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => f.write_str("choice input title cannot be empty"),
            Self::InvalidLengthConstraints {
                input_id,
                min_length,
                max_length,
            } => write!(
                f,
                "choice input `{input_id}` length constraints are invalid: min {min_length} exceeds max {max_length}"
            ),
            Self::DuplicateDefinition(input_id) => {
                write!(f, "duplicate choice input `{input_id}`")
            }
            Self::UnknownInput {
                choice_id,
                input_id,
            } => write!(f, "choice `{choice_id}` does not define input `{input_id}`"),
            Self::UnexpectedUnit {
                choice_id,
                input_id,
            } => write!(
                f,
                "choice input `{input_id}` on choice `{choice_id}` is collected once and cannot identify a unit"
            ),
            Self::UnitRequired {
                choice_id,
                input_id,
            } => write!(
                f,
                "choice input `{input_id}` on choice `{choice_id}` repeats per quantity and must identify a unit"
            ),
            Self::UnitOutOfRange {
                choice_id,
                input_id,
                unit,
                quantity,
            } => write!(
                f,
                "choice input `{input_id}` unit {unit} is outside choice `{choice_id}` quantity {quantity}"
            ),
            Self::DuplicateValue {
                choice_id,
                input_id,
                unit,
            } => match unit {
                Some(unit) => write!(
                    f,
                    "choice input `{input_id}` has duplicate value for choice `{choice_id}` unit {unit}"
                ),
                None => write!(
                    f,
                    "choice input `{input_id}` has duplicate value for choice `{choice_id}`"
                ),
            },
            Self::MissingRequiredValue {
                choice_id,
                input_id,
                expected,
                actual,
            } => write!(
                f,
                "required choice input `{input_id}` on choice `{choice_id}` expected {expected} value(s), found {actual}"
            ),
            Self::BelowMinimumLength {
                choice_id,
                input_id,
                min_length,
                actual,
            } => write!(
                f,
                "choice input `{input_id}` on choice `{choice_id}` length {actual} is below min {min_length}"
            ),
            Self::AboveMaximumLength {
                choice_id,
                input_id,
                max_length,
                actual,
            } => write!(
                f,
                "choice input `{input_id}` on choice `{choice_id}` length {actual} exceeds max {max_length}"
            ),
        }
    }
}

impl std::error::Error for ChoiceInputError {}

impl From<ChoiceInputError> for ModifierError {
    fn from(error: ChoiceInputError) -> Self {
        Self::ChoiceInput(Box::new(error))
    }
}

impl From<LabelError> for ModifierError {
    fn from(error: LabelError) -> Self {
        Self::Label(error)
    }
}
