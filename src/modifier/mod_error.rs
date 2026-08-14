use std::fmt;

use crate::modifier::mod_rule::RuleKind;
use crate::primitives::ids::ComponentId;
use crate::primitives::label::LabelError;
use crate::primitives::money::MoneyError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ModifierError {
    EmptyPromptTitle,
    EmptyChoiceTitle,
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

impl From<LabelError> for ModifierError {
    fn from(error: LabelError) -> Self {
        Self::Label(error)
    }
}
