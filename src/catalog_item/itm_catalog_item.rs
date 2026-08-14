use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::effect::Effect;
use crate::modifier::{
    Configuration as ModifierConfiguration, ModifierApplicability, ModifierError,
    ModifierPricingPolicy, Modifiers, PricedConfiguration, Selections,
};
use crate::primitives::consumer::ConsumerProfile;
use crate::primitives::ids::{CatalogItemId, LabelId, VariantDimensionId, VariantId};
use crate::primitives::label::{Label, LabelError, ResolvedLabel};
use crate::primitives::media::MediaCollection;
use crate::primitives::money::{Money, MoneyError};
use crate::primitives::time::EvaluationTime;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct VariantSettings {
    allow_free_variant: bool,
}

impl VariantSettings {
    pub const fn new() -> Self {
        Self {
            allow_free_variant: false,
        }
    }

    pub const fn with_allow_free_variant(mut self, allow_free_variant: bool) -> Self {
        self.allow_free_variant = allow_free_variant;
        self
    }

    pub const fn allow_free_variant(self) -> bool {
        self.allow_free_variant
    }
}

#[doc = include_str!("catalog-item.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CatalogItem {
    catalog_item_id: CatalogItemId,
    label: Label,
    description: Option<Label>,
    media: MediaCollection,
    dimensions: Vec<VariantDimension>,
    variant_matches: Vec<VariantMatch>,
    variant_settings: VariantSettings,
    modifiers: Modifiers,
    pricing_policy: ModifierPricingPolicy,
}

impl CatalogItem {
    pub fn new(
        catalog_item_id: CatalogItemId,
        title: impl Into<String>,
        dimensions: Vec<VariantDimension>,
        variant_matches: Vec<VariantMatch>,
        modifiers: Modifiers,
    ) -> Result<Self, CatalogItemError> {
        Self::with_pricing_policy(
            catalog_item_id,
            title,
            dimensions,
            variant_matches,
            modifiers,
            ModifierPricingPolicy::default(),
        )
    }

    pub fn with_variant_settings(
        catalog_item_id: CatalogItemId,
        title: impl Into<String>,
        dimensions: Vec<VariantDimension>,
        variant_matches: Vec<VariantMatch>,
        variant_settings: VariantSettings,
        modifiers: Modifiers,
    ) -> Result<Self, CatalogItemError> {
        Self::with_variant_settings_and_pricing_policy(
            catalog_item_id,
            title,
            dimensions,
            variant_matches,
            variant_settings,
            modifiers,
            ModifierPricingPolicy::default(),
        )
    }

    pub fn with_pricing_policy(
        catalog_item_id: CatalogItemId,
        title: impl Into<String>,
        dimensions: Vec<VariantDimension>,
        variant_matches: Vec<VariantMatch>,
        modifiers: Modifiers,
        pricing_policy: ModifierPricingPolicy,
    ) -> Result<Self, CatalogItemError> {
        Self::with_variant_settings_and_pricing_policy(
            catalog_item_id,
            title,
            dimensions,
            variant_matches,
            VariantSettings::default(),
            modifiers,
            pricing_policy,
        )
    }

    pub fn with_variant_settings_and_pricing_policy(
        catalog_item_id: CatalogItemId,
        title: impl Into<String>,
        dimensions: Vec<VariantDimension>,
        variant_matches: Vec<VariantMatch>,
        variant_settings: VariantSettings,
        modifiers: Modifiers,
        pricing_policy: ModifierPricingPolicy,
    ) -> Result<Self, CatalogItemError> {
        let title = title.into();

        if title.trim().is_empty() {
            return Err(CatalogItemError::EmptyCatalogItemTitle);
        }

        let label = generated_label(&format!("{}-TITLE", catalog_item_id.suffix()), title);
        Self::with_variant_settings_and_pricing_policy_labeled(
            catalog_item_id,
            label,
            dimensions,
            variant_matches,
            variant_settings,
            modifiers,
            pricing_policy,
        )
    }

