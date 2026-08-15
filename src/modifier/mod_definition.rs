use std::collections::{BTreeMap, BTreeSet};

use crate::effect::Effect;
use crate::modifier::mod_error::{ChoiceInputError, ModifierError};
use crate::modifier::mod_price::ChoicePrice;
use crate::modifier::mod_rule::{Rule, SelectionBounds};
use crate::modifier::mod_selection::{
    ChoiceInputValue, ChoiceSelection, PromptSelection, SelectionCandidate, SelectionSource,
    Selections,
};
use crate::modifier::mod_state::{
    ChoiceConfiguration, ChoiceInputConfiguration, Configuration, PromptConfiguration,
    ValidatedChoiceSelection,
};
use crate::modifier::mod_walk::ModifierNode;
use crate::primitives::consumer::ConsumerProfile;
use crate::primitives::ids::{ComponentId, LabelId};
use crate::primitives::label::Label;
use crate::primitives::media::MediaCollection;
use crate::primitives::schedule::Schedule;
use crate::primitives::time::EvaluationTime;

#[doc = include_str!("modifiers.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Modifiers {
    label: Option<Label>,
    prompts: Vec<Prompt>,
}

impl Modifiers {
    pub fn new(prompts: Vec<Prompt>) -> Self {
        Self {
            label: None,
            prompts,
        }
    }

    pub fn titled(title: impl Into<String>, prompts: Vec<Prompt>) -> Self {
        Self {
            label: Some(generated_label("MODIFIERS-TITLE", title)),
            prompts,
        }
    }

    pub fn titled_labeled(label: Label, prompts: Vec<Prompt>) -> Self {
        Self {
            label: Some(label),
            prompts,
        }
    }

    pub fn title(&self) -> Option<&str> {
        self.label.as_ref().map(Label::default_text)
    }

