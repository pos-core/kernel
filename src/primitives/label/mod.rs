use std::fmt;

use crate::primitives::consumer::ConsumerProfile;
use crate::primitives::ids::LabelId;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Label {
    label_id: Option<LabelId>,
    default: String,
    values: Vec<LabelValue>,
}

impl Label {
    pub fn new(label_id: LabelId, default: impl Into<String>) -> Result<Self, LabelError> {
        Self::with_optional_id(Some(label_id), default)
    }

    pub fn without_id(default: impl Into<String>) -> Result<Self, LabelError> {
        Self::with_optional_id(None, default)
    }

    fn with_optional_id(
        label_id: Option<LabelId>,
        default: impl Into<String>,
    ) -> Result<Self, LabelError> {
        let default = default.into();

        if default.trim().is_empty() {
            return Err(LabelError::EmptyDefault);
        }

        Ok(Self {
            label_id,
            default,
            values: Vec::new(),
        })
    }

    pub fn with_value(
        mut self,
        profile: ConsumerProfile,
        value: impl Into<String>,
    ) -> Result<Self, LabelError> {
        self.add_value(profile, value)?;
        Ok(self)
    }

    pub fn add_value(
        &mut self,
        profile: ConsumerProfile,
        value: impl Into<String>,
    ) -> Result<(), LabelError> {
        if profile.is_empty() {
            return Err(LabelError::EmptyProfile);
        }

        let value = value.into();

        if value.trim().is_empty() {
            return Err(LabelError::EmptyValue);
        }

        if self
            .values
            .iter()
            .any(|label_value| label_value.required_profile() == &profile)
        {
            return Err(LabelError::DuplicateProfile {
                label_id: self.label_id.clone(),
                profile,
            });
        }

        self.values.push(LabelValue { profile, value });

        Ok(())
    }

    pub fn label_id(&self) -> Option<&LabelId> {
        self.label_id.as_ref()
    }

    pub fn default_text(&self) -> &str {
        &self.default
    }

    pub fn values(&self) -> &[LabelValue] {
        &self.values
    }