    pub fn new_labeled(
        catalog_item_id: CatalogItemId,
        label: Label,
        dimensions: Vec<VariantDimension>,
        variant_matches: Vec<VariantMatch>,
        modifiers: Modifiers,
    ) -> Result<Self, CatalogItemError> {
        Self::with_pricing_policy_labeled(
            catalog_item_id,
            label,
            dimensions,
            variant_matches,
            modifiers,
            ModifierPricingPolicy::default(),
        )
    }

    pub fn with_variant_settings_labeled(
        catalog_item_id: CatalogItemId,
        label: Label,
        dimensions: Vec<VariantDimension>,
        variant_matches: Vec<VariantMatch>,
        variant_settings: VariantSettings,
        modifiers: Modifiers,
    ) -> Result<Self, CatalogItemError> {
        Self::with_variant_settings_and_pricing_policy_labeled(
            catalog_item_id,
            label,
            dimensions,
            variant_matches,
            variant_settings,
            modifiers,
            ModifierPricingPolicy::default(),
        )
    }

    pub fn with_pricing_policy_labeled(
        catalog_item_id: CatalogItemId,
        label: Label,
        dimensions: Vec<VariantDimension>,
        variant_matches: Vec<VariantMatch>,
        modifiers: Modifiers,
        pricing_policy: ModifierPricingPolicy,
    ) -> Result<Self, CatalogItemError> {
        Self::with_variant_settings_and_pricing_policy_labeled(
            catalog_item_id,
            label,
            dimensions,
            variant_matches,
            VariantSettings::default(),
            modifiers,
            pricing_policy,
        )
    }

