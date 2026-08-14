use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::ModuleReport;

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "label",
        title: "Label",
        description: "Described behavior tests for consumer profiles and label value resolution.",
        cases: vec![
            case(
                "consumer profile matches required attributes as a set",
                "A consumer profile satisfies a required profile when it contains every required consumer attribute.",
                consumer_profile_matches_required_attributes_as_a_set,
            ),
            case(
                "consumer profile rejects duplicate attributes",
                "ConsumerProfile owns the uniqueness contract and rejects duplicate ConsumerAttribute IDs at construction time.",
                consumer_profile_rejects_duplicate_attributes,
            ),
            case(
                "label resolves most specific consumer profile value",
                "A label chooses the matching value with the largest required consumer profile and preserves the label ID.",
                label_resolves_most_specific_consumer_profile_value,
            ),
            case(
                "label falls back to default",
                "A label uses its default value when no profile-specific value matches the active consumer profile.",
                label_falls_back_to_default,
            ),
            case(
                "label can exist without id",
                "A manual or custom label can have no label ID while preserving the same default text and resolution behavior.",
                label_can_exist_without_id,
            ),
            case(
                "label rejects ambiguous equal specificity matches",
                "A label resolution with two equally specific matching values is invalid rather than order-dependent.",
                label_rejects_ambiguous_equal_specificity_matches,
            ),
        ],
    }
}

fn consumer_profile_matches_required_attributes_as_a_set() {
    let web = consumer_attribute_id("WEB");
    let delivery = consumer_attribute_id("DELIVERY");
    let spanish = consumer_attribute_id("SPANISH");
    let active = ConsumerProfile::new([web.clone(), delivery.clone(), spanish]).unwrap();
    let required = ConsumerProfile::new([web, delivery]).unwrap();

    assert!(active.contains_all(&required));
}

fn consumer_profile_rejects_duplicate_attributes() {
    let web = consumer_attribute_id("WEB");

    assert_eq!(
        ConsumerProfile::new([web.clone(), web.clone()]),
        Err(ConsumerProfileError::DuplicateAttribute(web))
    );
}

fn label_resolves_most_specific_consumer_profile_value() {
    let web = consumer_attribute_id("WEB");
    let delivery = consumer_attribute_id("DELIVERY");
    let spanish = consumer_attribute_id("SPANISH");
    let profile = ConsumerProfile::new([web.clone(), delivery.clone(), spanish.clone()]).unwrap();
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

fn label_falls_back_to_default() {
    let label = Label::new(label_id("PEPPERONI"), "Pepperoni")
        .unwrap()
        .with_value(
            ConsumerProfile::new([consumer_attribute_id("WEB")]).unwrap(),
            "Pepperoni topping",
        )
        .unwrap();

    let resolved = label
        .resolve(&ConsumerProfile::new([consumer_attribute_id("PREP")]).unwrap())
        .unwrap();

    assert_eq!(resolved.value(), "Pepperoni");
    assert!(resolved.matched_profile().is_none());
}

fn label_can_exist_without_id() {
    let label = Label::without_id("Custom sandwich").unwrap();
    let resolved = label.resolve(&ConsumerProfile::empty()).unwrap();

    assert_eq!(label.label_id(), None);
    assert_eq!(resolved.label_id(), None);
    assert_eq!(resolved.value(), "Custom sandwich");
}

fn label_rejects_ambiguous_equal_specificity_matches() {
    let web = consumer_attribute_id("WEB");
    let delivery = consumer_attribute_id("DELIVERY");
    let spanish = consumer_attribute_id("SPANISH");
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
