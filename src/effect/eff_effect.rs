use crate::primitives::ids::{
    CheckId, CommandId, ComponentId, EntryId, FulfillmentModeId, SurfaceId,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Effect {
    source: EffectSource,
    target: EffectTarget,
    domain: EffectDomain,
    requirement: EffectRequirement,
    payload: EffectPayload,
}

impl Effect {
    pub fn new(
        source: EffectSource,
        target: EffectTarget,
        domain: EffectDomain,
        payload: EffectPayload,
    ) -> Self {
        Self {
            source,
            target,
            domain,
            requirement: EffectRequirement::Required,
            payload,
        }
    }

    pub fn optional(mut self) -> Self {
        self.requirement = EffectRequirement::Optional;
        self
    }

    pub fn source(&self) -> &EffectSource {
        &self.source
    }

    pub fn target(&self) -> &EffectTarget {
        &self.target
    }

    pub fn domain(&self) -> EffectDomain {
        self.domain
    }

    pub fn requirement(&self) -> EffectRequirement {
        self.requirement
    }

    pub fn payload(&self) -> &EffectPayload {
        &self.payload
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EffectSource {
    CatalogItem(ComponentId),
    Choice(ComponentId),
    ChoiceGroup(ComponentId),
    Surface(SurfaceId),
    FulfillmentMode(FulfillmentModeId),
    Command(CommandId),
    System,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EffectTarget {
    Order,
    ConfiguredCatalogItem,
    Entry(EntryId),
    Selection(ComponentId),
    Check(CheckId),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EffectDomain {
    Price,
    Tax,
    Stock,
    Prep,
    Time,
    Availability,
    Reporting,
    Settlement,
    Validation,
    Attribute,
    Instruction,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EffectRequirement {
    Required,
    Optional,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EffectPayload {
    Standard {
        kind: String,
        value: String,
    },
    External {
        namespace: String,
        kind: String,
        value: String,
    },
}
