use crate::modifier::mod_error::ModifierError;
use crate::modifier::mod_selection::SelectionSource;
use crate::primitives::ids::ComponentId;
use crate::primitives::label::{Label, ResolvedLabel};
use crate::primitives::money::{Money, Rate, RationalMoney, RoundingStrategy};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChoicePrice {
    flat_amount: Option<Money>,
    invariant_rate: Rate,
    factor: Option<Rate>,
}

impl ChoicePrice {
    pub fn none() -> Self {
        Self {
            flat_amount: None,
            invariant_rate: Rate::zero(),
            factor: None,
        }
    }

    pub fn flat_amount(amount: Money) -> Result<Self, ModifierError> {
        Self::none().with_flat_amount(amount)
    }

    pub fn from_invariant_rate(rate: Rate) -> Self {
        Self::none().with_invariant_rate(rate)
    }

    pub fn from_factor(rate: Rate) -> Self {
        Self::none().with_factor(rate)
    }

    pub fn with_flat_amount(mut self, amount: Money) -> Result<Self, ModifierError> {
        if amount.amount_minor() < 0 {
            return Err(ModifierError::NegativeChoicePrice);
        }

        self.flat_amount = Some(amount);
        Ok(self)
    }

    pub fn with_invariant_rate(mut self, rate: Rate) -> Self {
        self.invariant_rate = rate;
        self
    }

    pub fn with_factor(mut self, rate: Rate) -> Self {
        self.factor = Some(rate);
        self
    }

    pub fn flat_amount_ref(&self) -> Option<&Money> {
        self.flat_amount.as_ref()
    }

    pub fn invariant_rate(&self) -> Rate {
        self.invariant_rate
    }

    pub fn factor(&self) -> Option<Rate> {
        self.factor
    }

    pub fn has_amount(&self) -> bool {
        self.flat_amount
            .as_ref()
            .is_some_and(|amount| amount.amount_minor() > 0)
            || !self.invariant_rate.is_zero()
    }
}

impl Default for ChoicePrice {
    fn default() -> Self {
        Self::none()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ModifierPricingPolicy {
    defaults_are_free: bool,
    rounding: RoundingStrategy,
}

impl ModifierPricingPolicy {
    pub fn new(defaults_are_free: bool, rounding: RoundingStrategy) -> Self {
        Self {
            defaults_are_free,
            rounding,
        }
    }

    pub fn defaults_are_free(self) -> bool {
        self.defaults_are_free
    }

    pub fn rounding(self) -> RoundingStrategy {
        self.rounding
    }
}

impl Default for ModifierPricingPolicy {
    fn default() -> Self {
        Self::new(true, RoundingStrategy::CentRoundUp)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PriceContribution {
    choice_id: ComponentId,
    label: ResolvedLabel,
    label_definition: Label,
    quantity: u32,
    source: SelectionSource,
    amount: Money,
    flat_amount: Option<Money>,
    invariant_rate: Rate,
    factors: Vec<PriceFactor>,
}

impl PriceContribution {
    pub(crate) fn new(
        choice_id: ComponentId,
        label: ResolvedLabel,
        label_definition: Label,
        quantity: u32,
        source: SelectionSource,
        amount: Money,
        price: &ChoicePrice,
        factors: Vec<PriceFactor>,
    ) -> Self {
        Self {
            choice_id,
            label,
            label_definition,
            quantity,
            source,
            amount,
            flat_amount: price.flat_amount.clone(),
            invariant_rate: price.invariant_rate,
            factors,
        }
    }

    pub fn choice_id(&self) -> &ComponentId {
        &self.choice_id
    }

    pub fn title(&self) -> &str {
        self.label.value()
    }

    pub fn label(&self) -> &ResolvedLabel {
        &self.label
    }

    pub fn label_definition(&self) -> &Label {
        &self.label_definition
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn source(&self) -> SelectionSource {
        self.source
    }

    pub fn amount(&self) -> &Money {
        &self.amount
    }

    pub fn flat_amount(&self) -> Option<&Money> {
        self.flat_amount.as_ref()
    }

    pub fn invariant_rate(&self) -> Rate {
        self.invariant_rate
    }

    pub fn factors(&self) -> &[PriceFactor] {
        &self.factors
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PriceFactor {
    choice_id: ComponentId,
    rate: Rate,
}

impl PriceFactor {
    pub(crate) fn new(choice_id: ComponentId, rate: Rate) -> Self {
        Self { choice_id, rate }
    }

    pub fn choice_id(&self) -> &ComponentId {
        &self.choice_id
    }

    pub fn rate(&self) -> Rate {
        self.rate
    }
}

#[doc = include_str!("modifier-pricing.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PricedConfiguration {
    contributions: Vec<PriceContribution>,
    total: Money,
}

impl PricedConfiguration {
    pub(crate) fn new(contributions: Vec<PriceContribution>, total: Money) -> PricedConfiguration {
        Self {
            contributions,
            total,
        }
    }

    pub fn contributions(&self) -> &[PriceContribution] {
        &self.contributions
    }

    pub fn total(&self) -> &Money {
        &self.total
    }
}

pub(crate) fn branch_amount(
    invariant_price: &Money,
    price: &ChoicePrice,
    quantity: u32,
    factors: &[PriceFactor],
    rounding: RoundingStrategy,
) -> Result<Money, ModifierError> {
    let mut amount = if let Some(flat_amount) = price.flat_amount_ref() {
        RationalMoney::from_money(flat_amount.clone()).map_err(ModifierError::Money)?
    } else {
        RationalMoney::zero(invariant_price.currency().clone())
    };

    let invariant_component =
        RationalMoney::from_money_rate(invariant_price, price.invariant_rate())
            .map_err(ModifierError::Money)?;

    amount = amount
        .checked_add(&invariant_component)
        .map_err(ModifierError::Money)?;

    for factor in factors {
        amount = amount
            .checked_mul_rate(factor.rate())
            .map_err(ModifierError::Money)?;
    }

    amount
        .checked_mul_quantity(quantity)
        .map_err(ModifierError::Money)?
        .round(rounding)
        .map_err(ModifierError::Money)
}
