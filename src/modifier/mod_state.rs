use crate::effect::Effect;
use crate::modifier::mod_definition::Modifiers;
use crate::modifier::mod_error::ModifierError;
use crate::modifier::mod_price::{
    ChoicePrice, ModifierPricingPolicy, PriceContribution, PriceFactor, PricedConfiguration,
    branch_amount,
};
use crate::modifier::mod_selection::{
    ChoiceSelection, PromptSelection, SelectionSource, Selections,
};
use crate::primitives::ids::ComponentId;
use crate::primitives::label::{Label, ResolvedLabel};
use crate::primitives::money::Money;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Configuration {
    pub(super) prompts: Vec<PromptConfiguration>,
}

impl Configuration {
    pub fn prompts(&self) -> &[PromptConfiguration] {
        &self.prompts
    }

    pub fn prompt(&self, prompt_id: &ComponentId) -> Option<&PromptConfiguration> {
        self.prompts
            .iter()
            .find(|prompt| &prompt.prompt_id == prompt_id)
    }

    pub fn prompt_at(
        &self,
        prompt_id: &ComponentId,
        occurrence: usize,
    ) -> Option<&PromptConfiguration> {
        self.prompts
            .iter()
            .filter(|prompt| &prompt.prompt_id == prompt_id)
            .nth(occurrence)
    }

    pub fn dehydrate(&self) -> Selections {
        let mut selections = Selections::new();

        for prompt in &self.prompts {
            let choice_selections = prompt.dehydrate_choices();

            if !choice_selections.is_empty() {
                selections.push_prompt(PromptSelection::new(
                    prompt.prompt_id.clone(),
                    choice_selections,
                ));
            }
        }

        selections
    }

    pub fn price(
        &self,
        invariant_price: &Money,
        policy: ModifierPricingPolicy,
    ) -> Result<PricedConfiguration, ModifierError> {
        let priced = price_prompts(&self.prompts, invariant_price, policy)?;

        if let Some(factor) = priced.upward_factors.first() {
            return Err(ModifierError::UnconsumedPriceFactor(
                factor.choice_id().clone(),
            ));
        }

        let mut total = Money::zero(invariant_price.currency().clone());

        for contribution in &priced.contributions {
            total = total
                .checked_add(contribution.amount())
                .map_err(ModifierError::Money)?;
        }

        Ok(PricedConfiguration::new(priced.contributions, total))
    }

