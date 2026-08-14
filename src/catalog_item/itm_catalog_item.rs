use std::collections::BTreeSet;
use std::fmt;

use crate::effect::Effect;
use crate::modifier::{
    Configuration as ModifierConfiguration, ModifierApplicability, ModifierError,
    ModifierPricingPolicy, Modifiers, PricedConfiguration, Selections,
};
use crate::primitives::consumer::ConsumerProfile;
use crate::primitives::ids::{CatalogItemId, LabelId, VariantId};
use crate::primitives::label::{Label, LabelError, ResolvedLabel};
use crate::primitives::money::{Money, MoneyError};
use crate::primitives::time::EvaluationTime;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CatalogItem {
    catalog_item_id: CatalogItemId,
    label: Label,
    variants: Vec<Variant>,
    modifiers: Modifiers,
    pricing_policy: ModifierPricingPolicy,
}

impl CatalogItem {
    pub fn new(
        catalog_item_id: CatalogItemId,
        title: impl Into<String>,
        variants: Vec<Variant>,
        modifiers: Modifiers,
    ) -> Result<Self, CatalogItemError> {
        Self::with_pricing_policy(
            catalog_item_id,
            title,
            variants,
            modifiers,
            ModifierPricingPolicy::default(),
        )
    }

    pub fn with_pricing_policy(
        catalog_item_id: CatalogItemId,
        title: impl Into<String>,
        variants: Vec<Variant>,
        modifiers: Modifiers,
        pricing_policy: ModifierPricingPolicy,
    ) -> Result<Self, CatalogItemError> {
        let title = title.into();

        if title.trim().is_empty() {
            return Err(CatalogItemError::EmptyCatalogItemTitle);
        }

        let label = generated_label(&format!("{}-TITLE", catalog_item_id.suffix()), title);
        Self::with_pricing_policy_labeled(
            catalog_item_id,
            label,
            variants,
            modifiers,
            pricing_policy,
        )
    }

    pub fn new_labeled(
        catalog_item_id: CatalogItemId,
        label: Label,
        variants: Vec<Variant>,
        modifiers: Modifiers,
    ) -> Result<Self, CatalogItemError> {
        Self::with_pricing_policy_labeled(
            catalog_item_id,
            label,
            variants,
            modifiers,
            ModifierPricingPolicy::default(),
        )
    }

    pub fn with_pricing_policy_labeled(
        catalog_item_id: CatalogItemId,
        label: Label,
        variants: Vec<Variant>,
        modifiers: Modifiers,
        pricing_policy: ModifierPricingPolicy,
    ) -> Result<Self, CatalogItemError> {
        if variants.is_empty() {
            return Err(CatalogItemError::CatalogItemHasNoVariants(catalog_item_id));
        }

        validate_variants(&variants)?;

        Ok(Self {
            catalog_item_id,
            label,
            variants,
            modifiers,
            pricing_policy,
        })
    }

    pub fn catalog_item_id(&self) -> &CatalogItemId {
        &self.catalog_item_id
    }

