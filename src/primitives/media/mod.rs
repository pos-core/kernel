use std::collections::BTreeSet;
use std::fmt;

use crate::primitives::consumer::ConsumerProfile;
use crate::primitives::ids::MediaId;
use crate::primitives::label::{Label, LabelError, ResolvedLabel};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct MediaCollection {
    media: Vec<Media>,
}

impl MediaCollection {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn new(media: Vec<Media>) -> Result<Self, MediaError> {
        validate_unique_default_media_ids(&media)?;

        Ok(Self { media })
    }

    pub fn media(&self) -> &[Media] {
        &self.media
    }

    pub fn is_empty(&self) -> bool {
        self.media.is_empty()
    }

    pub fn len(&self) -> usize {
        self.media.len()
    }

    pub fn resolve(
        &self,
        consumer_profile: &ConsumerProfile,
    ) -> Result<Vec<ResolvedMedia>, MediaError> {
        self.media
            .iter()
            .map(|media| media.resolve(consumer_profile))
            .collect()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Media {
    media_id: MediaId,
    mime_type: MediaMimeType,
    label: Option<Label>,
    dimensions: Option<MediaDimensions>,
    variants: Vec<MediaVariant>,
}

impl Media {
    pub fn new(media_id: MediaId, mime_type: MediaMimeType) -> Self {
        Self {
            media_id,
            mime_type,
            label: None,
            dimensions: None,
            variants: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_dimensions(mut self, dimensions: MediaDimensions) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    pub fn with_variant(mut self, variant: MediaVariant) -> Result<Self, MediaError> {
        self.add_variant(variant)?;
        Ok(self)
    }

    pub fn add_variant(&mut self, variant: MediaVariant) -> Result<(), MediaError> {
        if self
            .variants
            .iter()
            .any(|existing| existing.required_profile() == variant.required_profile())
        {
            return Err(MediaError::DuplicateVariantProfile {
                media_id: self.media_id.clone(),
                profile: variant.required_profile().clone(),
            });
        }

        self.variants.push(variant);

        Ok(())
    }

    pub fn media_id(&self) -> &MediaId {
        &self.media_id
    }

    pub fn mime_type(&self) -> &MediaMimeType {
        &self.mime_type
    }

    pub fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }

    pub fn dimensions(&self) -> Option<&MediaDimensions> {
        self.dimensions.as_ref()
    }

    pub fn variants(&self) -> &[MediaVariant] {
        &self.variants
    }

    pub fn resolve(&self, consumer_profile: &ConsumerProfile) -> Result<ResolvedMedia, MediaError> {
        let variant = self.resolve_variant(consumer_profile)?;

        let (media_id, mime_type, label, dimensions, matched_profile) = match variant {
            Some(variant) => (
                variant.media_id.clone(),
                variant.mime_type.clone(),
                variant.label.as_ref().or(self.label.as_ref()),
                variant.dimensions.clone(),
                Some(variant.required_profile.clone()),
            ),
            None => (
                self.media_id.clone(),
                self.mime_type.clone(),
                self.label.as_ref(),
                self.dimensions.clone(),
                None,
            ),
        };

        Ok(ResolvedMedia {
            media_id,
            mime_type,
            label: label
                .map(|label| label.resolve(consumer_profile))
                .transpose()?,
            dimensions,
            matched_profile,
        })
    }

    fn resolve_variant(
        &self,
        consumer_profile: &ConsumerProfile,
    ) -> Result<Option<&MediaVariant>, MediaError> {
        let mut best: Option<&MediaVariant> = None;
        let mut ambiguous_specificity = None;

        for variant in self
            .variants
            .iter()
            .filter(|variant| consumer_profile.contains_all(variant.required_profile()))
        {
            let specificity = variant.required_profile().len();

            match best.map(|current| specificity.cmp(&current.required_profile().len())) {
                None | Some(std::cmp::Ordering::Greater) => {
                    best = Some(variant);
                    ambiguous_specificity = None;
                }
                Some(std::cmp::Ordering::Equal) => {
                    ambiguous_specificity = Some(specificity);
                }
                Some(std::cmp::Ordering::Less) => {}
            }
        }

        if let Some(specificity) = ambiguous_specificity {
            return Err(MediaError::AmbiguousResolution {
                media_id: self.media_id.clone(),
                consumer_profile: consumer_profile.clone(),
                specificity,
            });
        }

        Ok(best)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MediaVariant {
    required_profile: ConsumerProfile,
    media_id: MediaId,
    mime_type: MediaMimeType,
    label: Option<Label>,
    dimensions: Option<MediaDimensions>,
}

impl MediaVariant {
    pub fn new(
        required_profile: ConsumerProfile,
        media_id: MediaId,
        mime_type: MediaMimeType,
    ) -> Result<Self, MediaError> {
        if required_profile.is_empty() {
            return Err(MediaError::EmptyVariantProfile);
        }

        Ok(Self {
            required_profile,
            media_id,
            mime_type,
            label: None,
            dimensions: None,
        })
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_dimensions(mut self, dimensions: MediaDimensions) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    pub fn required_profile(&self) -> &ConsumerProfile {
        &self.required_profile
    }

    pub fn media_id(&self) -> &MediaId {
        &self.media_id
    }

    pub fn mime_type(&self) -> &MediaMimeType {
        &self.mime_type
    }

    pub fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }

    pub fn dimensions(&self) -> Option<&MediaDimensions> {
        self.dimensions.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResolvedMedia {
    media_id: MediaId,
    mime_type: MediaMimeType,
    label: Option<ResolvedLabel>,
    dimensions: Option<MediaDimensions>,
    matched_profile: Option<ConsumerProfile>,
}

impl ResolvedMedia {
    pub fn media_id(&self) -> &MediaId {
        &self.media_id
    }

    pub fn mime_type(&self) -> &MediaMimeType {
        &self.mime_type
    }

    pub fn label(&self) -> Option<&ResolvedLabel> {
        self.label.as_ref()
    }

    pub fn dimensions(&self) -> Option<&MediaDimensions> {
        self.dimensions.as_ref()
    }

    pub fn matched_profile(&self) -> Option<&ConsumerProfile> {
        self.matched_profile.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MediaMimeType {
    value: String,
    slash_index: usize,
}

impl MediaMimeType {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, MediaError> {
        let normalized = value.as_ref().trim().to_ascii_lowercase();

        let Some((top_level_type, subtype)) = normalized.split_once('/') else {
            return Err(MediaError::InvalidMimeType(normalized));
        };

        if top_level_type.is_empty()
            || subtype.is_empty()
            || subtype.contains('/')
            || normalized.bytes().any(|byte| byte.is_ascii_whitespace())
        {
            return Err(MediaError::InvalidMimeType(normalized));
        }

        Ok(Self {
            slash_index: top_level_type.len(),
            value: normalized,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn top_level_type(&self) -> &str {
        &self.value[..self.slash_index]
    }

    pub fn subtype(&self) -> &str {
        &self.value[self.slash_index + 1..]
    }

    pub fn is_image(&self) -> bool {
        self.top_level_type() == "image"
    }

    pub fn is_video(&self) -> bool {
        self.top_level_type() == "video"
    }

    pub fn is_application(&self) -> bool {
        self.top_level_type() == "application"
    }
}

impl fmt::Display for MediaMimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for MediaMimeType {
    type Err = MediaError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for MediaMimeType {
    type Error = MediaError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl AsRef<str> for MediaMimeType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MediaDimensions {
    width_px: u32,
    height_px: u32,
}

impl MediaDimensions {
    pub fn new(width_px: u32, height_px: u32) -> Result<Self, MediaError> {
        if width_px == 0 {
            return Err(MediaError::ZeroWidth);
        }

        if height_px == 0 {
            return Err(MediaError::ZeroHeight);
        }

        Ok(Self {
            width_px,
            height_px,
        })
    }

    pub fn width_px(&self) -> u32 {
        self.width_px
    }

    pub fn height_px(&self) -> u32 {
        self.height_px
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MediaError {
    DuplicateMedia(MediaId),
    DuplicateVariantProfile {
        media_id: MediaId,
        profile: ConsumerProfile,
    },
    AmbiguousResolution {
        media_id: MediaId,
        consumer_profile: ConsumerProfile,
        specificity: usize,
    },
    EmptyVariantProfile,
    InvalidMimeType(String),
    ZeroWidth,
    ZeroHeight,
    Label(LabelError),
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateMedia(media_id) => write!(f, "duplicate media `{media_id}`"),
            Self::DuplicateVariantProfile { media_id, .. } => {
                write!(f, "media `{media_id}` has duplicate variant profiles")
            }
            Self::AmbiguousResolution {
                media_id,
                specificity,
                ..
            } => write!(
                f,
                "media `{media_id}` has multiple matching variants at specificity `{specificity}`"
            ),
            Self::EmptyVariantProfile => f.write_str("media variant profile cannot be empty"),
            Self::InvalidMimeType(value) => {
                write!(f, "invalid media MIME type `{value}`")
            }
            Self::ZeroWidth => f.write_str("media width must be greater than zero"),
            Self::ZeroHeight => f.write_str("media height must be greater than zero"),
            Self::Label(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MediaError {}

impl From<LabelError> for MediaError {
    fn from(error: LabelError) -> Self {
        Self::Label(error)
    }
}

fn validate_unique_default_media_ids(media: &[Media]) -> Result<(), MediaError> {
    let mut media_ids = BTreeSet::new();

    for media in media {
        if !media_ids.insert(media.media_id.clone()) {
            return Err(MediaError::DuplicateMedia(media.media_id.clone()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Media, MediaCollection, MediaDimensions, MediaError, MediaMimeType, MediaVariant};
    use crate::primitives::consumer::ConsumerProfile;
    use crate::primitives::ids::{ConsumerAttributeId, LabelId, MediaId};
    use crate::primitives::label::Label;

    #[test]
    fn media_collection_rejects_duplicate_default_media_ids() {
        let media_id = media_id("BURGER");

        assert_eq!(
            MediaCollection::new(vec![
                Media::new(media_id.clone(), mime("image/webp")),
                Media::new(media_id.clone(), mime("image/jpeg")),
            ]),
            Err(MediaError::DuplicateMedia(media_id))
        );
    }

    #[test]
    fn media_resolves_most_specific_matching_variant_and_label() {
        let web = attribute_id("WEB");
        let delivery = attribute_id("DELIVERY");
        let profile = ConsumerProfile::new([web.clone(), delivery.clone()]).unwrap();
        let label = Label::new(label_id("BURGER"), "Burger")
            .unwrap()
            .with_value(
                ConsumerProfile::new([web.clone(), delivery.clone()]).unwrap(),
                "Delivery burger",
            )
            .unwrap();
        let media = Media::new(media_id("BURGER-DEFAULT"), mime("image/webp"))
            .with_label(label)
            .with_variant(
                MediaVariant::new(
                    ConsumerProfile::new([web.clone()]).unwrap(),
                    media_id("BURGER-WEB"),
                    mime("image/jpeg"),
                )
                .unwrap(),
            )
            .unwrap()
            .with_variant(
                MediaVariant::new(
                    ConsumerProfile::new([delivery.clone()]).unwrap(),
                    media_id("BURGER-DELIVERY"),
                    mime("image/png"),
                )
                .unwrap(),
            )
            .unwrap()
            .with_variant(
                MediaVariant::new(
                    ConsumerProfile::new([web.clone(), delivery]).unwrap(),
                    media_id("BURGER-WEB-DELIVERY"),
                    mime("image/jpeg"),
                )
                .unwrap(),
            )
            .unwrap();

        let resolved = media.resolve(&profile).unwrap();

        assert_eq!(resolved.media_id(), &media_id("BURGER-WEB-DELIVERY"));
        assert_eq!(resolved.mime_type().as_str(), "image/jpeg");
        assert_eq!(resolved.label().unwrap().value(), "Delivery burger");
    }

    #[test]
    fn media_falls_back_to_default_when_no_variant_matches() {
        let media = Media::new(media_id("BURGER-DEFAULT"), mime("image/webp"))
            .with_dimensions(MediaDimensions::new(800, 600).unwrap());

        let resolved = media.resolve(&ConsumerProfile::empty()).unwrap();

        assert_eq!(resolved.media_id(), &media_id("BURGER-DEFAULT"));
        assert_eq!(resolved.mime_type().as_str(), "image/webp");
        assert_eq!(resolved.dimensions().unwrap().width_px(), 800);
        assert!(resolved.matched_profile().is_none());
    }

    #[test]
    fn media_rejects_ambiguous_equal_specificity_variants() {
        let web = attribute_id("WEB");
        let delivery = attribute_id("DELIVERY");
        let spanish = attribute_id("SPANISH");
        let media = Media::new(media_id("BURGER-DEFAULT"), mime("image/webp"))
            .with_variant(
                MediaVariant::new(
                    ConsumerProfile::new([web.clone(), delivery.clone()]).unwrap(),
                    media_id("BURGER-DELIVERY"),
                    mime("image/jpeg"),
                )
                .unwrap(),
            )
            .unwrap()
            .with_variant(
                MediaVariant::new(
                    ConsumerProfile::new([web.clone(), spanish.clone()]).unwrap(),
                    media_id("BURGER-SPANISH"),
                    mime("image/png"),
                )
                .unwrap(),
            )
            .unwrap();

        assert!(matches!(
            media.resolve(&ConsumerProfile::new([web, delivery, spanish]).unwrap()),
            Err(MediaError::AmbiguousResolution { specificity: 2, .. })
        ));
    }

    #[test]
    fn media_validates_mime_types_and_dimensions() {
        let mime_type = MediaMimeType::parse("IMAGE/WEBP").unwrap();

        assert_eq!(mime_type.as_str(), "image/webp");
        assert_eq!(mime_type.top_level_type(), "image");
        assert_eq!(mime_type.subtype(), "webp");
        assert!(mime_type.is_image());
        assert_eq!(
            MediaMimeType::parse("image"),
            Err(MediaError::InvalidMimeType("image".to_owned()))
        );
        assert_eq!(MediaDimensions::new(0, 100), Err(MediaError::ZeroWidth));
        assert_eq!(MediaDimensions::new(100, 0), Err(MediaError::ZeroHeight));
    }

    fn attribute_id(suffix: &str) -> ConsumerAttributeId {
        ConsumerAttributeId::from_suffix(suffix).unwrap()
    }

    fn label_id(suffix: &str) -> LabelId {
        LabelId::from_suffix(suffix).unwrap()
    }

    fn media_id(suffix: &str) -> MediaId {
        MediaId::from_suffix(suffix).unwrap()
    }

    fn mime(value: &str) -> MediaMimeType {
        MediaMimeType::parse(value).unwrap()
    }
}