    pub fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }

    pub fn prompts(&self) -> &[Prompt] {
        &self.prompts
    }

    pub fn walk(&self, visit: &mut impl FnMut(ModifierNode<'_>)) {
        visit(ModifierNode::Modifiers(self));

        for prompt in &self.prompts {
            prompt.walk(visit);
        }
    }

    pub fn walk_visible_at(
        &self,
        evaluation_time: &EvaluationTime,
        visit: &mut impl FnMut(ModifierNode<'_>),
    ) {
        visit(ModifierNode::Modifiers(self));

        for prompt in &self.prompts {
            prompt.walk_visible_at(evaluation_time, visit);
        }
    }

    pub fn hydrate(&self, selections: &Selections) -> Result<Configuration, ModifierError> {
        let applicability = ModifierApplicability::all();
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_with_inputs(
            selections,
            HydrateInputs::new(&applicability, &consumer_profile, None),
        )
    }

    pub fn hydrate_at(
        &self,
        selections: &Selections,
        evaluation_time: &EvaluationTime,
    ) -> Result<Configuration, ModifierError> {
        let applicability = ModifierApplicability::all();
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_with_inputs(
            selections,
            HydrateInputs::new(&applicability, &consumer_profile, Some(evaluation_time)),
        )
    }

    pub fn hydrate_for_profile(
        &self,
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
    ) -> Result<Configuration, ModifierError> {
        let applicability = ModifierApplicability::all();
        self.hydrate_with_inputs(
            selections,
            HydrateInputs::new(&applicability, consumer_profile, None),
        )
    }

    pub fn hydrate_for_profile_at(
        &self,
        selections: &Selections,
        consumer_profile: &ConsumerProfile,
        evaluation_time: &EvaluationTime,
    ) -> Result<Configuration, ModifierError> {
        let applicability = ModifierApplicability::all();
        self.hydrate_with_inputs(
            selections,
            HydrateInputs::new(&applicability, consumer_profile, Some(evaluation_time)),
        )
    }

    pub fn hydrate_with_applicability(
        &self,
        selections: &Selections,
        applicability: &ModifierApplicability,
    ) -> Result<Configuration, ModifierError> {
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_with_inputs(
            selections,
            HydrateInputs::new(applicability, &consumer_profile, None),
        )
    }

    pub fn hydrate_with_applicability_at(
        &self,
        selections: &Selections,
        applicability: &ModifierApplicability,
        evaluation_time: &EvaluationTime,
    ) -> Result<Configuration, ModifierError> {
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_with_inputs(
            selections,
            HydrateInputs::new(applicability, &consumer_profile, Some(evaluation_time)),
        )
    }

    pub fn hydrate_with_applicability_and_profile(
        &self,
        selections: &Selections,
        applicability: &ModifierApplicability,
        consumer_profile: &ConsumerProfile,
    ) -> Result<Configuration, ModifierError> {
        self.hydrate_with_inputs(
            selections,
            HydrateInputs::new(applicability, consumer_profile, None),
        )
    }

    pub fn hydrate_with_applicability_and_profile_at(
        &self,
        selections: &Selections,
        applicability: &ModifierApplicability,
        consumer_profile: &ConsumerProfile,
        evaluation_time: &EvaluationTime,
    ) -> Result<Configuration, ModifierError> {
        self.hydrate_with_inputs(
            selections,
            HydrateInputs::new(applicability, consumer_profile, Some(evaluation_time)),
        )
    }

    fn hydrate_with_inputs(
        &self,
        selections: &Selections,
        inputs: HydrateInputs<'_>,
    ) -> Result<Configuration, ModifierError> {
        self.validate_prompt_selections(selections, inputs.applicability)?;

        let mut prompt_occurrences = BTreeMap::<ComponentId, usize>::new();
        let mut prompts = Vec::with_capacity(self.prompts.len());

        for prompt in &self.prompts {
            if !inputs
                .applicability
                .is_prompt_applicable(prompt.prompt_id())
            {
                continue;
            }

            let occurrence = prompt_occurrences
                .entry(prompt.prompt_id().clone())
                .or_insert(0);
            prompts.push(prompt.hydrate_with_inputs(
                selections.prompt_at(prompt.prompt_id(), *occurrence),
                inputs,
            )?);
            *occurrence += 1;
        }

        Ok(Configuration { prompts })
    }

    fn validate_prompt_selections(
        &self,
        selections: &Selections,
        applicability: &ModifierApplicability,
    ) -> Result<(), ModifierError> {
        let mut known_prompts = BTreeSet::<&ComponentId>::new();
        let mut definition_counts = BTreeMap::<&ComponentId, usize>::new();
        let mut selection_counts = BTreeMap::<&ComponentId, usize>::new();

        for prompt in &self.prompts {
            known_prompts.insert(prompt.prompt_id());

            if applicability.is_prompt_applicable(prompt.prompt_id()) {
                *definition_counts.entry(prompt.prompt_id()).or_insert(0) += 1;
            }
        }

        for prompt in selections.prompts() {
            let selected_count = selection_counts.entry(prompt.prompt_id()).or_insert(0);
            *selected_count += 1;

            let allowed_count = definition_counts
                .get(prompt.prompt_id())
                .copied()
                .unwrap_or(0);

            if *selected_count > allowed_count {
                if known_prompts.contains(prompt.prompt_id()) && allowed_count == 0 {
                    return Err(ModifierError::InapplicablePromptSelection(
                        prompt.prompt_id().clone(),
                    ));
                }

                return Err(ModifierError::UnknownPromptSelection(
                    prompt.prompt_id().clone(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
struct HydrateInputs<'a> {
    applicability: &'a ModifierApplicability,
    consumer_profile: &'a ConsumerProfile,
    evaluation_time: Option<&'a EvaluationTime>,
}

impl<'a> HydrateInputs<'a> {
    fn new(
        applicability: &'a ModifierApplicability,
        consumer_profile: &'a ConsumerProfile,
        evaluation_time: Option<&'a EvaluationTime>,
    ) -> Self {
        Self {
            applicability,
            consumer_profile,
            evaluation_time,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Prompt {
    prompt_id: ComponentId,
    label: Label,
    description: Option<Label>,
    rules: Vec<Rule>,
    effects: Vec<Effect>,
    choices: Vec<Choice>,
}

impl Prompt {
    pub fn new(
        prompt_id: ComponentId,
        title: impl Into<String>,
        description: Option<String>,
        rules: Vec<Rule>,
        effects: Vec<Effect>,
        choices: Vec<Choice>,
    ) -> Result<Self, ModifierError> {
        let title = title.into();

        if title.trim().is_empty() {
            return Err(ModifierError::EmptyPromptTitle);
        }

        let label = generated_label(&format!("{}-TITLE", prompt_id.suffix()), title);
        let description = description.and_then(|description| {
            if description.trim().is_empty() {
                None
            } else {
                Some(generated_label(
                    &format!("{}-DESCRIPTION", prompt_id.suffix()),
                    description,
                ))
            }
        });

        Self::new_labeled(prompt_id, label, description, rules, effects, choices)
    }

    pub fn new_labeled(
        prompt_id: ComponentId,
        label: Label,
        description: Option<Label>,
        rules: Vec<Rule>,
        effects: Vec<Effect>,
        choices: Vec<Choice>,
    ) -> Result<Self, ModifierError> {
        if rules.iter().any(|rule| matches!(rule, Rule::Default(_))) {
            return Err(ModifierError::DefaultRuleOnPrompt);
        }

        let bounds = SelectionBounds::from_rules(&rules)?;

        if bounds.min > 0 && choices.is_empty() {
            return Err(ModifierError::RequiredPromptHasNoChoices {
                prompt_id,
                min_select: bounds.min,
            });
        }

        validate_choices(&choices)?;

        Ok(Self {
            prompt_id,
            label,
            description,
            rules,
            effects,
            choices,
        })
    }

    pub fn prompt_id(&self) -> &ComponentId {
        &self.prompt_id
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

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn choices(&self) -> &[Choice] {
        &self.choices
    }

    pub fn walk(&self, visit: &mut impl FnMut(ModifierNode<'_>)) {
        visit(ModifierNode::Prompt(self));

        for choice in &self.choices {
            choice.walk(visit);
        }
    }

    pub fn walk_visible_at(
        &self,
        evaluation_time: &EvaluationTime,
        visit: &mut impl FnMut(ModifierNode<'_>),
    ) {
        visit(ModifierNode::Prompt(self));

        for choice in self.visible_choices_at(evaluation_time) {
            choice.walk_visible_at(evaluation_time, visit);
        }
    }

    pub fn visible_choices_at(&self, evaluation_time: &EvaluationTime) -> Vec<&Choice> {
        self.choices
            .iter()
            .filter(|choice| choice.is_visible_at(evaluation_time))
            .collect()
    }

    pub fn hydrate(
        &self,
        selection: Option<&PromptSelection>,
    ) -> Result<PromptConfiguration, ModifierError> {
        let applicability = ModifierApplicability::all();
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_with_inputs(
            selection,
            HydrateInputs::new(&applicability, &consumer_profile, None),
        )
    }

    pub fn hydrate_at(
        &self,
        selection: Option<&PromptSelection>,
        evaluation_time: &EvaluationTime,
    ) -> Result<PromptConfiguration, ModifierError> {
        let applicability = ModifierApplicability::all();
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_with_inputs(
            selection,
            HydrateInputs::new(&applicability, &consumer_profile, Some(evaluation_time)),
        )
    }

    pub fn hydrate_for_profile(
        &self,
        selection: Option<&PromptSelection>,
        consumer_profile: &ConsumerProfile,
    ) -> Result<PromptConfiguration, ModifierError> {
        let applicability = ModifierApplicability::all();
        self.hydrate_with_inputs(
            selection,
            HydrateInputs::new(&applicability, consumer_profile, None),
        )
    }

    pub fn hydrate_for_profile_at(
        &self,
        selection: Option<&PromptSelection>,
        consumer_profile: &ConsumerProfile,
        evaluation_time: &EvaluationTime,
    ) -> Result<PromptConfiguration, ModifierError> {
        let applicability = ModifierApplicability::all();
        self.hydrate_with_inputs(
            selection,
            HydrateInputs::new(&applicability, consumer_profile, Some(evaluation_time)),
        )
    }

    pub fn hydrate_with_applicability(
        &self,
        selection: Option<&PromptSelection>,
        applicability: &ModifierApplicability,
    ) -> Result<PromptConfiguration, ModifierError> {
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_with_inputs(
            selection,
            HydrateInputs::new(applicability, &consumer_profile, None),
        )
    }

    pub fn hydrate_with_applicability_at(
        &self,
        selection: Option<&PromptSelection>,
        applicability: &ModifierApplicability,
        evaluation_time: &EvaluationTime,
    ) -> Result<PromptConfiguration, ModifierError> {
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_with_inputs(
            selection,
            HydrateInputs::new(applicability, &consumer_profile, Some(evaluation_time)),
        )
    }

    fn hydrate_with_inputs(
        &self,
        selection: Option<&PromptSelection>,
        inputs: HydrateInputs<'_>,
    ) -> Result<PromptConfiguration, ModifierError> {
        let input_choices = selection.map(PromptSelection::choices).unwrap_or(&[]);
        let validated = self.hydrate_selections_with_inputs(input_choices, inputs)?;
        let mut choices = Vec::with_capacity(validated.len());

        for (index, validated_choice) in validated.into_iter().enumerate() {
            let choice = self.choice(validated_choice.choice_id()).ok_or_else(|| {
                ModifierError::UnknownSelection(validated_choice.choice_id().clone())
            })?;
            let input_choice = input_choices.get(index);

            let choice_inputs = validated_choice
                .inputs()
                .iter()
                .map(|value| {
                    let input = choice
                        .inputs()
                        .iter()
                        .find(|input| input.input_id() == value.input_id())
                        .expect("validated choice input values have authored definitions");

                    Ok(ChoiceInputConfiguration {
                        input_id: value.input_id().clone(),
                        label: input.label().resolve(inputs.consumer_profile)?,
                        label_definition: input.label().clone(),
                        unit: value.unit(),
                        value: value.value().to_owned(),
                    })
                })
                .collect::<Result<Vec<_>, ModifierError>>()?;

            let modifiers = match choice.modifiers() {
                Some(modifiers) => {
                    let empty_selections = Selections::new();
                    let nested_selections = input_choice
                        .and_then(ChoiceSelection::modifiers)
                        .unwrap_or(&empty_selections);
                    Some(Box::new(
                        modifiers.hydrate_with_inputs(nested_selections, inputs)?,
                    ))
                }
                None => {
                    if let Some(nested) = input_choice.and_then(ChoiceSelection::modifiers)
                        && !nested.is_empty()
                    {
                        return Err(ModifierError::UnexpectedNestedSelections(
                            choice.choice_id.clone(),
                        ));
                    }

                    None
                }
            };

            choices.push(ChoiceConfiguration {
                choice_id: validated_choice.choice_id,
                label: choice.label.resolve(inputs.consumer_profile)?,
                label_definition: choice.label.clone(),
                quantity: validated_choice.quantity,
                source: validated_choice.source,
                effects: validated_choice.effects,
                price: validated_choice.price,
                inputs: choice_inputs,
                modifiers,
            });
        }

        Ok(PromptConfiguration {
            prompt_id: self.prompt_id.clone(),
            label: self.label.resolve(inputs.consumer_profile)?,
            label_definition: self.label.clone(),
            description: self
                .description
                .as_ref()
                .map(|label| label.resolve(inputs.consumer_profile))
                .transpose()?,
            description_definition: self.description.clone(),
            effects: self.effects.clone(),
            choices,
        })
    }

    pub fn hydrate_selections(
        &self,
        selections: &[ChoiceSelection],
    ) -> Result<Vec<ValidatedChoiceSelection>, ModifierError> {
        self.hydrate_selections_with_applicability(selections, &ModifierApplicability::all())
    }

    pub fn hydrate_selections_at(
        &self,
        selections: &[ChoiceSelection],
        evaluation_time: &EvaluationTime,
    ) -> Result<Vec<ValidatedChoiceSelection>, ModifierError> {
        let applicability = ModifierApplicability::all();
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_selections_with_inputs(
            selections,
            HydrateInputs::new(&applicability, &consumer_profile, Some(evaluation_time)),
        )
    }

    pub fn hydrate_selections_with_applicability(
        &self,
        selections: &[ChoiceSelection],
        applicability: &ModifierApplicability,
    ) -> Result<Vec<ValidatedChoiceSelection>, ModifierError> {
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_selections_with_inputs(
            selections,
            HydrateInputs::new(applicability, &consumer_profile, None),
        )
    }

    pub fn hydrate_selections_with_applicability_at(
        &self,
        selections: &[ChoiceSelection],
        applicability: &ModifierApplicability,
        evaluation_time: &EvaluationTime,
    ) -> Result<Vec<ValidatedChoiceSelection>, ModifierError> {
        let consumer_profile = ConsumerProfile::empty();
        self.hydrate_selections_with_inputs(
            selections,
            HydrateInputs::new(applicability, &consumer_profile, Some(evaluation_time)),
        )
    }

    fn hydrate_selections_with_inputs(
        &self,
        selections: &[ChoiceSelection],
        inputs: HydrateInputs<'_>,
    ) -> Result<Vec<ValidatedChoiceSelection>, ModifierError> {
        if selections.is_empty() {
            let defaults = self.default_candidates(inputs)?;
            return self.validate_candidates(defaults, inputs);
        }

        self.validate_candidates(
            selections
                .iter()
                .map(|selection| SelectionCandidate {
                    choice_id: selection.choice_id().clone(),
                    quantity: selection.quantity(),
                    source: selection.source(),
                    inputs: selection.inputs().to_vec(),
                })
                .collect(),
            inputs,
        )
    }

    pub fn validate_selections(
        &self,
        selections: &[ChoiceSelection],
    ) -> Result<Vec<ValidatedChoiceSelection>, ModifierError> {
        self.validate_candidates(
            selections
                .iter()
                .map(|selection| SelectionCandidate {
                    choice_id: selection.choice_id().clone(),
                    quantity: selection.quantity(),
                    source: selection.source(),
                    inputs: selection.inputs().to_vec(),
                })
                .collect(),
            HydrateInputs::new(
                &ModifierApplicability::all(),
                &ConsumerProfile::empty(),
                None,
            ),
        )
    }

    fn validate_candidates(
        &self,
        candidates: Vec<SelectionCandidate>,
        inputs: HydrateInputs<'_>,
    ) -> Result<Vec<ValidatedChoiceSelection>, ModifierError> {
        let bounds = SelectionBounds::from_rules(&self.rules)?;
        let mut seen = BTreeSet::new();
        let mut selected_count = 0_u32;
        let mut validated = Vec::with_capacity(candidates.len());

        for candidate in candidates {
            if candidate.quantity == 0 {
                return Err(ModifierError::ZeroQuantity(candidate.choice_id));
            }

            if !seen.insert(candidate.choice_id.clone()) {
                return Err(ModifierError::DuplicateSelection(candidate.choice_id));
            }

            let choice = self
                .choice(&candidate.choice_id)
                .ok_or_else(|| ModifierError::UnknownSelection(candidate.choice_id.clone()))?;

            if !inputs
                .applicability
                .is_choice_applicable(choice.choice_id())
            {
                return Err(ModifierError::InapplicableChoiceSelection(
                    choice.choice_id.clone(),
                ));
            }

            if !choice.is_available_at(inputs.evaluation_time)? {
                return Err(ModifierError::UnavailableChoiceSelection(
                    choice.choice_id.clone(),
                ));
            }

            choice.validate_quantity(candidate.quantity)?;
            let choice_inputs = choice.validate_inputs(candidate.quantity, &candidate.inputs)?;

            selected_count = selected_count
                .checked_add(candidate.quantity)
                .ok_or(ModifierError::SelectionCountOverflow)?;

            validated.push(ValidatedChoiceSelection {
                choice_id: choice.choice_id.clone(),
                quantity: candidate.quantity,
                source: candidate.source,
                effects: choice.effects.clone(),
                price: choice.price.clone(),
                inputs: choice_inputs,
                modifiers: choice.modifiers.clone(),
            });
        }

        if selected_count < bounds.min {
            return Err(ModifierError::BelowMinimum {
                min_select: bounds.min,
                actual: selected_count,
            });
        }

        if let Some(max_select) = bounds.max
            && selected_count > max_select
        {
            return Err(ModifierError::AboveMaximum {
                max_select,
                actual: selected_count,
            });
        }

        Ok(validated)
    }

    fn default_candidates(
        &self,
        inputs: HydrateInputs<'_>,
    ) -> Result<Vec<SelectionCandidate>, ModifierError> {
        let mut defaults = Vec::new();

        for choice in &self.choices {
            if !inputs
                .applicability
                .is_choice_applicable(choice.choice_id())
            {
                continue;
            }

            if let Some(quantity) = choice.default_quantity()? {
                if !choice.is_available_at(inputs.evaluation_time)? {
                    continue;
                }

                defaults.push(SelectionCandidate {
                    choice_id: choice.choice_id.clone(),
                    quantity,
                    source: SelectionSource::Default,
                    inputs: Vec::new(),
                });
            }
        }

        Ok(defaults)
    }

    fn choice(&self, choice_id: &ComponentId) -> Option<&Choice> {
        self.choices
            .iter()
            .find(|choice| choice.choice_id() == choice_id)
    }
}

#[doc = include_str!("choice-inputs.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChoiceInput {
    input_id: ComponentId,
    label: Label,
    required: bool,
    min_length: Option<u32>,
    max_length: Option<u32>,
    repeat_per_quantity: bool,
}

impl ChoiceInput {
    pub fn new(
        input_id: ComponentId,
        title: impl Into<String>,
        required: bool,
        min_length: Option<u32>,
        max_length: Option<u32>,
        repeat_per_quantity: bool,
    ) -> Result<Self, ModifierError> {
        let title = title.into();

        if title.trim().is_empty() {
            return Err(ChoiceInputError::EmptyTitle.into());
        }

        let label = generated_label(&format!("{}-TITLE", input_id.suffix()), title);
        Self::new_labeled(
            input_id,
            label,
            required,
            min_length,
            max_length,
            repeat_per_quantity,
        )
    }

    pub fn new_labeled(
        input_id: ComponentId,
        label: Label,
        required: bool,
        min_length: Option<u32>,
        max_length: Option<u32>,
        repeat_per_quantity: bool,
    ) -> Result<Self, ModifierError> {
        if let (Some(min_length), Some(max_length)) = (min_length, max_length)
            && min_length > max_length
        {
            return Err(ChoiceInputError::InvalidLengthConstraints {
                input_id,
                min_length,
                max_length,
            }
            .into());
        }

        Ok(Self {
            input_id,
            label,
            required,
            min_length,
            max_length,
            repeat_per_quantity,
        })
    }

    pub fn input_id(&self) -> &ComponentId {
        &self.input_id
    }

    pub fn title(&self) -> &str {
        self.label.default_text()
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn min_length(&self) -> Option<u32> {
        self.min_length
    }

    pub fn max_length(&self) -> Option<u32> {
        self.max_length
    }

    pub fn repeat_per_quantity(&self) -> bool {
        self.repeat_per_quantity
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Choice {
    choice_id: ComponentId,
    label: Label,
    media: MediaCollection,
    schedule: Option<Schedule>,
    rules: Vec<Rule>,
    effects: Vec<Effect>,
    price: ChoicePrice,
    inputs: Vec<ChoiceInput>,
    modifiers: Option<Box<Modifiers>>,
}

impl Choice {
    pub fn new(
        choice_id: ComponentId,
        title: impl Into<String>,
        rules: Vec<Rule>,
        effects: Vec<Effect>,
    ) -> Result<Self, ModifierError> {
        let title = title.into();

        if title.trim().is_empty() {
            return Err(ModifierError::EmptyChoiceTitle);
        }

        let label = generated_label(&format!("{}-TITLE", choice_id.suffix()), title);
        Self::new_labeled(choice_id, label, rules, effects)
    }

    pub fn new_labeled(
        choice_id: ComponentId,
        label: Label,
        rules: Vec<Rule>,
        effects: Vec<Effect>,
    ) -> Result<Self, ModifierError> {
        let choice = Self {
            choice_id,
            label,
            media: MediaCollection::empty(),
            schedule: None,
            rules,
            effects,
            price: ChoicePrice::none(),
            inputs: Vec::new(),
            modifiers: None,
        };

        choice.validate_rules()?;

        Ok(choice)
    }

    pub fn with_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = Some(Box::new(modifiers));
        self
    }

    pub fn with_price(mut self, price: ChoicePrice) -> Self {
        self.price = price;
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<ChoiceInput>) -> Result<Self, ModifierError> {
        let mut input_ids = BTreeSet::new();

        for input in &inputs {
            if !input_ids.insert(input.input_id.clone()) {
                return Err(ChoiceInputError::DuplicateDefinition(input.input_id.clone()).into());
            }
        }

        self.inputs = inputs;
        Ok(self)
    }

    pub fn with_media(mut self, media: MediaCollection) -> Self {
        self.media = media;
        self
    }

    pub fn with_schedule(mut self, schedule: Schedule) -> Self {
        self.schedule = Some(schedule);
        self
    }

    pub fn choice_id(&self) -> &ComponentId {
        &self.choice_id
    }

    pub fn title(&self) -> &str {
        self.label.default_text()
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn media(&self) -> &MediaCollection {
        &self.media
    }

    pub fn schedule(&self) -> Option<&Schedule> {
        self.schedule.as_ref()
    }

    pub fn is_available_at(
        &self,
        evaluation_time: Option<&EvaluationTime>,
    ) -> Result<bool, ModifierError> {
        let Some(schedule) = self.schedule() else {
            return Ok(true);
        };

        let Some(evaluation_time) = evaluation_time else {
            return Err(ModifierError::ScheduledChoiceRequiresEvaluationTime(
                self.choice_id.clone(),
            ));
        };

        Ok(schedule.is_scheduled_at(evaluation_time))
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn price(&self) -> &ChoicePrice {
        &self.price
    }

    pub fn inputs(&self) -> &[ChoiceInput] {
        &self.inputs
    }

    pub fn modifiers(&self) -> Option<&Modifiers> {
        self.modifiers.as_deref()
    }

    pub fn walk(&self, visit: &mut impl FnMut(ModifierNode<'_>)) {
        visit(ModifierNode::Choice(self));

        if let Some(modifiers) = self.modifiers() {
            modifiers.walk(visit);
        }
    }

    pub fn walk_visible_at(
        &self,
        evaluation_time: &EvaluationTime,
        visit: &mut impl FnMut(ModifierNode<'_>),
    ) {
        visit(ModifierNode::Choice(self));

        if let Some(modifiers) = self.modifiers() {
            modifiers.walk_visible_at(evaluation_time, visit);
        }
    }

    pub fn is_visible_at(&self, evaluation_time: &EvaluationTime) -> bool {
        match self.schedule() {
            Some(schedule) => schedule.is_scheduled_at(evaluation_time),
            None => true,
        }
    }

    fn validate_rules(&self) -> Result<(), ModifierError> {
        SelectionBounds::from_rules(&self.rules)?;

        if let Some(default_quantity) = self.default_quantity()? {
            self.validate_quantity(default_quantity)?;
        }

        Ok(())
    }

    fn validate_quantity(&self, quantity: u32) -> Result<(), ModifierError> {
        let bounds = SelectionBounds::from_rules(&self.rules)?;

        if quantity < bounds.min {
            return Err(ModifierError::ChoiceBelowMinimum {
                choice_id: self.choice_id.clone(),
                min_select: bounds.min,
                actual: quantity,
            });
        }

        if let Some(max_select) = bounds.max
            && quantity > max_select
        {
            return Err(ModifierError::ChoiceAboveMaximum {
                choice_id: self.choice_id.clone(),
                max_select,
                actual: quantity,
            });
        }

        Ok(())
    }

    fn validate_inputs(
        &self,
        quantity: u32,
        values: &[ChoiceInputValue],
    ) -> Result<Vec<ChoiceInputValue>, ModifierError> {
        let mut seen = BTreeSet::new();
        let mut counts = BTreeMap::<&ComponentId, usize>::new();

        for value in values {
            let input = self
                .inputs
                .iter()
                .find(|input| input.input_id() == value.input_id())
                .ok_or_else(|| ChoiceInputError::UnknownInput {
                    choice_id: self.choice_id.clone(),
                    input_id: value.input_id().clone(),
                })
                .map_err(ModifierError::from)?;

            match (input.repeat_per_quantity(), value.unit()) {
                (false, Some(_)) => {
                    return Err(ChoiceInputError::UnexpectedUnit {
                        choice_id: self.choice_id.clone(),
                        input_id: value.input_id().clone(),
                    }
                    .into());
                }
                (true, None) => {
                    return Err(ChoiceInputError::UnitRequired {
                        choice_id: self.choice_id.clone(),
                        input_id: value.input_id().clone(),
                    }
                    .into());
                }
                (true, Some(unit)) if unit == 0 || unit > quantity => {
                    return Err(ChoiceInputError::UnitOutOfRange {
                        choice_id: self.choice_id.clone(),
                        input_id: value.input_id().clone(),
                        unit,
                        quantity,
                    }
                    .into());
                }
                _ => {}
            }

            if !seen.insert((value.input_id().clone(), value.unit())) {
                return Err(ChoiceInputError::DuplicateValue {
                    choice_id: self.choice_id.clone(),
                    input_id: value.input_id().clone(),
                    unit: value.unit(),
                }
                .into());
            }

            let length = value.value().chars().count();

            if let Some(min_length) = input.min_length()
                && (length as u64) < u64::from(min_length)
            {
                return Err(ChoiceInputError::BelowMinimumLength {
                    choice_id: self.choice_id.clone(),
                    input_id: value.input_id().clone(),
                    min_length,
                    actual: length,
                }
                .into());
            }

            if let Some(max_length) = input.max_length()
                && (length as u64) > u64::from(max_length)
            {
                return Err(ChoiceInputError::AboveMaximumLength {
                    choice_id: self.choice_id.clone(),
                    input_id: value.input_id().clone(),
                    max_length,
                    actual: length,
                }
                .into());
            }

            *counts.entry(input.input_id()).or_insert(0) += 1;
        }

        for input in &self.inputs {
            if !input.required() {
                continue;
            }

            let expected = if input.repeat_per_quantity() {
                quantity
            } else {
                1
            };
            let actual = counts.get(input.input_id()).copied().unwrap_or(0);

            if actual as u64 != u64::from(expected) {
                return Err(ChoiceInputError::MissingRequiredValue {
                    choice_id: self.choice_id.clone(),
                    input_id: input.input_id().clone(),
                    expected,
                    actual,
                }
                .into());
            }
        }

        let mut normalized = Vec::with_capacity(values.len());

        for input in &self.inputs {
            let mut input_values: Vec<_> = values
                .iter()
                .filter(|value| value.input_id() == input.input_id())
                .cloned()
                .collect();
            input_values.sort_by_key(ChoiceInputValue::unit);
            normalized.extend(input_values);
        }

        Ok(normalized)
    }

    fn default_quantity(&self) -> Result<Option<u32>, ModifierError> {
        let mut default_quantity = None;

        for rule in &self.rules {
            if let Rule::Default(quantity) = rule {
                if *quantity == 0 {
                    return Err(ModifierError::ZeroDefaultQuantity(self.choice_id.clone()));
                }

                if default_quantity.replace(*quantity).is_some() {
                    return Err(ModifierError::DuplicateDefault(self.choice_id.clone()));
                }
            }
        }

        Ok(default_quantity)
    }
}

fn validate_choices(choices: &[Choice]) -> Result<(), ModifierError> {
    let mut choice_ids = BTreeSet::new();

    for choice in choices {
        if !choice_ids.insert(choice.choice_id.clone()) {
            return Err(ModifierError::DuplicateChoice(choice.choice_id.clone()));
        }
    }

    Ok(())
}

fn generated_label(label_suffix: &str, default: impl Into<String>) -> Label {
    let label_id = LabelId::from_suffix(label_suffix)
        .expect("generated label suffixes are based on validated IDs and static slots");
    Label::new(label_id, default).expect("generated labels are created after empty text validation")
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModifierApplicability {
    inapplicable_prompts: BTreeSet<ComponentId>,
    inapplicable_choices: BTreeSet<ComponentId>,
}

impl ModifierApplicability {
    pub fn all() -> Self {
        Self {
            inapplicable_prompts: BTreeSet::new(),
            inapplicable_choices: BTreeSet::new(),
        }
    }

    pub fn without_prompt(mut self, prompt_id: ComponentId) -> Self {
        self.inapplicable_prompts.insert(prompt_id);
        self
    }

    pub fn without_choice(mut self, choice_id: ComponentId) -> Self {
        self.inapplicable_choices.insert(choice_id);
        self
    }

    pub fn is_prompt_applicable(&self, prompt_id: &ComponentId) -> bool {
        !self.inapplicable_prompts.contains(prompt_id)
    }

    pub fn is_choice_applicable(&self, choice_id: &ComponentId) -> bool {
        !self.inapplicable_choices.contains(choice_id)
    }
}

impl Default for ModifierApplicability {
    fn default() -> Self {
        Self::all()
    }
}