    pub fn title(&self) -> &str {
        self.label.default_text()
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn variants(&self) -> &[Variant] {
        &self.variants
    }

    pub fn modifiers(&self) -> &Modifiers {
        &self.modifiers
    }

    pub fn pricing_policy(&self) -> ModifierPricingPolicy {
        self.pricing_policy
    }

    pub fn variant(&self, variant_id: &VariantId) -> Option<&Variant> {
        self.variants
            .iter()
            .find(|variant| variant.variant_id() == variant_id)
    }

    pub fn configure_variant(
        &self,
        variant_id: &VariantId,
        selections: &Selections,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        self.configure_variant_with_inputs(variant_id, selections, &ConsumerProfile::empty(), None)
    }

    pub fn configure_variant_at(
        &self,
        variant_id: &VariantId,
        selections: &Selections,
        evaluation_time: &EvaluationTime,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        self.configure_variant_with_inputs(
            variant_id,
            selections,
            &ConsumerProfile::empty(),
            Some(evaluation_time),
        )
    }

    pub fn configure_variant_for_profile(
        &self,
        variant_id: &VariantId,
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        self.configure_variant_with_inputs(variant_id, selections, consumer_profile, None)
    }

    pub fn configure_variant_for_profile_at(
        &self,
        variant_id: &VariantId,
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
        evaluation_time: &EvaluationTime,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        self.configure_variant_with_inputs(
            variant_id,
            selections,
            consumer_profile,
            Some(evaluation_time),
        )
    }

    fn configure_variant_with_inputs(
        &self,
        variant_id: &VariantId,
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
        evaluation_time: Option<&EvaluationTime>,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        let variant = self
            .variant(variant_id)
            .ok_or_else(|| CatalogItemError::UnknownVariant(variant_id.clone()))?;

        let modifiers = if let Some(evaluation_time) = evaluation_time {
            self.modifiers.hydrate_with_applicability_and_profile_at(
                selections,
                variant.modifier_applicability(),
                consumer_profile,
                evaluation_time,
            )
        } else {
            self.modifiers.hydrate_with_applicability_and_profile(
                selections,
                variant.modifier_applicability(),
                consumer_profile,
            )
        }
        .map_err(CatalogItemError::Modifier)?;
        let modifier_price = modifiers
            .price(variant.invariant_price(), self.pricing_policy)
            .map_err(CatalogItemError::Modifier)?;
        let total_price = variant
            .invariant_price()
            .checked_add(modifier_price.total())
            .map_err(CatalogItemError::Money)?;

        Ok(ConfiguredCatalogItem {
            catalog_item_id: self.catalog_item_id.clone(),
            catalog_item_label_definition: self.label.clone(),
            catalog_item_label: self.label.resolve(consumer_profile)?,
            variant_id: variant.variant_id.clone(),
            variant_label_definition: variant.label.clone(),
            variant_label: variant.label.resolve(consumer_profile)?,
            effects: variant.effects.clone(),
            invariant_price: variant.invariant_price.clone(),
            modifiers,
            modifier_price,
            pricing_policy: self.pricing_policy,
            total_price,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variant {
    variant_id: VariantId,
    label: Label,
    invariant_price: Money,
    effects: Vec<Effect>,
    modifier_applicability: ModifierApplicability,
}

impl Variant {
    pub fn new(
        variant_id: VariantId,
        title: impl Into<String>,
        invariant_price: Money,
        effects: Vec<Effect>,
    ) -> Result<Self, CatalogItemError> {
        let title = title.into();

        if title.trim().is_empty() {
            return Err(CatalogItemError::EmptyVariantTitle);
        }

        let label = generated_label(&format!("{}-TITLE", variant_id.suffix()), title);
        Self::new_labeled(variant_id, label, invariant_price, effects)
    }

    pub fn new_labeled(
        variant_id: VariantId,
        label: Label,
        invariant_price: Money,
        effects: Vec<Effect>,
    ) -> Result<Self, CatalogItemError> {
        if invariant_price.amount_minor() < 0 {
            return Err(CatalogItemError::NegativeVariantPrice(variant_id));
        }

        Ok(Self {
            variant_id,
            label,
            invariant_price,
            effects,
            modifier_applicability: ModifierApplicability::all(),
        })
    }

    pub fn with_modifier_applicability(
        mut self,
        modifier_applicability: ModifierApplicability,
    ) -> Self {
        self.modifier_applicability = modifier_applicability;
        self
    }

    pub fn variant_id(&self) -> &VariantId {
        &self.variant_id
    }

    pub fn title(&self) -> &str {
        self.label.default_text()
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn invariant_price(&self) -> &Money {
        &self.invariant_price
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn modifier_applicability(&self) -> &ModifierApplicability {
        &self.modifier_applicability
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConfiguredCatalogItem {
    catalog_item_id: CatalogItemId,
    catalog_item_label_definition: Label,
    catalog_item_label: ResolvedLabel,
    variant_id: VariantId,
    variant_label_definition: Label,
    variant_label: ResolvedLabel,
    effects: Vec<Effect>,
    invariant_price: Money,
    modifiers: ModifierConfiguration,
    modifier_price: PricedConfiguration,
    pricing_policy: ModifierPricingPolicy,
    total_price: Money,
}

impl ConfiguredCatalogItem {
    pub fn catalog_item_id(&self) -> &CatalogItemId {
        &self.catalog_item_id
    }

    pub fn catalog_item_label(&self) -> &ResolvedLabel {
        &self.catalog_item_label
    }

    pub fn catalog_item_label_definition(&self) -> &Label {
        &self.catalog_item_label_definition
    }

    pub fn variant_id(&self) -> &VariantId {
        &self.variant_id
    }

    pub fn variant_label(&self) -> &ResolvedLabel {
        &self.variant_label
    }

    pub fn variant_label_definition(&self) -> &Label {
        &self.variant_label_definition
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn invariant_price(&self) -> &Money {
        &self.invariant_price
    }

    pub fn modifiers(&self) -> &ModifierConfiguration {
        &self.modifiers
    }

    pub fn modifier_price(&self) -> &PricedConfiguration {
        &self.modifier_price
    }

    pub fn pricing_policy(&self) -> ModifierPricingPolicy {
        self.pricing_policy
    }

    pub fn total_price(&self) -> &Money {
        &self.total_price
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CatalogItemError {
    EmptyCatalogItemTitle,
    EmptyVariantTitle,
    CatalogItemHasNoVariants(CatalogItemId),
    NegativeVariantPrice(VariantId),
    DuplicateVariant(VariantId),
    VariantCurrencyMismatch { left: VariantId, right: VariantId },
    UnknownVariant(VariantId),
    Modifier(ModifierError),
    Label(LabelError),
    Money(MoneyError),
}

impl fmt::Display for CatalogItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCatalogItemTitle => f.write_str("catalog item title cannot be empty"),
            Self::EmptyVariantTitle => f.write_str("variant title cannot be empty"),
            Self::CatalogItemHasNoVariants(catalog_item_id) => {
                write!(
                    f,
                    "catalog item `{catalog_item_id}` must have at least one variant"
                )
            }
            Self::NegativeVariantPrice(variant_id) => {
                write!(f, "variant `{variant_id}` cannot have a negative price")
            }
            Self::DuplicateVariant(variant_id) => write!(f, "duplicate variant `{variant_id}`"),
            Self::VariantCurrencyMismatch { left, right } => write!(
                f,
                "variant `{left}` and variant `{right}` must use the same currency"
            ),
            Self::UnknownVariant(variant_id) => write!(f, "unknown variant `{variant_id}`"),
            Self::Modifier(error) => write!(f, "{error}"),
            Self::Label(error) => write!(f, "{error}"),
            Self::Money(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CatalogItemError {}

impl From<LabelError> for CatalogItemError {
    fn from(error: LabelError) -> Self {
        Self::Label(error)
    }
}

fn validate_variants(variants: &[Variant]) -> Result<(), CatalogItemError> {
    let mut variant_ids = BTreeSet::new();
    let first_variant = variants
        .first()
        .expect("validate_variants is only called after non-empty validation");

    for variant in variants {
        if !variant_ids.insert(variant.variant_id.clone()) {
            return Err(CatalogItemError::DuplicateVariant(
                variant.variant_id.clone(),
            ));
        }

        if variant.invariant_price.currency() != first_variant.invariant_price.currency() {
            return Err(CatalogItemError::VariantCurrencyMismatch {
                left: first_variant.variant_id.clone(),
                right: variant.variant_id.clone(),
            });
        }
    }

    Ok(())
}

fn generated_label(label_suffix: &str, default: impl Into<String>) -> Label {
    let label_id = LabelId::from_suffix(label_suffix)
        .expect("generated label suffixes are based on validated IDs and static slots");
    Label::new(label_id, default).expect("generated labels are created after empty text validation")
}