    pub fn with_variant_settings_and_pricing_policy_labeled(
        catalog_item_id: CatalogItemId,
        label: Label,
        dimensions: Vec<VariantDimension>,
        mut variant_matches: Vec<VariantMatch>,
        variant_settings: VariantSettings,
        modifiers: Modifiers,
        pricing_policy: ModifierPricingPolicy,
    ) -> Result<Self, CatalogItemError> {
        let variant_locations = validate_dimensions(&dimensions)?;
        validate_and_canonicalize_matches(
            &catalog_item_id,
            &dimensions,
            &variant_locations,
            &mut variant_matches,
            variant_settings,
        )?;

        Ok(Self {
            catalog_item_id,
            label,
            description: None,
            media: MediaCollection::empty(),
            dimensions,
            variant_matches,
            variant_settings,
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

    pub fn with_description(mut self, description: Label) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_media(mut self, media: MediaCollection) -> Self {
        self.media = media;
        self
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(Label::default_text)
    }

    pub fn description_label(&self) -> Option<&Label> {
        self.description.as_ref()
    }

    pub fn media(&self) -> &MediaCollection {
        &self.media
    }

    pub fn dimensions(&self) -> &[VariantDimension] {
        &self.dimensions
    }

    pub fn variant_matches(&self) -> &[VariantMatch] {
        &self.variant_matches
    }

    pub fn variant_settings(&self) -> VariantSettings {
        self.variant_settings
    }

    pub fn variant(&self, variant_id: &VariantId) -> Option<&Variant> {
        self.dimensions
            .iter()
            .find_map(|dimension| dimension.variant(variant_id))
    }

    pub fn variant_match(&self, variant_ids: &[VariantId]) -> Option<&VariantMatch> {
        let canonical = self.canonical_variant_ids(variant_ids).ok()?;
        self.variant_matches
            .iter()
            .find(|variant_match| variant_match.variant_ids == canonical)
    }

    pub fn default_variant_match(&self) -> Option<&VariantMatch> {
        let deepest: Vec<_> = self
            .variant_matches
            .iter()
            .filter(|variant_match| self.is_deepest(variant_match))
            .collect();

        deepest
            .iter()
            .copied()
            .find(|variant_match| variant_match.is_default())
            .or_else(|| (deepest.len() == 1).then(|| deepest[0]))
    }

    pub fn default_variant_ids(&self) -> Option<&[VariantId]> {
        self.default_variant_match().map(VariantMatch::variant_ids)
    }

    pub fn modifiers(&self) -> &Modifiers {
        &self.modifiers
    }

    pub fn pricing_policy(&self) -> ModifierPricingPolicy {
        self.pricing_policy
    }

    pub fn configure(
        &self,
        selections: &Selections,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        let variant_ids = self
            .default_variant_ids()
            .ok_or_else(|| {
                CatalogItemError::VariantSelectionRequired(self.catalog_item_id.clone())
            })?
            .to_vec();
        self.configure_variants(&variant_ids, selections)
    }

    pub fn configure_at(
        &self,
        selections: &Selections,
        evaluation_time: &EvaluationTime,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        let variant_ids = self
            .default_variant_ids()
            .ok_or_else(|| {
                CatalogItemError::VariantSelectionRequired(self.catalog_item_id.clone())
            })?
            .to_vec();
        self.configure_variants_at(&variant_ids, selections, evaluation_time)
    }

    pub fn configure_for_profile(
        &self,
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        let variant_ids = self
            .default_variant_ids()
            .ok_or_else(|| {
                CatalogItemError::VariantSelectionRequired(self.catalog_item_id.clone())
            })?
            .to_vec();
        self.configure_variants_for_profile(&variant_ids, selections, consumer_profile)
    }

    pub fn configure_for_profile_at(
        &self,
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
        evaluation_time: &EvaluationTime,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        let variant_ids = self
            .default_variant_ids()
            .ok_or_else(|| {
                CatalogItemError::VariantSelectionRequired(self.catalog_item_id.clone())
            })?
            .to_vec();
        self.configure_variants_for_profile_at(
            &variant_ids,
            selections,
            consumer_profile,
            evaluation_time,
        )
    }

    pub fn configure_variants(
        &self,
        variant_ids: &[VariantId],
        selections: &Selections,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        self.configure_variants_with_inputs(
            variant_ids,
            selections,
            &ConsumerProfile::empty(),
            None,
        )
    }

    pub fn configure_variants_at(
        &self,
        variant_ids: &[VariantId],
        selections: &Selections,
        evaluation_time: &EvaluationTime,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        self.configure_variants_with_inputs(
            variant_ids,
            selections,
            &ConsumerProfile::empty(),
            Some(evaluation_time),
        )
    }

    pub fn configure_variants_for_profile(
        &self,
        variant_ids: &[VariantId],
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        self.configure_variants_with_inputs(variant_ids, selections, consumer_profile, None)
    }

    pub fn configure_variants_for_profile_at(
        &self,
        variant_ids: &[VariantId],
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
        evaluation_time: &EvaluationTime,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        self.configure_variants_with_inputs(
            variant_ids,
            selections,
            consumer_profile,
            Some(evaluation_time),
        )
    }

    fn configure_variants_with_inputs(
        &self,
        variant_ids: &[VariantId],
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
        evaluation_time: Option<&EvaluationTime>,
    ) -> Result<ConfiguredCatalogItem, CatalogItemError> {
        let canonical_variant_ids = self.canonical_variant_ids(variant_ids)?;
        let variant_match = self
            .variant_matches
            .iter()
            .find(|candidate| {
                self.is_deepest(candidate) && candidate.variant_ids == canonical_variant_ids
            })
            .ok_or_else(|| {
                CatalogItemError::VariantCombinationDoesNotExist(canonical_variant_ids.clone())
            })?;
        let invariant_price = variant_match.price.clone();

        let modifiers = if let Some(evaluation_time) = evaluation_time {
            self.modifiers.hydrate_with_applicability_and_profile_at(
                selections,
                variant_match.modifier_applicability(),
                consumer_profile,
                evaluation_time,
            )
        } else {
            self.modifiers.hydrate_with_applicability_and_profile(
                selections,
                variant_match.modifier_applicability(),
                consumer_profile,
            )
        }
        .map_err(CatalogItemError::Modifier)?;
        let modifier_price = modifiers
            .price(&invariant_price, self.pricing_policy)
            .map_err(CatalogItemError::Modifier)?;
        let total_price = invariant_price
            .checked_add(modifier_price.total())
            .map_err(CatalogItemError::Money)?;

        let variant_label_definitions: Vec<_> = canonical_variant_ids
            .iter()
            .map(|variant_id| {
                self.variant(variant_id)
                    .expect("canonical variant IDs came from this catalog item")
                    .label
                    .clone()
            })
            .collect();
        let variant_labels = variant_label_definitions
            .iter()
            .map(|label| label.resolve(consumer_profile))
            .collect::<Result<Vec<_>, _>>()?;
        let variant_match_label = variant_match
            .label
            .as_ref()
            .map(|label| label.resolve(consumer_profile))
            .transpose()?;

        Ok(ConfiguredCatalogItem {
            catalog_item_id: self.catalog_item_id.clone(),
            catalog_item_label_definition: self.label.clone(),
            catalog_item_label: self.label.resolve(consumer_profile)?,
            variant_ids: canonical_variant_ids,
            variant_label_definitions,
            variant_labels,
            variant_match_label_definition: variant_match.label.clone(),
            variant_match_label,
            effects: variant_match.effects.clone(),
            invariant_price,
            modifiers,
            modifier_price,
            pricing_policy: self.pricing_policy,
            total_price,
        })
    }

    fn canonical_variant_ids(
        &self,
        variant_ids: &[VariantId],
    ) -> Result<Vec<VariantId>, CatalogItemError> {
        let locations = variant_locations(&self.dimensions);
        canonicalize_variant_ids(variant_ids, &self.dimensions, &locations)
    }

    fn is_deepest(&self, variant_match: &VariantMatch) -> bool {
        is_deepest_match(variant_match, &self.variant_matches)
    }
}

#[doc = include_str!("variant-dimension.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VariantDimension {
    variant_dimension_id: VariantDimensionId,
    label: Label,
    variants: Vec<Variant>,
}

impl VariantDimension {
    pub fn new(
        variant_dimension_id: VariantDimensionId,
        title: impl Into<String>,
        variants: Vec<Variant>,
    ) -> Result<Self, CatalogItemError> {
        let title = title.into();

        if title.trim().is_empty() {
            return Err(CatalogItemError::EmptyVariantDimensionTitle);
        }

        let label = generated_label(&format!("{}-TITLE", variant_dimension_id.suffix()), title);
        Self::new_labeled(variant_dimension_id, label, variants)
    }

    pub fn new_labeled(
        variant_dimension_id: VariantDimensionId,
        label: Label,
        variants: Vec<Variant>,
    ) -> Result<Self, CatalogItemError> {
        if variants.is_empty() {
            return Err(CatalogItemError::VariantDimensionHasNoVariants(
                variant_dimension_id,
            ));
        }

        Ok(Self {
            variant_dimension_id,
            label,
            variants,
        })
    }

    pub fn variant_dimension_id(&self) -> &VariantDimensionId {
        &self.variant_dimension_id
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

    pub fn variant(&self, variant_id: &VariantId) -> Option<&Variant> {
        self.variants
            .iter()
            .find(|variant| variant.variant_id() == variant_id)
    }
}

#[doc = include_str!("variant.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variant {
    variant_id: VariantId,
    label: Label,
    description: Option<Label>,
    media: MediaCollection,
}

impl Variant {
    pub fn new(variant_id: VariantId, title: impl Into<String>) -> Result<Self, CatalogItemError> {
        let title = title.into();

        if title.trim().is_empty() {
            return Err(CatalogItemError::EmptyVariantTitle);
        }

        let label = generated_label(&format!("{}-TITLE", variant_id.suffix()), title);
        Ok(Self::new_labeled(variant_id, label))
    }

    pub fn new_labeled(variant_id: VariantId, label: Label) -> Self {
        Self {
            variant_id,
            label,
            description: None,
            media: MediaCollection::empty(),
        }
    }

    pub fn with_description(mut self, description: Label) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_media(mut self, media: MediaCollection) -> Self {
        self.media = media;
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

    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(Label::default_text)
    }

    pub fn description_label(&self) -> Option<&Label> {
        self.description.as_ref()
    }

    pub fn media(&self) -> &MediaCollection {
        &self.media
    }
}

#[doc = include_str!("variant-match.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VariantMatch {
    variant_ids: Vec<VariantId>,
    price: Money,
    label: Option<Label>,
    description: Option<Label>,
    media: MediaCollection,
    is_default: bool,
    effects: Vec<Effect>,
    modifier_applicability: ModifierApplicability,
}

impl VariantMatch {
    pub fn new(
        variant_ids: Vec<VariantId>,
        price: Money,
        effects: Vec<Effect>,
    ) -> Result<Self, CatalogItemError> {
        let mut seen = BTreeSet::new();
        for variant_id in &variant_ids {
            if !seen.insert(variant_id.clone()) {
                return Err(CatalogItemError::DuplicateVariantInMatch(
                    variant_id.clone(),
                ));
            }
        }

        if price.amount_minor() < 0 {
            return Err(CatalogItemError::NegativeVariantPrice(variant_ids));
        }

        Ok(Self {
            variant_ids,
            price,
            label: None,
            description: None,
            media: MediaCollection::empty(),
            is_default: false,
            effects,
            modifier_applicability: ModifierApplicability::all(),
        })
    }

