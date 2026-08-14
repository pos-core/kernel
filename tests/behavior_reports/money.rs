use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "money",
        title: "Money",
        description: "Described behavior tests for checked minor-unit arithmetic, integer rates, and named rounding strategies.",
        definitions: vec![DefinitionLink::new(
            "Money",
            "../src/primitives/money/money.md",
        )],
        cases: vec![
            MONEY_USES_CHECKED_MINOR_UNITS.report_case(),
            MONEY_MULTIPLIES_BY_INTEGER_RATES_WITH_NAMED_ROUNDING.report_case(),
            MONEY_ROUNDS_UP_TO_NAMED_INCREMENT_AND_ENDING_TARGETS.report_case(),
        ],
    }
}

pub const MONEY_USES_CHECKED_MINOR_UNITS: DescribedBehavior = DescribedBehavior::new(
    "money uses checked minor units",
    "Money stores integer minor units and performs checked arithmetic without floating-point values.",
    money_uses_checked_minor_units,
);

#[test]
fn money_uses_checked_minor_units() {
    let usd = CurrencyCode::parse("USD").unwrap();
    let one = Money::new(100, usd.clone());
    let two = Money::new(250, usd);

    assert_eq!(one.checked_add(&two).unwrap().amount_minor(), 350);
}

pub const MONEY_MULTIPLIES_BY_INTEGER_RATES_WITH_NAMED_ROUNDING: DescribedBehavior =
    DescribedBehavior::new(
        "money multiplies by integer rates with named rounding",
        "Rate multiplication keeps rational arithmetic exact until an explicit named rounding strategy materializes minor units.",
        money_multiplies_by_integer_rates_with_named_rounding,
    );

#[test]
fn money_multiplies_by_integer_rates_with_named_rounding() {
    let amount = Money::new(199, CurrencyCode::parse("USD").unwrap());

    assert_eq!(
        amount
            .checked_mul_rate(Rate::percent(50), RoundingStrategy::CentRoundDown)
            .unwrap()
            .amount_minor(),
        99
    );
    assert_eq!(
        amount
            .checked_mul_rate(Rate::percent(50), RoundingStrategy::CentRoundUp)
            .unwrap()
            .amount_minor(),
        100
    );
}

pub const MONEY_ROUNDS_UP_TO_NAMED_INCREMENT_AND_ENDING_TARGETS: DescribedBehavior =
    DescribedBehavior::new(
        "money rounds up to named increment and ending targets",
        "Named rounding can move an amount upward to a required minor-unit increment or price ending.",
        money_rounds_up_to_named_increment_and_ending_targets,
    );

#[test]
fn money_rounds_up_to_named_increment_and_ending_targets() {
    let amount = Money::new(201, CurrencyCode::parse("USD").unwrap());

    assert_eq!(
        amount
            .checked_mul_rate(Rate::one(), RoundingStrategy::NearestUpIncrement(25))
            .unwrap()
            .amount_minor(),
        225
    );
    assert_eq!(
        amount
            .checked_mul_rate(Rate::one(), RoundingStrategy::NearestUpEnding(99))
            .unwrap()
            .amount_minor(),
        299
    );
}
