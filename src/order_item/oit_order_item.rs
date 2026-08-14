use std::fmt;

use crate::catalog_item::ConfiguredCatalogItem;
use crate::effect::Effect;
use crate::entry::{EntryError, EntryKind, EntrySource, OrderEntry, PriceCategory};
use crate::modifier::{
    ChoicePrice, ChoiceSnapshot, ConfigurationSnapshot, ModifierError, PriceContribution,
    PriceFactor, PromptSnapshot, SelectionSource,
};
use crate::primitives::ids::{
    CatalogItemId, CatalogVersionId, ComponentId, EntryId, OrderItemId, VariantId,
};
use crate::primitives::label::Label;
use crate::primitives::money::{CurrencyCode, Money, MoneyError, Rate};

#[doc = include_str!("order-item.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrderItem {
    order_item_id: OrderItemId,
    source: OrderItemSource,
    item_label: Label,
    variant_label: Option<Label>,
    quantity: u32,
    invariant_unit_price: Money,
    modifier_unit_price: Money,
    unit_price: Money,
    total_price: Money,
    effects: Vec<Effect>,
    modifiers: OrderItemModifierSnapshot,
}

impl OrderItem {
    pub fn from_configured_catalog_item(
        order_item_id: OrderItemId,
        catalog_version_id: CatalogVersionId,
        quantity: u32,
        configured: &ConfiguredCatalogItem,
    ) -> Result<Self, OrderItemError> {
        let modifiers = configured
            .modifiers()
            .snapshot(configured.invariant_price(), configured.pricing_policy())
            .map_err(OrderItemError::Modifier)?;

        Self::new(
            order_item_id,
            OrderItemSource::Catalog {
                catalog_version_id,
                catalog_item_id: configured.catalog_item_id().clone(),
                variant_id: configured.variant_id().clone(),
            },
            configured.catalog_item_label_definition().clone(),
            configured.variant_label_definition().cloned(),
            quantity,
            configured.invariant_price().clone(),
            OrderItemModifierSnapshot::from_configuration_snapshot(&modifiers)?,
            configured.effects().to_vec(),
        )
    }

    pub fn manual(
        order_item_id: OrderItemId,
        item_label: Label,
        variant_label: Option<Label>,
        quantity: u32,
        invariant_unit_price: Money,
        modifiers: OrderItemModifierSnapshot,
    ) -> Result<Self, OrderItemError> {
        Self::new(
            order_item_id,
            OrderItemSource::Manual,
            item_label,
            variant_label,
            quantity,
            invariant_unit_price,
            modifiers,
            Vec::new(),
        )
    }

    fn new(
        order_item_id: OrderItemId,
        source: OrderItemSource,
        item_label: Label,
        variant_label: Option<Label>,
        quantity: u32,
        invariant_unit_price: Money,
        modifiers: OrderItemModifierSnapshot,
        effects: Vec<Effect>,
    ) -> Result<Self, OrderItemError> {
        if quantity == 0 {
            return Err(OrderItemError::ZeroQuantity);
        }

        let modifier_unit_price = modifiers.price().total().clone();
        let unit_price = invariant_unit_price
            .checked_add(&modifier_unit_price)
            .map_err(OrderItemError::Money)?;
        let total_price = unit_price
            .checked_mul_quantity(quantity)
            .map_err(OrderItemError::Money)?;

        Ok(Self {
            order_item_id,
            source,
            item_label,
            variant_label,
            quantity,
            invariant_unit_price,
            modifier_unit_price,
            unit_price,
            total_price,
            effects,
            modifiers,
        })
    }

    pub fn order_item_id(&self) -> &OrderItemId {
        &self.order_item_id
    }

    pub fn source(&self) -> &OrderItemSource {
        &self.source
    }

    pub fn catalog_version_id(&self) -> Option<&CatalogVersionId> {
        match &self.source {
            OrderItemSource::Catalog {
                catalog_version_id, ..
            } => Some(catalog_version_id),
            OrderItemSource::External { .. } | OrderItemSource::Manual => None,
        }
    }

    pub fn catalog_item_id(&self) -> Option<&CatalogItemId> {
        match &self.source {
            OrderItemSource::Catalog {
                catalog_item_id, ..
            } => Some(catalog_item_id),
            OrderItemSource::External { .. } | OrderItemSource::Manual => None,
        }
    }

    pub fn item_label(&self) -> &Label {
        &self.item_label
    }

    pub fn catalog_item_label(&self) -> &Label {
        &self.item_label
    }