    pub fn with_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn with_modifier_applicability(
        mut self,
        modifier_applicability: ModifierApplicability,
    ) -> Self {
        self.modifier_applicability = modifier_applicability;
        self
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_description(mut self, description: Label) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_media(mut self, media: MediaCollection) -> Self {
        self.media = media;
        self
    }

    pub fn variant_ids(&self) -> &[VariantId] {
        &self.variant_ids
    }

    pub fn price(&self) -> &Money {
        &self.price
    }

    pub fn title(&self) -> Option<&str> {
        self.label.as_ref().map(Label::default_text)
    }

    pub fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(Label::default_text)
    }

    pub fn description_label(&self) -> Option<&Label> {
        self.description.as_ref()
    }

    pub fn media(&self) -> &MediaCollection {
        &self.media
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn modifier_applicability(&self) -> &ModifierApplicability {
        &self.modifier_applicability
    }
}

#[doc = include_str!("configured-catalog-item.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConfiguredCatalogItem {
    catalog_item_id: CatalogItemId,
    catalog_item_label_definition: Label,
    catalog_item_label: ResolvedLabel,
    variant_ids: Vec<VariantId>,
    variant_label_definitions: Vec<Label>,
    variant_labels: Vec<ResolvedLabel>,
    variant_match_label_definition: Option<Label>,
    variant_match_label: Option<ResolvedLabel>,
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

