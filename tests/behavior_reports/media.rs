use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::ModuleReport;

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "media",
        title: "Media",
        description: "Described behavior tests for media collections, MIME metadata, dimensions, and consumer-profile variants.",
        cases: vec![
            case(
                "media collection rejects duplicate defaults",
                "A MediaCollection preserves definition order but rejects duplicate default Media IDs.",
                media_collection_rejects_duplicate_defaults,
            ),
            case(
                "media resolves most specific consumer profile variant",
                "Media always has a default and may resolve to the most specific matching consumer-profile variant.",
                media_resolves_most_specific_consumer_profile_variant,
            ),
            case(
                "media falls back to default",
                "Media resolves to its default representation when no consumer-profile variant matches.",
                media_falls_back_to_default,
            ),
            case(
                "media rejects ambiguous equal specificity variants",
                "Media resolution rejects equally specific matching variants instead of relying on definition order.",
                media_rejects_ambiguous_equal_specificity_variants,
            ),
            case(
                "media validates mime types and dimensions",
                "Media MIME types are normalized and dimensions must be nonzero when provided.",
                media_validates_mime_types_and_dimensions,
            ),
        ],
    }
}

fn media_collection_rejects_duplicate_defaults() {
    let media_id = media_id("BURGER");

    assert_eq!(
        MediaCollection::new(vec![
            Media::new(media_id.clone(), mime("image/webp")),
            Media::new(media_id.clone(), mime("image/jpeg")),
        ]),
        Err(MediaError::DuplicateMedia(media_id))
    );
}

fn media_resolves_most_specific_consumer_profile_variant() {
    let web = consumer_attribute_id("WEB");
    let delivery = consumer_attribute_id("DELIVERY");
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
                ConsumerProfile::new([web, delivery]).unwrap(),
                media_id("BURGER-WEB-DELIVERY"),
                mime("image/png"),
            )
            .unwrap(),
        )
        .unwrap();

    let resolved = media.resolve(&profile).unwrap();

    assert_eq!(resolved.media_id(), &media_id("BURGER-WEB-DELIVERY"));
    assert_eq!(resolved.mime_type().as_str(), "image/png");
    assert_eq!(resolved.label().unwrap().value(), "Delivery burger");
}

fn media_falls_back_to_default() {
    let media = Media::new(media_id("BURGER-DEFAULT"), mime("image/webp"))
        .with_dimensions(MediaDimensions::new(800, 600).unwrap());

    let resolved = media.resolve(&ConsumerProfile::empty()).unwrap();

    assert_eq!(resolved.media_id(), &media_id("BURGER-DEFAULT"));
    assert_eq!(resolved.mime_type().as_str(), "image/webp");
    assert_eq!(resolved.dimensions().unwrap().width_px(), 800);
    assert!(resolved.matched_profile().is_none());
}

fn media_rejects_ambiguous_equal_specificity_variants() {
    let web = consumer_attribute_id("WEB");
    let delivery = consumer_attribute_id("DELIVERY");
    let spanish = consumer_attribute_id("SPANISH");
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

fn media_validates_mime_types_and_dimensions() {
    let mime_type = MediaMimeType::parse("IMAGE/WEBP").unwrap();

    assert_eq!(mime_type.as_str(), "image/webp");
    assert!(mime_type.is_image());
    assert_eq!(
        MediaMimeType::parse("image"),
        Err(MediaError::InvalidMimeType("image".to_owned()))
    );
    assert_eq!(MediaDimensions::new(0, 100), Err(MediaError::ZeroWidth));
    assert_eq!(MediaDimensions::new(100, 0), Err(MediaError::ZeroHeight));
}