    pub fn variant_id(&self) -> Option<&VariantId> {
        match &self.source {
            OrderItemSource::Catalog { variant_id, .. } => Some(variant_id),
            OrderItemSource::External { .. } | OrderItemSource::Manual => None,
        }
    }

    pub fn variant_label(&self) -> Option<&Label> {
        self.variant_label.as_ref()
    }

    pub fn description(&self) -> String {
        if let Some(variant_label) = &self.variant_label {
            format!(
                "{} ({})",
                self.item_label.default_text(),
                variant_label.default_text()
            )
        } else {
            self.item_label.default_text().to_owned()
        }
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn invariant_unit_price(&self) -> &Money {
        &self.invariant_unit_price
    }

    pub fn modifier_unit_price(&self) -> &Money {
        &self.modifier_unit_price
    }

    pub fn unit_price(&self) -> &Money {
        &self.unit_price
    }

    pub fn total_price(&self) -> &Money {
        &self.total_price
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn modifiers(&self) -> &OrderItemModifierSnapshot {
        &self.modifiers
    }

    pub fn entries(
        &self,
        item_entry_id: EntryId,
        modifier_entry_ids: Vec<EntryId>,
    ) -> Result<Vec<OrderEntry>, OrderItemError> {
        let contributions = self.modifiers.price().contributions();

        if modifier_entry_ids.len() != contributions.len() {
            return Err(OrderItemError::ModifierEntryIdCountMismatch {
                expected: contributions.len(),
                actual: modifier_entry_ids.len(),
            });
        }

        let mut entries = Vec::with_capacity(1 + contributions.len());
        let item_entry = OrderEntry::builder(
            item_entry_id.clone(),
            EntryKind::Item,
            self.item_entry_source(),
            self.description(),
            self.quantity,
            self.invariant_unit_price.clone(),
        )
        .with_price_category(PriceCategory::BaseItem)
        .build()
        .map_err(OrderItemError::Entry)?;

        entries.push(item_entry);

        for (entry_id, contribution) in modifier_entry_ids.into_iter().zip(contributions) {
            let modifier_entry = OrderEntry::builder(
                entry_id,
                EntryKind::Modifier,
                self.modifier_entry_source(contribution),
                contribution.label().default_text(),
                self.quantity,
                contribution.amount().clone(),
            )
            .with_price_category(PriceCategory::Modifier)
            .build()
            .map_err(OrderItemError::Entry)?;

            entries.push(modifier_entry);
        }

        Ok(entries)
    }

    fn item_entry_source(&self) -> EntrySource {
        match &self.source {
            OrderItemSource::Catalog {
                catalog_version_id,
                catalog_item_id,
                variant_id,
            } => EntrySource::CatalogItem {
                catalog_version_id: catalog_version_id.clone(),
                catalog_item_id: catalog_item_id.clone(),
                variant_id: variant_id.clone(),
            },
            OrderItemSource::External {
                system,
                external_id,
            } => EntrySource::External {
                system: system.clone(),
                external_id: external_id.clone(),
                mapped_component_id: None,
            },
            OrderItemSource::Manual => EntrySource::Manual,
        }
    }

    fn modifier_entry_source(&self, contribution: &OrderItemPriceContribution) -> EntrySource {
        match (&self.source, contribution.choice_id()) {
            (
                OrderItemSource::Catalog {
                    catalog_version_id, ..
                },
                Some(component_id),
            ) => EntrySource::Catalog {
                catalog_version_id: catalog_version_id.clone(),
                component_id: component_id.clone(),
            },
            (
                OrderItemSource::External {
                    system,
                    external_id,
                },
                component_id,
            ) => EntrySource::External {
                system: system.clone(),
                external_id: external_id.clone(),
                mapped_component_id: component_id.cloned(),
            },
            _ => EntrySource::Manual,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrderItemSource {
    Catalog {
        catalog_version_id: CatalogVersionId,
        catalog_item_id: CatalogItemId,
        variant_id: VariantId,
    },
    External {
        system: String,
        external_id: Option<String>,
    },
    Manual,
}

#[doc = include_str!("order-item-modifier-snapshot.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrderItemModifierSnapshot {
    prompts: Vec<OrderItemPromptSnapshot>,
    price: OrderItemModifierPrice,
}

impl OrderItemModifierSnapshot {
    pub fn new(
        prompts: Vec<OrderItemPromptSnapshot>,
        price: OrderItemModifierPrice,
    ) -> Result<Self, OrderItemError> {
        Ok(Self { prompts, price })
    }

    pub fn empty(currency: CurrencyCode) -> Self {
        Self {
            prompts: Vec::new(),
            price: OrderItemModifierPrice::empty(currency),
        }
    }

    pub fn prompts(&self) -> &[OrderItemPromptSnapshot] {
        &self.prompts
    }

    pub fn price(&self) -> &OrderItemModifierPrice {
        &self.price
    }

    pub fn prompt(&self, prompt_id: &ComponentId) -> Option<&OrderItemPromptSnapshot> {
        self.prompts
            .iter()
            .find(|prompt| prompt.prompt_id() == Some(prompt_id))
    }

    fn from_configuration_snapshot(
        snapshot: &ConfigurationSnapshot,
    ) -> Result<Self, OrderItemError> {
        let price = OrderItemModifierPrice::from_priced_configuration(snapshot.price())?;
        Ok(Self {
            prompts: snapshot
                .prompts()
                .iter()
                .map(OrderItemPromptSnapshot::from_prompt_snapshot)
                .collect(),
            price,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrderItemPromptSnapshot {
    prompt_id: Option<ComponentId>,
    label: Label,
    description: Option<Label>,
    effects: Vec<Effect>,
    choices: Vec<OrderItemChoiceSnapshot>,
}

impl OrderItemPromptSnapshot {
    pub fn new(
        prompt_id: Option<ComponentId>,
        label: Label,
        description: Option<Label>,
        effects: Vec<Effect>,
        choices: Vec<OrderItemChoiceSnapshot>,
    ) -> Self {
        Self {
            prompt_id,
            label,
            description,
            effects,
            choices,
        }
    }

    pub fn prompt_id(&self) -> Option<&ComponentId> {
        self.prompt_id.as_ref()
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn title(&self) -> &str {
        self.label.default_text()
    }

    pub fn description_label(&self) -> Option<&Label> {
        self.description.as_ref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(Label::default_text)
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn choices(&self) -> &[OrderItemChoiceSnapshot] {
        &self.choices
    }

    fn from_prompt_snapshot(prompt: &PromptSnapshot) -> Self {
        Self {
            prompt_id: Some(prompt.prompt_id().clone()),
            label: prompt.label_definition().clone(),
            description: prompt.description_label_definition().cloned(),
            effects: prompt.effects().to_vec(),
            choices: prompt
                .choices()
                .iter()
                .map(OrderItemChoiceSnapshot::from_choice_snapshot)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrderItemChoiceSnapshot {
    choice_id: Option<ComponentId>,
    label: Label,
    quantity: u32,
    source: SelectionSource,
    effects: Vec<Effect>,
    price: ChoicePrice,
    modifiers: Vec<OrderItemPromptSnapshot>,
}

impl OrderItemChoiceSnapshot {
    pub fn new(
        choice_id: Option<ComponentId>,
        label: Label,
        quantity: u32,
        source: SelectionSource,
        effects: Vec<Effect>,
        price: ChoicePrice,
        modifiers: Vec<OrderItemPromptSnapshot>,
    ) -> Result<Self, OrderItemError> {
        if quantity == 0 {
            return Err(OrderItemError::ZeroQuantity);
        }

        Ok(Self {
            choice_id,
            label,
            quantity,
            source,
            effects,
            price,
            modifiers,
        })
    }

    pub fn choice_id(&self) -> Option<&ComponentId> {
        self.choice_id.as_ref()
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn title(&self) -> &str {
        self.label.default_text()
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn source(&self) -> SelectionSource {
        self.source
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn price_definition(&self) -> &ChoicePrice {
        &self.price
    }

    pub fn modifiers(&self) -> &[OrderItemPromptSnapshot] {
        &self.modifiers
    }

    fn from_choice_snapshot(choice: &ChoiceSnapshot) -> Self {
        Self {
            choice_id: Some(choice.choice_id().clone()),
            label: choice.label_definition().clone(),
            quantity: choice.quantity(),
            source: choice.source(),
            effects: choice.effects().to_vec(),
            price: choice.price_definition().clone(),
            modifiers: choice
                .modifiers()
                .iter()
                .map(OrderItemPromptSnapshot::from_prompt_snapshot)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrderItemModifierPrice {
    contributions: Vec<OrderItemPriceContribution>,
    total: Money,
}

impl OrderItemModifierPrice {
    pub fn new(
        contributions: Vec<OrderItemPriceContribution>,
        total: Money,
    ) -> Result<Self, OrderItemError> {
        validate_total(&contributions, &total)?;

        Ok(Self {
            contributions,
            total,
        })
    }

    pub fn empty(currency: CurrencyCode) -> Self {
        Self {
            contributions: Vec::new(),
            total: Money::zero(currency),
        }
    }

    pub fn contributions(&self) -> &[OrderItemPriceContribution] {
        &self.contributions
    }

    pub fn total(&self) -> &Money {
        &self.total
    }

    fn from_priced_configuration(
        price: &crate::modifier::PricedConfiguration,
    ) -> Result<Self, OrderItemError> {
        Self::new(
            price
                .contributions()
                .iter()
                .map(OrderItemPriceContribution::from_price_contribution)
                .collect(),
            price.total().clone(),
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrderItemPriceContribution {
    choice_id: Option<ComponentId>,
    label: Label,
    quantity: u32,
    source: SelectionSource,
    amount: Money,
    flat_amount: Option<Money>,
    invariant_rate: Rate,
    factors: Vec<OrderItemPriceFactor>,
}

impl OrderItemPriceContribution {
    pub fn new(
        choice_id: Option<ComponentId>,
        label: Label,
        quantity: u32,
        source: SelectionSource,
        amount: Money,
        price: ChoicePrice,
        factors: Vec<OrderItemPriceFactor>,
    ) -> Result<Self, OrderItemError> {
        if quantity == 0 {
            return Err(OrderItemError::ZeroQuantity);
        }

        Ok(Self {
            choice_id,
            label,
            quantity,
            source,
            amount,
            flat_amount: price.flat_amount_ref().cloned(),
            invariant_rate: price.invariant_rate(),
            factors,
        })
    }

    pub fn unconnected(label: Label, quantity: u32, amount: Money) -> Result<Self, OrderItemError> {
        let price = ChoicePrice::flat_amount(amount.clone()).map_err(OrderItemError::Modifier)?;
        Self::new(
            None,
            label,
            quantity,
            SelectionSource::Explicit,
            amount,
            price,
            Vec::new(),
        )
    }

    pub fn choice_id(&self) -> Option<&ComponentId> {
        self.choice_id.as_ref()
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn title(&self) -> &str {
        self.label.default_text()
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

    pub fn factors(&self) -> &[OrderItemPriceFactor] {
        &self.factors
    }

    fn from_price_contribution(contribution: &PriceContribution) -> Self {
        Self {
            choice_id: Some(contribution.choice_id().clone()),
            label: contribution.label_definition().clone(),
            quantity: contribution.quantity(),
            source: contribution.source(),
            amount: contribution.amount().clone(),
            flat_amount: contribution.flat_amount().cloned(),
            invariant_rate: contribution.invariant_rate(),
            factors: contribution
                .factors()
                .iter()
                .map(OrderItemPriceFactor::from_price_factor)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrderItemPriceFactor {
    choice_id: Option<ComponentId>,
    rate: Rate,
}

impl OrderItemPriceFactor {
    pub fn new(choice_id: Option<ComponentId>, rate: Rate) -> Self {
        Self { choice_id, rate }
    }

    pub fn choice_id(&self) -> Option<&ComponentId> {
        self.choice_id.as_ref()
    }

    pub fn rate(&self) -> Rate {
        self.rate
    }

    fn from_price_factor(factor: &PriceFactor) -> Self {
        Self {
            choice_id: Some(factor.choice_id().clone()),
            rate: factor.rate(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrderItemError {
    ZeroQuantity,
    Modifier(ModifierError),
    Money(MoneyError),
    Entry(EntryError),
    ModifierPriceTotalMismatch { expected: Money, actual: Money },
    ModifierEntryIdCountMismatch { expected: usize, actual: usize },
}

impl fmt::Display for OrderItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroQuantity => f.write_str("order item quantity must be greater than zero"),
            Self::Modifier(error) => write!(f, "{error}"),
            Self::Money(error) => write!(f, "{error}"),
            Self::Entry(error) => write!(f, "{error}"),
            Self::ModifierPriceTotalMismatch { expected, actual } => write!(
                f,
                "modifier price total `{actual:?}` does not match contribution sum `{expected:?}`"
            ),
            Self::ModifierEntryIdCountMismatch { expected, actual } => write!(
                f,
                "expected {expected} modifier entry IDs but received {actual}"
            ),
        }
    }
}

impl std::error::Error for OrderItemError {}

fn validate_total(
    contributions: &[OrderItemPriceContribution],
    total: &Money,
) -> Result<(), OrderItemError> {
    let mut expected = Money::zero(total.currency().clone());

    for contribution in contributions {
        expected = expected
            .checked_add(contribution.amount())
            .map_err(OrderItemError::Money)?;
    }

    if &expected == total {
        Ok(())
    } else {
        Err(OrderItemError::ModifierPriceTotalMismatch {
            expected,
            actual: total.clone(),
        })
    }
}