    pub fn resolve(&self, consumer_profile: &ConsumerProfile) -> Result<ResolvedLabel, LabelError> {
        let mut best: Option<&LabelValue> = None;
        let mut ambiguous_specificity = None;

        for value in self
            .values
            .iter()
            .filter(|value| consumer_profile.contains_all(value.required_profile()))
        {
            let specificity = value.required_profile().len();

            match best.map(|current| specificity.cmp(&current.required_profile().len())) {
                None | Some(std::cmp::Ordering::Greater) => {
                    best = Some(value);
                    ambiguous_specificity = None;
                }
                Some(std::cmp::Ordering::Equal) => {
                    ambiguous_specificity = Some(specificity);
                }
                Some(std::cmp::Ordering::Less) => {}
            }
        }

        if let Some(specificity) = ambiguous_specificity {
            return Err(LabelError::AmbiguousResolution {
                label_id: self.label_id.clone(),
                consumer_profile: consumer_profile.clone(),
                specificity,
            });
        }

        Ok(match best {
            Some(value) => ResolvedLabel {
                label_id: self.label_id.clone(),
                value: value.value.clone(),
                matched_profile: Some(value.required_profile().clone()),
            },
            None => ResolvedLabel {
                label_id: self.label_id.clone(),
                value: self.default.clone(),
                matched_profile: None,
            },
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LabelValue {
    profile: ConsumerProfile,
    value: String,
}

impl LabelValue {
    pub fn required_profile(&self) -> &ConsumerProfile {
        &self.profile
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResolvedLabel {
    label_id: Option<LabelId>,
    value: String,
    matched_profile: Option<ConsumerProfile>,
}

impl ResolvedLabel {
    pub fn label_id(&self) -> Option<&LabelId> {
        self.label_id.as_ref()
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn matched_profile(&self) -> Option<&ConsumerProfile> {
        self.matched_profile.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LabelError {
    EmptyDefault,
    EmptyValue,
    EmptyProfile,
    DuplicateProfile {
        label_id: Option<LabelId>,
        profile: ConsumerProfile,
    },
    AmbiguousResolution {
        label_id: Option<LabelId>,
        consumer_profile: ConsumerProfile,
        specificity: usize,
    },
}

impl fmt::Display for LabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDefault => f.write_str("label default cannot be empty"),
            Self::EmptyValue => f.write_str("label value cannot be empty"),
            Self::EmptyProfile => f.write_str("label value profile cannot be empty"),
            Self::DuplicateProfile { label_id, .. } => {
                write!(
                    f,
                    "label `{}` has duplicate consumer profile values",
                    display_label_id(label_id.as_ref())
                )
            }
            Self::AmbiguousResolution {
                label_id,
                specificity,
                ..
            } => write!(
                f,
                "label `{}` has multiple matching values at specificity `{specificity}`",
                display_label_id(label_id.as_ref())
            ),
        }
    }
}

impl std::error::Error for LabelError {}

fn display_label_id(label_id: Option<&LabelId>) -> String {
    label_id
        .map(ToString::to_string)
        .unwrap_or_else(|| "<none>".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{Label, LabelError};
    use crate::primitives::consumer::ConsumerProfile;
    use crate::primitives::ids::{ConsumerAttributeId, LabelId};

    #[test]
    fn label_resolves_the_most_specific_matching_consumer_profile_value() {
        let web = attribute_id("WEB");
        let delivery = attribute_id("DELIVERY");
        let spanish = attribute_id("SPANISH");
        let profile =
            ConsumerProfile::new([web.clone(), delivery.clone(), spanish.clone()]).unwrap();
        let label = Label::new(label_id("PEPPERONI"), "Pepperoni")
            .unwrap()
            .with_value(
                ConsumerProfile::new([web.clone()]).unwrap(),
                "Pepperoni topping",
            )
            .unwrap()
            .with_value(
                ConsumerProfile::new([web, delivery, spanish]).unwrap(),
                "Pepperoni para entrega",
            )
            .unwrap();

        let resolved = label.resolve(&profile).unwrap();

        assert_eq!(resolved.value(), "Pepperoni para entrega");
        assert_eq!(resolved.label_id(), Some(&label_id("PEPPERONI")));
    }

    #[test]
    fn label_without_id_resolves_like_any_other_label() {
        let label = Label::without_id("Custom sandwich").unwrap();
        let resolved = label.resolve(&ConsumerProfile::empty()).unwrap();

        assert_eq!(label.label_id(), None);
        assert_eq!(resolved.label_id(), None);
        assert_eq!(resolved.value(), "Custom sandwich");
    }

    #[test]
    fn label_falls_back_to_default_when_no_consumer_profile_value_matches() {
        let label = Label::new(label_id("PEPPERONI"), "Pepperoni")
            .unwrap()
            .with_value(
                ConsumerProfile::new([attribute_id("WEB")]).unwrap(),
                "Pepperoni topping",
            )
            .unwrap();

        let resolved = label
            .resolve(&ConsumerProfile::new([attribute_id("PREP")]).unwrap())
            .unwrap();

        assert_eq!(resolved.value(), "Pepperoni");
        assert!(resolved.matched_profile().is_none());
    }

    #[test]
    fn equally_specific_label_values_are_ambiguous() {
        let web = attribute_id("WEB");
        let delivery = attribute_id("DELIVERY");
        let spanish = attribute_id("SPANISH");
        let label = Label::new(label_id("PEPPERONI"), "Pepperoni")
            .unwrap()
            .with_value(
                ConsumerProfile::new([web.clone(), delivery.clone()]).unwrap(),
                "Delivery",
            )
            .unwrap()
            .with_value(
                ConsumerProfile::new([web.clone(), spanish.clone()]).unwrap(),
                "Spanish",
            )
            .unwrap();

        assert!(matches!(
            label.resolve(&ConsumerProfile::new([web, delivery, spanish]).unwrap()),
            Err(LabelError::AmbiguousResolution { specificity: 2, .. })
        ));
    }

    fn attribute_id(suffix: &str) -> ConsumerAttributeId {
        ConsumerAttributeId::from_suffix(suffix).unwrap()
    }

    fn label_id(suffix: &str) -> LabelId {
        LabelId::from_suffix(suffix).unwrap()
    }
}
