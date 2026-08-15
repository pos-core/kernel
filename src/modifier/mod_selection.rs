use crate::primitives::ids::ComponentId;

#[doc = include_str!("selections.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Selections {
    prompts: Vec<PromptSelection>,
}

impl Selections {
    pub fn new() -> Self {
        Self {
            prompts: Vec::new(),
        }
    }

    pub fn with_prompt(mut self, prompt_id: ComponentId, choices: Vec<ChoiceSelection>) -> Self {
        self.set_prompt(prompt_id, choices);
        self
    }

    pub fn with_prompt_instance(mut self, prompt: PromptSelection) -> Self {
        self.push_prompt(prompt);
        self
    }

    pub fn set_prompt(&mut self, prompt_id: ComponentId, choices: Vec<ChoiceSelection>) {
        if let Some(prompt) = self
            .prompts
            .iter_mut()
            .find(|prompt| prompt.prompt_id == prompt_id)
        {
            prompt.choices = choices;
        } else {
            self.prompts.push(PromptSelection { prompt_id, choices });
        }
    }

    pub fn push_prompt(&mut self, prompt: PromptSelection) {
        self.prompts.push(prompt);
    }

    pub fn prompt(&self, prompt_id: &ComponentId) -> Option<&PromptSelection> {
        self.prompts
            .iter()
            .find(|prompt| &prompt.prompt_id == prompt_id)
    }

    pub fn prompt_at(
        &self,
        prompt_id: &ComponentId,
        occurrence: usize,
    ) -> Option<&PromptSelection> {
        self.prompts
            .iter()
            .filter(|prompt| &prompt.prompt_id == prompt_id)
            .nth(occurrence)
    }

    pub fn prompts(&self) -> &[PromptSelection] {
        &self.prompts
    }

    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }
}

impl Default for Selections {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PromptSelection {
    prompt_id: ComponentId,
    choices: Vec<ChoiceSelection>,
}

impl PromptSelection {
    pub fn new(prompt_id: ComponentId, choices: Vec<ChoiceSelection>) -> Self {
        Self { prompt_id, choices }
    }

    pub fn prompt_id(&self) -> &ComponentId {
        &self.prompt_id
    }

    pub fn choices(&self) -> &[ChoiceSelection] {
        &self.choices
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChoiceSelection {
    choice_id: ComponentId,
    quantity: u32,
    source: SelectionSource,
    inputs: Vec<ChoiceInputValue>,
    modifiers: Option<Box<Selections>>,
}

impl ChoiceSelection {
    pub fn new(choice_id: ComponentId, quantity: u32) -> Self {
        Self {
            choice_id,
            quantity,
            source: SelectionSource::Explicit,
            inputs: Vec::new(),
            modifiers: None,
        }
    }

    pub fn with_source(mut self, source: SelectionSource) -> Self {
        self.source = source;
        self
    }

    pub fn with_modifiers(mut self, modifiers: Selections) -> Self {
        self.modifiers = Some(Box::new(modifiers));
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<ChoiceInputValue>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn choice_id(&self) -> &ComponentId {
        &self.choice_id
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn source(&self) -> SelectionSource {
        self.source
    }

    pub fn inputs(&self) -> &[ChoiceInputValue] {
        &self.inputs
    }

    pub fn modifiers(&self) -> Option<&Selections> {
        self.modifiers.as_deref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChoiceInputValue {
    input_id: ComponentId,
    unit: Option<u32>,
    value: String,
}

impl ChoiceInputValue {
    pub fn once(input_id: ComponentId, value: impl Into<String>) -> Self {
        Self {
            input_id,
            unit: None,
            value: value.into(),
        }
    }

    pub fn for_unit(input_id: ComponentId, unit: u32, value: impl Into<String>) -> Self {
        Self {
            input_id,
            unit: Some(unit),
            value: value.into(),
        }
    }

    pub fn input_id(&self) -> &ComponentId {
        &self.input_id
    }

    pub fn unit(&self) -> Option<u32> {
        self.unit
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SelectionSource {
    Explicit,
    Default,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct SelectionCandidate {
    pub(super) choice_id: ComponentId,
    pub(super) quantity: u32,
    pub(super) source: SelectionSource,
    pub(super) inputs: Vec<ChoiceInputValue>,
}