    pub fn variant_ids(&self) -> &[VariantId] {
        &self.variant_ids
    }

    pub fn variant_labels(&self) -> &[ResolvedLabel] {
        &self.variant_labels
    }

    pub fn variant_label_definitions(&self) -> &[Label] {
        &self.variant_label_definitions
    }

    pub fn variant_match_label(&self) -> Option<&ResolvedLabel> {
        self.variant_match_label.as_ref()
    }

    pub fn variant_match_label_definition(&self) -> Option<&Label> {
        self.variant_match_label_definition.as_ref()
    }

    pub fn variant_title(&self) -> Option<String> {
        if let Some(label) = &self.variant_match_label {
            return Some(label.value().to_owned());
        }

        (!self.variant_labels.is_empty()).then(|| {
            self.variant_labels
                .iter()
                .map(ResolvedLabel::value)
                .collect::<Vec<_>>()
                .join(", ")
        })
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
    EmptyVariantDimensionTitle,
    EmptyVariantTitle,
    VariantDimensionHasNoVariants(VariantDimensionId),
    DuplicateVariantDimension(VariantDimensionId),
    DuplicateVariant(VariantId),
    CatalogItemHasNoVariantMatches(CatalogItemId),
    DuplicateVariantInMatch(VariantId),
    UnknownVariant(VariantId),
    MultipleVariantsForDimension {
        variant_dimension_id: VariantDimensionId,
        left: VariantId,
        right: VariantId,
    },
    DuplicateVariantMatch(Vec<VariantId>),
    NegativeVariantPrice(Vec<VariantId>),
    FreeVariantNotAllowed(Vec<VariantId>),
    VariantCurrencyMismatch {
        left: Vec<VariantId>,
        right: Vec<VariantId>,
    },
    DefaultRequiresConcreteVariantMatch(Vec<VariantId>),
    MultipleDefaultVariantMatches {
        left: Vec<VariantId>,
        right: Vec<VariantId>,
    },
    VariantSelectionRequired(CatalogItemId),
    VariantCombinationDoesNotExist(Vec<VariantId>),
    Modifier(ModifierError),
    Label(LabelError),
    Money(MoneyError),
}

impl fmt::Display for CatalogItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCatalogItemTitle => f.write_str("catalog item title cannot be empty"),
            Self::EmptyVariantDimensionTitle => {
                f.write_str("variant dimension title cannot be empty")
            }
            Self::EmptyVariantTitle => f.write_str("variant title cannot be empty"),
            Self::VariantDimensionHasNoVariants(dimension_id) => write!(
                f,
                "variant dimension `{dimension_id}` must have at least one variant"
            ),
            Self::DuplicateVariantDimension(dimension_id) => {
                write!(f, "duplicate variant dimension `{dimension_id}`")
            }
            Self::DuplicateVariant(variant_id) => write!(f, "duplicate variant `{variant_id}`"),
            Self::CatalogItemHasNoVariantMatches(catalog_item_id) => write!(
                f,
                "catalog item `{catalog_item_id}` must have at least one variant match"
            ),
            Self::DuplicateVariantInMatch(variant_id) => {
                write!(f, "variant match contains duplicate variant `{variant_id}`")
            }
            Self::UnknownVariant(variant_id) => write!(f, "unknown variant `{variant_id}`"),
            Self::MultipleVariantsForDimension {
                variant_dimension_id,
                left,
                right,
            } => write!(
                f,
                "variant match cannot contain both `{left}` and `{right}` from dimension `{variant_dimension_id}`"
            ),
            Self::DuplicateVariantMatch(variant_ids) => {
                write!(
                    f,
                    "duplicate variant match `{}`",
                    display_variant_ids(variant_ids)
                )
            }
            Self::NegativeVariantPrice(variant_ids) => write!(
                f,
                "variant match `{}` cannot have a negative price",
                display_variant_ids(variant_ids)
            ),
            Self::FreeVariantNotAllowed(variant_ids) => write!(
                f,
                "variant match `{}` cannot be free unless `allow_free_variant` is enabled",
                display_variant_ids(variant_ids)
            ),
            Self::VariantCurrencyMismatch { left, right } => write!(
                f,
                "variant matches `{}` and `{}` must use the same currency",
                display_variant_ids(left),
                display_variant_ids(right)
            ),
            Self::DefaultRequiresConcreteVariantMatch(variant_ids) => write!(
                f,
                "default marker requires a concrete variant match, not `{}`",
                display_variant_ids(variant_ids)
            ),
            Self::MultipleDefaultVariantMatches { left, right } => write!(
                f,
                "variant matches `{}` and `{}` cannot both be the default",
                display_variant_ids(left),
                display_variant_ids(right)
            ),
            Self::VariantSelectionRequired(catalog_item_id) => write!(
                f,
                "catalog item `{catalog_item_id}` requires an explicit variant selection"
            ),
            Self::VariantCombinationDoesNotExist(variant_ids) => write!(
                f,
                "variant combination `{}` does not exist",
                display_variant_ids(variant_ids)
            ),
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

fn validate_dimensions(
    dimensions: &[VariantDimension],
) -> Result<BTreeMap<VariantId, usize>, CatalogItemError> {
    let mut dimension_ids = BTreeSet::new();
    let mut locations = BTreeMap::new();

    for (dimension_index, dimension) in dimensions.iter().enumerate() {
        if !dimension_ids.insert(dimension.variant_dimension_id.clone()) {
            return Err(CatalogItemError::DuplicateVariantDimension(
                dimension.variant_dimension_id.clone(),
            ));
        }

        for variant in &dimension.variants {
            if locations
                .insert(variant.variant_id.clone(), dimension_index)
                .is_some()
            {
                return Err(CatalogItemError::DuplicateVariant(
                    variant.variant_id.clone(),
                ));
            }
        }
    }

    Ok(locations)
}

fn validate_and_canonicalize_matches(
    catalog_item_id: &CatalogItemId,
    dimensions: &[VariantDimension],
    variant_locations: &BTreeMap<VariantId, usize>,
    variant_matches: &mut [VariantMatch],
    variant_settings: VariantSettings,
) -> Result<(), CatalogItemError> {
    if variant_matches.is_empty() {
        return Err(CatalogItemError::CatalogItemHasNoVariantMatches(
            catalog_item_id.clone(),
        ));
    }

    let mut match_keys = BTreeSet::new();
    let mut first_match_price: Option<(Vec<VariantId>, Money)> = None;

    for variant_match in variant_matches.iter_mut() {
        variant_match.variant_ids =
            canonicalize_variant_ids(&variant_match.variant_ids, dimensions, variant_locations)?;

        if !match_keys.insert(variant_match.variant_ids.clone()) {
            return Err(CatalogItemError::DuplicateVariantMatch(
                variant_match.variant_ids.clone(),
            ));
        }

        if let Some((left_ids, left_price)) = &first_match_price {
            if variant_match.price.currency() != left_price.currency() {
                return Err(CatalogItemError::VariantCurrencyMismatch {
                    left: left_ids.clone(),
                    right: variant_match.variant_ids.clone(),
                });
            }
        } else {
            first_match_price = Some((
                variant_match.variant_ids.clone(),
                variant_match.price.clone(),
            ));
        }
    }

    let mut default_match: Option<Vec<VariantId>> = None;
    for variant_match in variant_matches.iter() {
        let is_deepest = is_deepest_match(variant_match, variant_matches);

        if is_deepest
            && variant_match.price.amount_minor() == 0
            && !variant_settings.allow_free_variant()
        {
            return Err(CatalogItemError::FreeVariantNotAllowed(
                variant_match.variant_ids.clone(),
            ));
        }

        if variant_match.is_default {
            if !is_deepest {
                return Err(CatalogItemError::DefaultRequiresConcreteVariantMatch(
                    variant_match.variant_ids.clone(),
                ));
            }

            if let Some(left) = default_match {
                return Err(CatalogItemError::MultipleDefaultVariantMatches {
                    left,
                    right: variant_match.variant_ids.clone(),
                });
            }
            default_match = Some(variant_match.variant_ids.clone());
        }
    }

    Ok(())
}

fn variant_locations(dimensions: &[VariantDimension]) -> BTreeMap<VariantId, usize> {
    dimensions
        .iter()
        .enumerate()
        .flat_map(|(dimension_index, dimension)| {
            dimension
                .variants
                .iter()
                .map(move |variant| (variant.variant_id.clone(), dimension_index))
        })
        .collect()
}

fn canonicalize_variant_ids(
    variant_ids: &[VariantId],
    dimensions: &[VariantDimension],
    variant_locations: &BTreeMap<VariantId, usize>,
) -> Result<Vec<VariantId>, CatalogItemError> {
    let mut by_dimension: BTreeMap<usize, VariantId> = BTreeMap::new();

    for variant_id in variant_ids {
        let dimension_index = variant_locations
            .get(variant_id)
            .copied()
            .ok_or_else(|| CatalogItemError::UnknownVariant(variant_id.clone()))?;

        if let Some(left) = by_dimension.insert(dimension_index, variant_id.clone()) {
            if left == *variant_id {
                return Err(CatalogItemError::DuplicateVariantInMatch(
                    variant_id.clone(),
                ));
            }

            return Err(CatalogItemError::MultipleVariantsForDimension {
                variant_dimension_id: dimensions[dimension_index].variant_dimension_id.clone(),
                left,
                right: variant_id.clone(),
            });
        }
    }

    Ok(by_dimension.into_values().collect())
}

fn is_subset(candidate: &[VariantId], concrete: &[VariantId]) -> bool {
    candidate
        .iter()
        .all(|variant_id| concrete.contains(variant_id))
}

fn is_deepest_match(candidate: &VariantMatch, variant_matches: &[VariantMatch]) -> bool {
    !variant_matches.iter().any(|other| {
        other.variant_ids.len() > candidate.variant_ids.len()
            && is_subset(&candidate.variant_ids, &other.variant_ids)
    })
}

fn display_variant_ids(variant_ids: &[VariantId]) -> String {
    if variant_ids.is_empty() {
        return "<empty>".to_owned();
    }

    variant_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" + ")
}

fn generated_label(label_suffix: &str, default: impl Into<String>) -> Label {
    let label_id = LabelId::from_suffix(label_suffix)
        .expect("generated label suffixes are based on validated IDs and static slots");
    Label::new(label_id, default).expect("generated labels are created after empty text validation")
}