    pub fn snapshot(
        &self,
        invariant_price: &Money,
        policy: ModifierPricingPolicy,
    ) -> Result<ConfigurationSnapshot, ModifierError> {
        Ok(ConfigurationSnapshot {
            prompts: snapshot_prompts(&self.prompts),
            price: self.price(invariant_price, policy)?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PromptConfiguration {
    pub(super) prompt_id: ComponentId,
    pub(super) label: ResolvedLabel,
    pub(super) label_definition: Label,
    pub(super) description: Option<ResolvedLabel>,
    pub(super) description_definition: Option<Label>,
    pub(super) effects: Vec<Effect>,
    pub(super) choices: Vec<ChoiceConfiguration>,
}

impl PromptConfiguration {
    pub fn prompt_id(&self) -> &ComponentId {
        &self.prompt_id
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

    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(ResolvedLabel::value)
    }

    pub fn description_label(&self) -> Option<&ResolvedLabel> {
        self.description.as_ref()
    }

    pub fn description_label_definition(&self) -> Option<&Label> {
        self.description_definition.as_ref()
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn choices(&self) -> &[ChoiceConfiguration] {
        &self.choices
    }

    pub fn dehydrate(&self) -> Option<PromptSelection> {
        let choices = self.dehydrate_choices();

        if choices.is_empty() {
            return None;
        }

        Some(PromptSelection::new(self.prompt_id.clone(), choices))
    }

    fn dehydrate_choices(&self) -> Vec<ChoiceSelection> {
        self.choices
            .iter()
            .filter_map(ChoiceConfiguration::dehydrate)
            .collect()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChoiceConfiguration {
    pub(super) choice_id: ComponentId,
    pub(super) label: ResolvedLabel,
    pub(super) label_definition: Label,
    pub(super) quantity: u32,
    pub(super) source: SelectionSource,
    pub(super) effects: Vec<Effect>,
    pub(super) price: ChoicePrice,
    pub(super) modifiers: Option<Box<Configuration>>,
}

impl ChoiceConfiguration {
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

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn price_definition(&self) -> &ChoicePrice {
        &self.price
    }

    pub fn modifiers(&self) -> Option<&Configuration> {
        self.modifiers.as_deref()
    }

    pub fn dehydrate(&self) -> Option<ChoiceSelection> {
        let mut selection =
            ChoiceSelection::new(self.choice_id.clone(), self.quantity).with_source(self.source);

        if let Some(modifiers) = self.modifiers() {
            let nested = modifiers.dehydrate();

            if !nested.is_empty() {
                selection = selection.with_modifiers(nested);
            }
        }

        Some(selection)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConfigurationSnapshot {
    prompts: Vec<PromptSnapshot>,
    price: PricedConfiguration,
}

impl ConfigurationSnapshot {
    pub fn prompts(&self) -> &[PromptSnapshot] {
        &self.prompts
    }

    pub fn price(&self) -> &PricedConfiguration {
        &self.price
    }

    pub fn prompt(&self, prompt_id: &ComponentId) -> Option<&PromptSnapshot> {
        self.prompts
            .iter()
            .find(|prompt| prompt.prompt_id() == prompt_id)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PromptSnapshot {
    prompt_id: ComponentId,
    label: ResolvedLabel,
    label_definition: Label,
    description: Option<ResolvedLabel>,
    description_definition: Option<Label>,
    effects: Vec<Effect>,
    choices: Vec<ChoiceSnapshot>,
}

impl PromptSnapshot {
    pub fn prompt_id(&self) -> &ComponentId {
        &self.prompt_id
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

    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(ResolvedLabel::value)
    }

    pub fn description_label(&self) -> Option<&ResolvedLabel> {
        self.description.as_ref()
    }

    pub fn description_label_definition(&self) -> Option<&Label> {
        self.description_definition.as_ref()
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn choices(&self) -> &[ChoiceSnapshot] {
        &self.choices
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChoiceSnapshot {
    choice_id: ComponentId,
    label: ResolvedLabel,
    label_definition: Label,
    quantity: u32,
    source: SelectionSource,
    effects: Vec<Effect>,
    price: ChoicePrice,
    modifiers: Vec<PromptSnapshot>,
}

impl ChoiceSnapshot {
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

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn price_definition(&self) -> &ChoicePrice {
        &self.price
    }

    pub fn modifiers(&self) -> &[PromptSnapshot] {
        &self.modifiers
    }
}

fn snapshot_prompts(prompts: &[PromptConfiguration]) -> Vec<PromptSnapshot> {
    prompts
        .iter()
        .map(|prompt| PromptSnapshot {
            prompt_id: prompt.prompt_id.clone(),
            label: prompt.label.clone(),
            label_definition: prompt.label_definition.clone(),
            description: prompt.description.clone(),
            description_definition: prompt.description_definition.clone(),
            effects: prompt.effects.clone(),
            choices: prompt
                .choices
                .iter()
                .map(|choice| ChoiceSnapshot {
                    choice_id: choice.choice_id.clone(),
                    label: choice.label.clone(),
                    label_definition: choice.label_definition.clone(),
                    quantity: choice.quantity,
                    source: choice.source,
                    effects: choice.effects.clone(),
                    price: choice.price.clone(),
                    modifiers: choice
                        .modifiers()
                        .map(|modifiers| snapshot_prompts(modifiers.prompts()))
                        .unwrap_or_default(),
                })
                .collect(),
        })
        .collect()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidatedChoiceSelection {
    pub(super) choice_id: ComponentId,
    pub(super) quantity: u32,
    pub(super) source: SelectionSource,
    pub(super) effects: Vec<Effect>,
    pub(super) price: ChoicePrice,
    pub(super) modifiers: Option<Box<Modifiers>>,
}

impl ValidatedChoiceSelection {
    pub fn choice_id(&self) -> &ComponentId {
        &self.choice_id
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

    pub fn modifiers(&self) -> Option<&Modifiers> {
        self.modifiers.as_deref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct PriceWalk {
    contributions: Vec<PriceContribution>,
    upward_factors: Vec<PriceFactor>,
}

impl PriceWalk {
    fn empty() -> Self {
        Self {
            contributions: Vec::new(),
            upward_factors: Vec::new(),
        }
    }

    fn append(&mut self, mut other: Self) {
        self.contributions.append(&mut other.contributions);
        self.upward_factors.append(&mut other.upward_factors);
    }
}

fn price_prompts(
    prompts: &[PromptConfiguration],
    invariant_price: &Money,
    policy: ModifierPricingPolicy,
) -> Result<PriceWalk, ModifierError> {
    let mut priced = PriceWalk::empty();

    for prompt in prompts {
        for choice in prompt.choices() {
            priced.append(price_choice(choice, invariant_price, policy)?);
        }
    }

    Ok(priced)
}

fn price_choice(
    choice: &ChoiceConfiguration,
    invariant_price: &Money,
    policy: ModifierPricingPolicy,
) -> Result<PriceWalk, ModifierError> {
    let mut priced = if let Some(modifiers) = choice.modifiers() {
        price_prompts(modifiers.prompts(), invariant_price, policy)?
    } else {
        PriceWalk::empty()
    };

    let mut own_upward_factors = Vec::new();

    if let Some(factor) = choice.price_definition().factor() {
        for _ in 0..choice.quantity() {
            own_upward_factors.push(PriceFactor::new(choice.choice_id.clone(), factor));
        }
    }

    if choice.price_definition().has_amount() {
        let factors = std::mem::take(&mut priced.upward_factors);

        if !(policy.defaults_are_free() && choice.source() == SelectionSource::Default) {
            let amount = branch_amount(
                invariant_price,
                choice.price_definition(),
                choice.quantity(),
                &factors,
                policy.rounding(),
            )?;

            priced.contributions.push(PriceContribution::new(
                choice.choice_id.clone(),
                choice.label.clone(),
                choice.label_definition.clone(),
                choice.quantity(),
                choice.source(),
                amount,
                choice.price_definition(),
                factors,
            ));
        }

        priced.upward_factors = own_upward_factors;
    } else {
        priced.upward_factors.append(&mut own_upward_factors);
    }

    Ok(priced)
}
