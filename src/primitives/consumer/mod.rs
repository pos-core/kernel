use std::collections::BTreeSet;
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

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ConsumerProfile {
    attributes: BTreeSet<ConsumerAttributeId>,
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
        if !self.attributes.insert(attribute_id.clone()) {
            return Err(ConsumerProfileError::DuplicateAttribute(attribute_id));
        }

        Ok(self)
    }

    pub fn attributes(&self) -> &BTreeSet<ConsumerAttributeId> {
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

#[cfg(test)]
mod tests {
    use super::{ConsumerProfile, ConsumerProfileError};
    use crate::primitives::ids::ConsumerAttributeId;

    #[test]
    fn consumer_profile_matches_required_attributes_as_a_set() {
        let web = attribute_id("WEB");
        let delivery = attribute_id("DELIVERY");
        let spanish = attribute_id("SPANISH");

        let active = ConsumerProfile::new([web.clone(), delivery.clone(), spanish]).unwrap();
        let required = ConsumerProfile::new([web, delivery]).unwrap();

        assert!(active.contains_all(&required));
    }

    #[test]
    fn consumer_profile_rejects_duplicate_attributes() {
        let web = attribute_id("WEB");

        assert_eq!(
            ConsumerProfile::new([web.clone(), web.clone()]),
            Err(ConsumerProfileError::DuplicateAttribute(web))
        );
    }

    #[test]
    fn consumer_profile_rejects_duplicate_attributes_added_later() {
        let web = attribute_id("WEB");
        let profile = ConsumerProfile::empty()
            .with_attribute(web.clone())
            .unwrap();

        assert_eq!(
            profile.with_attribute(web.clone()),
            Err(ConsumerProfileError::DuplicateAttribute(web))
        );
    }

    fn attribute_id(suffix: &str) -> ConsumerAttributeId {
        ConsumerAttributeId::from_suffix(suffix).unwrap()
    }
}
