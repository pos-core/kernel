use std::cmp::Ordering;
use std::fmt;

use crate::primitives::ids::ConsumerAttributeId;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConsumerAttribute {
    consumer_attribute_id: ConsumerAttributeId,
    code: String,
}

impl ConsumerAttribute {
    pub fn new(
        consumer_attribute_id: ConsumerAttributeId,
        code: impl Into<String>,
    ) -> Result<Self, ConsumerAttributeError> {
        let code = code.into();

        if code.trim().is_empty() {
            return Err(ConsumerAttributeError::EmptyCode);
        }

        Ok(Self {
            consumer_attribute_id,
            code,
        })
    }

    pub fn consumer_attribute_id(&self) -> &ConsumerAttributeId {
        &self.consumer_attribute_id
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConsumerAttributeError {
    EmptyCode,
}

impl fmt::Display for ConsumerAttributeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCode => f.write_str("consumer attribute code cannot be empty"),
        }
    }
}

impl std::error::Error for ConsumerAttributeError {}

#[doc = include_str!("consumer-profile.md")]
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ConsumerProfile {
    attributes: Vec<ConsumerAttributeId>,
}

impl ConsumerProfile {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn new(
        attributes: impl IntoIterator<Item = ConsumerAttributeId>,
    ) -> Result<Self, ConsumerProfileError> {
        let mut profile = Self::empty();

        for attribute_id in attributes {
            profile = profile.with_attribute(attribute_id)?;
        }

        Ok(profile)
    }

    pub fn with_attribute(
        mut self,
        attribute_id: ConsumerAttributeId,
    ) -> Result<Self, ConsumerProfileError> {
        if self.attributes.contains(&attribute_id) {
            return Err(ConsumerProfileError::DuplicateAttribute(attribute_id));
        }

        self.attributes.push(attribute_id);

        Ok(self)
    }

    pub fn attributes(&self) -> &[ConsumerAttributeId] {
        &self.attributes
    }

    pub fn contains(&self, attribute_id: &ConsumerAttributeId) -> bool {
        self.attributes.contains(attribute_id)
    }

    pub fn contains_all(&self, required: &ConsumerProfile) -> bool {
        required
            .attributes
            .iter()
            .all(|attribute_id| self.contains(attribute_id))
    }

    pub(crate) fn has_same_attributes(&self, other: &ConsumerProfile) -> bool {
        self.len() == other.len() && self.contains_all(other)
    }

    pub(crate) fn compare_matching_precedence(
        &self,
        left: &ConsumerProfile,
        right: &ConsumerProfile,
    ) -> Ordering {
        match left.len().cmp(&right.len()) {
            Ordering::Equal => {}
            ordering => return ordering,
        }

        for attribute_id in &self.attributes {
            match (left.contains(attribute_id), right.contains(attribute_id)) {
                (true, false) => return Ordering::Greater,
                (false, true) => return Ordering::Less,
                _ => {}
            }
        }

        Ordering::Equal
    }

    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConsumerProfileError {
    DuplicateAttribute(ConsumerAttributeId),
}

impl fmt::Display for ConsumerProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateAttribute(attribute_id) => {
                write!(f, "duplicate consumer attribute `{attribute_id}`")
            }
        }
    }
}

impl std::error::Error for ConsumerProfileError {}
