use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "catalog-item",
        title: "Catalog Item",
        description: "Described behavior tests for variants, optional descriptions and media, defaults, shared item modifiers, applicability, and configured item pricing.",
        definitions: vec![
            DefinitionLink::new("Catalog item", "../src/catalog_item/catalog-item.md"),
            DefinitionLink::new("Variant", "../src/catalog_item/variant.md"),
            DefinitionLink::new("Media", "../src/primitives/media/media.md"),
            DefinitionLink::new(
                "Configured catalog item",
                "../src/catalog_item/configured-catalog-item.md",
            ),
        ],
        cases: vec![
            CATALOG_ITEM_REJECTS_EMPTY_TITLES_AND_MISSING_VARIANTS.report_case(),
            SOLE_VARIANT_MAY_BE_UNLABELED_BUT_MULTIPLE_VARIANTS_REQUIRE_LABELS.report_case(),
            CATALOG_ITEMS_AND_VARIANTS_HAVE_OPTIONAL_DESCRIPTIONS.report_case(),
            EXPLICIT_DEFAULT_VARIANT_IS_OPTIONAL_UNIQUE_AND_OWNED_BY_VARIANT.report_case(),
            IMPLICIT_CONFIGURATION_USES_DEFAULT_OR_SOLE_VARIANT.report_case(),
            CATALOG_ITEM_MEDIA_IS_OPTIONAL_AND_PRESERVES_MULTIPLE_DEFINITIONS.report_case(),
            VARIANT_MEDIA_IS_OPTIONAL_AND_PRESERVES_MULTIPLE_DEFINITIONS.report_case(),
            CATALOG_ITEM_REJECTS_DUPLICATE_VARIANT_IDS.report_case(),
            CATALOG_ITEM_CONFIGURES_KNOWN_VARIANT_AND_RETURNS_EFFECTS_AND_MODIFIER_CONFIGURATION.report_case(),
            CATALOG_ITEM_REJECTS_UNKNOWN_VARIANT.report_case(),
            VARIANT_MODIFIERS_CONTROL_CHOICE_SELECTABILITY.report_case(),
            VARIANT_MODIFIERS_CAN_HAVE_DIFFERENT_PROMPT_RULES.report_case(),
            VARIANT_WITH_NO_MODIFIER_PROMPTS_ACCEPTS_EMPTY_SELECTION_AND_REJECTS_UNKNOWN_PROMPT_SELECTION.report_case(),
            RESOLVED_VARIANT_COMBINATIONS_ARE_MODELED_BY_VARIANT_EXISTENCE.report_case(),
        ],
    }
}

pub const CATALOG_ITEM_REJECTS_EMPTY_TITLES_AND_MISSING_VARIANTS: DescribedBehavior =
    DescribedBehavior::new(
        "catalog item rejects empty titles and missing variants",
        "Catalog item construction requires a non-empty title, non-empty text for labeled variants, and at least one resolved variant.",
        catalog_item_rejects_empty_titles_and_missing_variants,
    );

#[test]
fn catalog_item_rejects_empty_titles_and_missing_variants() {
    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01PIZZA"),
            " ",
            vec![catalog_item_large_pizza_variant()],
            catalog_item_pizza_modifiers(),
        ),
        Err(CatalogItemError::EmptyCatalogItemTitle)
    );
    assert_eq!(
        Variant::new(variant_id("01SMALL-HOT"), "", usd(500), Vec::new()),
        Err(CatalogItemError::EmptyVariantTitle)
    );
    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01PIZZA"),
            "Pizza",
            Vec::new(),
            catalog_item_pizza_modifiers(),
        ),
        Err(CatalogItemError::CatalogItemHasNoVariants(catalog_item_id(
            "01PIZZA"
        )))
    );
}

pub const SOLE_VARIANT_MAY_BE_UNLABELED_BUT_MULTIPLE_VARIANTS_REQUIRE_LABELS: DescribedBehavior =
    DescribedBehavior::new(
        "sole variant may be unlabeled but multiple variants require labels",
        "A catalog item may use one unlabeled priced variant for an implicit single configuration; once multiple variants exist, every variant requires a label.",
        sole_variant_may_be_unlabeled_but_multiple_variants_require_labels,
    );

#[test]
fn sole_variant_may_be_unlabeled_but_multiple_variants_require_labels() {
    let unlabeled_id = variant_id("01STANDARD");
    let catalog_item = CatalogItem::new(
        catalog_item_id("01CHIPS"),
        "Bag of chips",
        vec![Variant::without_label(unlabeled_id.clone(), usd(199), Vec::new()).unwrap()],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    let variant = catalog_item.variant(&unlabeled_id).unwrap();
    assert_eq!(variant.label(), None);
    assert_eq!(variant.title(), None);
    assert_eq!(catalog_item.default_variant_id(), Some(&unlabeled_id));

    let configured = catalog_item
        .configure_variant(&unlabeled_id, &Selections::new())
        .unwrap();
    assert_eq!(configured.variant_label(), None);
    assert_eq!(configured.variant_label_definition(), None);
    assert_eq!(configured.invariant_price().amount_minor(), 199);
    assert_eq!(configured.total_price().amount_minor(), 199);

    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01CHIPS"),
            "Bag of chips",
            vec![
                Variant::without_label(unlabeled_id.clone(), usd(199), Vec::new()).unwrap(),
                Variant::new(variant_id("01LARGE"), "Large", usd(349), Vec::new()).unwrap(),
            ],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::UnlabeledVariantRequiresSingleVariant(
            unlabeled_id
        ))
    );
}

pub const CATALOG_ITEMS_AND_VARIANTS_HAVE_OPTIONAL_DESCRIPTIONS: DescribedBehavior =
    DescribedBehavior::new(
        "catalog items and variants have optional descriptions",
        "Catalog items and variants start without descriptions and can independently own optional label-backed descriptions.",
        catalog_items_and_variants_have_optional_descriptions,
    );

#[test]
fn catalog_items_and_variants_have_optional_descriptions() {
    let plain_variant = Variant::new(variant_id("01PLAIN"), "Plain", usd(199), Vec::new()).unwrap();
    let plain_item = CatalogItem::new(
        catalog_item_id("01PLAIN-CHIPS"),
        "Plain chips",
        vec![plain_variant],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert_eq!(plain_item.description(), None);
    assert_eq!(plain_item.variants()[0].description(), None);

    let item_description = Label::new(
        label_id("01CHIPS-DESCRIPTION"),
        "Kettle-cooked potato chips",
    )
    .unwrap();
    let variant_description =
        Label::new(label_id("01LARGE-DESCRIPTION"), "A shareable bag").unwrap();
    let described_variant = Variant::new(variant_id("01LARGE"), "Large", usd(349), Vec::new())
        .unwrap()
        .with_description(variant_description.clone());
    let described_item = CatalogItem::new(
        catalog_item_id("01CHIPS"),
        "Bag of chips",
        vec![described_variant],
        Modifiers::new(Vec::new()),
    )
    .unwrap()
    .with_description(item_description.clone());

    assert_eq!(
        described_item.description(),
        Some("Kettle-cooked potato chips")
    );
    assert_eq!(described_item.description_label(), Some(&item_description));
    assert_eq!(
        described_item.variants()[0].description(),
        Some("A shareable bag")
    );
    assert_eq!(
        described_item.variants()[0].description_label(),
        Some(&variant_description)
    );
}

pub const EXPLICIT_DEFAULT_VARIANT_IS_OPTIONAL_UNIQUE_AND_OWNED_BY_VARIANT: DescribedBehavior =
    DescribedBehavior::new(
        "explicit default variant is optional unique and owned by the variant",
        "A multi-variant catalog item accepts zero or one explicit default marker and rejects multiple markers; after the marked variant is removed, a remaining sole variant becomes the effective default.",
        explicit_default_variant_is_optional_unique_and_owned_by_variant,
    );

#[test]
fn explicit_default_variant_is_optional_unique_and_owned_by_variant() {
    let small_id = variant_id("01SMALL");
    let large_id = variant_id("01LARGE");
    let catalog_item_id = catalog_item_id("01CHIPS");

    let without_default = CatalogItem::new(
        catalog_item_id.clone(),
        "Bag of chips",
        vec![
            Variant::new(small_id.clone(), "Small", usd(199), Vec::new()).unwrap(),
            Variant::new(large_id.clone(), "Large", usd(349), Vec::new()).unwrap(),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert_eq!(without_default.default_variant(), None);
    assert_eq!(without_default.default_variant_id(), None);

    let with_default = CatalogItem::new(
        catalog_item_id.clone(),
        "Bag of chips",
        vec![
            Variant::new(small_id.clone(), "Small", usd(199), Vec::new()).unwrap(),
            Variant::new(large_id.clone(), "Large", usd(349), Vec::new())
                .unwrap()
                .with_default(),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert_eq!(with_default.default_variant_id(), Some(&large_id));
    assert!(with_default.default_variant().unwrap().is_default());

    assert_eq!(
        CatalogItem::new(
            catalog_item_id.clone(),
            "Bag of chips",
            vec![
                Variant::new(small_id.clone(), "Small", usd(199), Vec::new())
                    .unwrap()
                    .with_default(),
                Variant::new(large_id.clone(), "Large", usd(349), Vec::new())
                    .unwrap()
                    .with_default(),
            ],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::MultipleDefaultVariants {
            left: small_id.clone(),
            right: large_id,
        })
    );

    let after_default_variant_is_removed = CatalogItem::new(
        catalog_item_id,
        "Bag of chips",
        vec![Variant::new(small_id.clone(), "Small", usd(199), Vec::new()).unwrap()],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert_eq!(
        after_default_variant_is_removed.default_variant_id(),
        Some(&small_id)
    );
    assert!(
        !after_default_variant_is_removed
            .default_variant()
            .unwrap()
            .is_default()
    );
}

pub const IMPLICIT_CONFIGURATION_USES_DEFAULT_OR_SOLE_VARIANT: DescribedBehavior =
    DescribedBehavior::new(
        "implicit configuration uses the default or sole variant",
        "Configuration without a variant ID uses the effective default: an explicit marker for multiple variants or the sole variant; multiple unmarked variants require an explicit selection.",
        implicit_configuration_uses_default_or_sole_variant,
    );

#[test]
fn implicit_configuration_uses_default_or_sole_variant() {
    let small_id = variant_id("01SMALL");
    let large_id = variant_id("01LARGE");
    let chips_id = catalog_item_id("01CHIPS");
    let with_default = CatalogItem::new(
        chips_id.clone(),
        "Bag of chips",
        vec![
            Variant::new(small_id.clone(), "Small", usd(199), Vec::new()).unwrap(),
            Variant::new(large_id.clone(), "Large", usd(349), Vec::new())
                .unwrap()
                .with_default(),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert_eq!(
        with_default
            .configure(&Selections::new())
            .unwrap()
            .variant_id(),
        &large_id
    );
    assert_eq!(
        with_default
            .configure_variant(&small_id, &Selections::new())
            .unwrap()
            .variant_id(),
        &small_id
    );

    let without_default = CatalogItem::new(
        chips_id.clone(),
        "Bag of chips",
        vec![
            Variant::new(small_id.clone(), "Small", usd(199), Vec::new()).unwrap(),
            Variant::new(large_id, "Large", usd(349), Vec::new()).unwrap(),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert_eq!(
        without_default.configure(&Selections::new()),
        Err(CatalogItemError::VariantSelectionRequired(chips_id))
    );

    let sole_variant_id = variant_id("01STANDARD");
    let sole_variant_item = CatalogItem::new(
        catalog_item_id("01CANDY"),
        "Candy bar",
        vec![Variant::without_label(sole_variant_id.clone(), usd(149), Vec::new()).unwrap()],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert_eq!(
        sole_variant_item
            .configure(&Selections::new())
            .unwrap()
            .variant_id(),
        &sole_variant_id
    );
}

pub const CATALOG_ITEM_MEDIA_IS_OPTIONAL_AND_PRESERVES_MULTIPLE_DEFINITIONS: DescribedBehavior =
    DescribedBehavior::new(
        "catalog item media is optional and preserves multiple definitions",
        "A catalog item starts with an empty media collection and can preserve multiple ordered media definitions independently of its variants.",
        catalog_item_media_is_optional_and_preserves_multiple_definitions,
    );

#[test]
fn catalog_item_media_is_optional_and_preserves_multiple_definitions() {
    let variant = Variant::new(variant_id("01REGULAR"), "Regular", usd(199), Vec::new()).unwrap();
    let plain_item = CatalogItem::new(
        catalog_item_id("01PLAIN-CHIPS"),
        "Plain chips",
        vec![variant.clone()],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert!(plain_item.media().is_empty());
    assert!(plain_item.variants()[0].media().is_empty());

    let front_id = media_id("01CHIPS-FRONT");
    let nutrition_id = media_id("01CHIPS-NUTRITION");
    let media = MediaCollection::new(vec![
        Media::new(front_id.clone(), mime("image/png")),
        Media::new(nutrition_id.clone(), mime("image/jpeg")),
    ])
    .unwrap();
    let catalog_item = CatalogItem::new(
        catalog_item_id("01CHIPS"),
        "Bag of chips",
        vec![variant],
        Modifiers::new(Vec::new()),
    )
    .unwrap()
    .with_media(media);

    assert_eq!(catalog_item.media().len(), 2);
    assert_eq!(catalog_item.media().media()[0].media_id(), &front_id);
    assert_eq!(catalog_item.media().media()[1].media_id(), &nutrition_id);
    assert!(catalog_item.variants()[0].media().is_empty());
}

pub const VARIANT_MEDIA_IS_OPTIONAL_AND_PRESERVES_MULTIPLE_DEFINITIONS: DescribedBehavior =
    DescribedBehavior::new(
        "variant media is optional and preserves multiple definitions",
        "A variant starts with an empty media collection and can preserve multiple ordered media definitions.",
        variant_media_is_optional_and_preserves_multiple_definitions,
    );

#[test]
fn variant_media_is_optional_and_preserves_multiple_definitions() {
    let plain_variant = Variant::new(variant_id("01PLAIN"), "Plain", usd(199), Vec::new()).unwrap();
    assert!(plain_variant.media().is_empty());

    let front_id = media_id("01CHIPS-FRONT");
    let nutrition_id = media_id("01CHIPS-NUTRITION");
    let media = MediaCollection::new(vec![
        Media::new(front_id.clone(), mime("image/png")),
        Media::new(nutrition_id.clone(), mime("image/jpeg")),
    ])
    .unwrap();
    let variant_id = variant_id("01REGULAR");
    let variant = Variant::new(variant_id.clone(), "Regular", usd(199), Vec::new())
        .unwrap()
        .with_media(media);
    let catalog_item = CatalogItem::new(
        catalog_item_id("01CHIPS"),
        "Bag of chips",
        vec![variant],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    let media = catalog_item.variant(&variant_id).unwrap().media();
    assert_eq!(media.len(), 2);
    assert_eq!(media.media()[0].media_id(), &front_id);
    assert_eq!(media.media()[1].media_id(), &nutrition_id);
}

pub const CATALOG_ITEM_REJECTS_DUPLICATE_VARIANT_IDS: DescribedBehavior = DescribedBehavior::new(
    "catalog item rejects duplicate variant IDs and currency mismatches",
    "A catalog item cannot define duplicate variants and all variant invariant prices must share one currency.",
    catalog_item_rejects_duplicate_variant_ids,
);

#[test]
fn catalog_item_rejects_duplicate_variant_ids() {
    let duplicate_id = variant_id("01LARGE-THIN");

    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01PIZZA"),
            "Pizza",
            vec![
                Variant::new(duplicate_id.clone(), "Large thin", usd(1800), Vec::new(),).unwrap(),
                Variant::new(
                    duplicate_id.clone(),
                    "Large thin again",
                    usd(1900),
                    Vec::new(),
                )
                .unwrap(),
            ],
            catalog_item_pizza_modifiers(),
        ),
        Err(CatalogItemError::DuplicateVariant(duplicate_id))
    );

    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01COFFEE"),
            "Coffee",
            vec![
                Variant::new(variant_id("01SMALL"), "Small", usd(300), Vec::new()).unwrap(),
                Variant::new(
                    variant_id("01LARGE"),
                    "Large",
                    Money::new(500, CurrencyCode::parse("CAD").unwrap()),
                    Vec::new(),
                )
                .unwrap(),
            ],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::VariantCurrencyMismatch {
            left: variant_id("01SMALL"),
            right: variant_id("01LARGE")
        })
    );
}

pub const CATALOG_ITEM_CONFIGURES_KNOWN_VARIANT_AND_RETURNS_EFFECTS_AND_MODIFIER_CONFIGURATION:
    DescribedBehavior = DescribedBehavior::new(
    "catalog item configures known variant and prices shared modifiers",
    "Configuring a known variant returns invariant price, hydrated shared modifiers, modifier price contributions, and total price.",
    catalog_item_configures_known_variant_and_returns_effects_and_modifier_configuration,
);

#[test]
fn catalog_item_configures_known_variant_and_returns_effects_and_modifier_configuration() {
    let catalog_item = catalog_item_pizza();
    let large_id = variant_id("01LARGE-THIN");
    let pepperoni_id = component_id("01PEPPER");
    let toppings_id = component_id("01TOPPING");
    let selections = Selections::new().with_prompt(
        toppings_id.clone(),
        vec![
            ChoiceSelection::new(pepperoni_id.clone(), 1).with_modifiers(
                Selections::new().with_prompt(
                    component_id("01PLACE"),
                    vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                ),
            ),
        ],
    );

    let configured = catalog_item
        .configure_variant(&large_id, &selections)
        .unwrap();

    assert_eq!(configured.catalog_item_id(), catalog_item.catalog_item_id());
    assert_eq!(configured.variant_id(), &large_id);
    assert_eq!(configured.effects().len(), 1);
    assert_eq!(configured.invariant_price().amount_minor(), 1800);
    assert_eq!(configured.modifier_price().total().amount_minor(), 100);
    assert_eq!(configured.total_price().amount_minor(), 1900);
    assert_eq!(
        configured
            .modifiers()
            .prompt(&toppings_id)
            .unwrap()
            .choices()[0]
            .choice_id(),
        &pepperoni_id
    );
}

pub const CATALOG_ITEM_REJECTS_UNKNOWN_VARIANT: DescribedBehavior = DescribedBehavior::new(
    "catalog item rejects unknown variant",
    "Invalid variant combinations are modeled by absence; configuring a missing variant fails.",
    catalog_item_rejects_unknown_variant,
);

#[test]
fn catalog_item_rejects_unknown_variant() {
    let catalog_item = catalog_item_pizza();
    let unknown_id = variant_id("01SMALL-ICED");

    assert_eq!(
        catalog_item.configure_variant(&unknown_id, &Selections::new()),
        Err(CatalogItemError::UnknownVariant(unknown_id))
    );
}

pub const VARIANT_MODIFIERS_CONTROL_CHOICE_SELECTABILITY: DescribedBehavior =
    DescribedBehavior::new(
        "selected variant controls modifier choice applicability",
        "A selected variant can make a shared modifier choice inapplicable without duplicating the modifier tree.",
        variant_modifiers_control_choice_selectability,
    );

#[test]
fn variant_modifiers_control_choice_selectability() {
    let catalog_item = catalog_item_pizza();
    let toppings_id = component_id("01TOPPING");
    let placement_id = component_id("01PLACE");
    let bacon_id = component_id("01BACON");
    let bacon_selection = Selections::new().with_prompt(
        toppings_id,
        vec![ChoiceSelection::new(bacon_id.clone(), 1).with_modifiers(
            Selections::new().with_prompt(
                placement_id,
                vec![ChoiceSelection::new(component_id("01WHOLE"), 1)],
            ),
        )],
    );

    assert_eq!(
        catalog_item.configure_variant(&variant_id("01SMALL-THIN"), &bacon_selection),
        Err(CatalogItemError::Modifier(
            ModifierError::InapplicableChoiceSelection(bacon_id.clone())
        ))
    );
    assert!(
        catalog_item
            .configure_variant(&variant_id("01LARGE-THIN"), &bacon_selection)
            .is_ok()
    );
}

pub const VARIANT_MODIFIERS_CAN_HAVE_DIFFERENT_PROMPT_RULES: DescribedBehavior =
    DescribedBehavior::new(
        "selected variant controls modifier prompt applicability",
        "A selected variant can make a shared modifier prompt inapplicable.",
        variant_modifiers_can_have_different_prompt_rules,
    );

#[test]
fn variant_modifiers_can_have_different_prompt_rules() {
    let catalog_item = catalog_item_pizza();
    let toppings_id = component_id("01TOPPING");
    let selections = Selections::new().with_prompt(
        toppings_id.clone(),
        vec![ChoiceSelection::new(component_id("01PEPPER"), 1)],
    );

    assert_eq!(
        catalog_item.configure_variant(&variant_id("01CHEESE-ONLY"), &selections),
        Err(CatalogItemError::Modifier(
            ModifierError::InapplicablePromptSelection(toppings_id)
        ))
    );
}

pub const VARIANT_WITH_NO_MODIFIER_PROMPTS_ACCEPTS_EMPTY_SELECTION_AND_REJECTS_UNKNOWN_PROMPT_SELECTION: DescribedBehavior = DescribedBehavior::new(
    "item with no modifier prompts accepts empty selection and rejects unknown prompt selection",
    "An item with no shared modifiers hydrates empty selections and rejects unexpected prompt selections.",
    variant_with_no_modifier_prompts_accepts_empty_selection_and_rejects_unknown_prompt_selection,
);

#[test]
fn variant_with_no_modifier_prompts_accepts_empty_selection_and_rejects_unknown_prompt_selection() {
    let catalog_item = catalog_item_coffee();
    let hot_id = variant_id("01SMALL-HOT");
    let configured = catalog_item
        .configure_variant(&hot_id, &Selections::new())
        .unwrap();

    assert!(configured.modifiers().prompts().is_empty());
    assert_eq!(configured.total_price().amount_minor(), 300);
    assert_eq!(
        catalog_item.configure_variant(
            &hot_id,
            &Selections::new().with_prompt(
                component_id("01MILK"),
                vec![ChoiceSelection::new(component_id("01OAT"), 1)]
            ),
        ),
        Err(CatalogItemError::Modifier(
            ModifierError::UnknownPromptSelection(component_id("01MILK"))
        ))
    );
}

pub const RESOLVED_VARIANT_COMBINATIONS_ARE_MODELED_BY_VARIANT_EXISTENCE: DescribedBehavior =
    DescribedBehavior::new(
        "resolved variant combinations are modeled by variant existence",
        "The core sees only concrete valid variants; unsupported combinations are unknown variant IDs.",
        resolved_variant_combinations_are_modeled_by_variant_existence,
    );

#[test]
fn resolved_variant_combinations_are_modeled_by_variant_existence() {
    let catalog_item = catalog_item_coffee();

    assert!(catalog_item.variant(&variant_id("01SMALL-HOT")).is_some());
    assert!(catalog_item.variant(&variant_id("01MEDIUM-ICED")).is_some());
    assert!(catalog_item.variant(&variant_id("01SMALL-ICED")).is_none());
    assert_eq!(
        catalog_item.configure_variant(&variant_id("01SMALL-ICED"), &Selections::new()),
        Err(CatalogItemError::UnknownVariant(variant_id("01SMALL-ICED")))
    );
}

fn catalog_item_pizza() -> CatalogItem {
    CatalogItem::new(
        catalog_item_id("01PIZZA"),
        "Pizza",
        vec![
            catalog_item_small_pizza_variant(),
            catalog_item_large_pizza_variant(),
            catalog_item_cheese_only_pizza_variant(),
        ],
        catalog_item_pizza_modifiers(),
    )
    .unwrap()
}

fn catalog_item_small_pizza_variant() -> Variant {
    Variant::new(
        variant_id("01SMALL-THIN"),
        "Small thin",
        usd(1200),
        vec![catalog_item_variant_effect("small-thin")],
    )
    .unwrap()
    .with_modifier_applicability(
        ModifierApplicability::all().without_choice(component_id("01BACON")),
    )
}

fn catalog_item_large_pizza_variant() -> Variant {
    Variant::new(
        variant_id("01LARGE-THIN"),
        "Large thin",
        usd(1800),
        vec![catalog_item_variant_effect("large-thin")],
    )
    .unwrap()
}

fn catalog_item_cheese_only_pizza_variant() -> Variant {
    Variant::new(
        variant_id("01CHEESE-ONLY"),
        "Cheese only",
        usd(1000),
        vec![catalog_item_variant_effect("cheese-only")],
    )
    .unwrap()
    .with_modifier_applicability(
        ModifierApplicability::all().without_prompt(component_id("01TOPPING")),
    )
}

fn catalog_item_pizza_modifiers() -> Modifiers {
    let placement = catalog_item_placement_modifiers();
    Modifiers::new(vec![
        Prompt::new(
            component_id("01TOPPING"),
            "Toppings",
            None,
            vec![Rule::Max(5)],
            Vec::new(),
            vec![
                Choice::new(
                    component_id("01PEPPER"),
                    "Pepperoni",
                    vec![Rule::Max(1)],
                    Vec::new(),
                )
                .unwrap()
                .with_price(ChoicePrice::flat_amount(usd(200)).unwrap())
                .with_modifiers(placement.clone()),
                Choice::new(component_id("01MUSH"), "Mushrooms", Vec::new(), Vec::new())
                    .unwrap()
                    .with_price(ChoicePrice::flat_amount(usd(150)).unwrap())
                    .with_modifiers(placement.clone()),
                Choice::new(component_id("01BACON"), "Bacon", Vec::new(), Vec::new())
                    .unwrap()
                    .with_price(ChoicePrice::flat_amount(usd(300)).unwrap())
                    .with_modifiers(placement),
            ],
        )
        .unwrap(),
    ])
}

fn catalog_item_placement_modifiers() -> Modifiers {
    Modifiers::new(vec![
        Prompt::new(
            component_id("01PLACE"),
            "Placement",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(component_id("01LEFT"), "Left side", Vec::new(), Vec::new())
                    .unwrap()
                    .with_price(ChoicePrice::from_factor(Rate::percent(50))),
                Choice::new(
                    component_id("01RIGHT"),
                    "Right side",
                    Vec::new(),
                    Vec::new(),
                )
                .unwrap()
                .with_price(ChoicePrice::from_factor(Rate::percent(50))),
                Choice::new(
                    component_id("01WHOLE"),
                    "Whole pizza",
                    Vec::new(),
                    Vec::new(),
                )
                .unwrap()
                .with_price(ChoicePrice::from_factor(Rate::percent(100))),
            ],
        )
        .unwrap(),
    ])
}

fn catalog_item_coffee() -> CatalogItem {
    CatalogItem::new(
        catalog_item_id("01LATTE"),
        "Latte",
        vec![
            Variant::new(variant_id("01SMALL-HOT"), "Small hot", usd(300), Vec::new()).unwrap(),
            Variant::new(
                variant_id("01MEDIUM-ICED"),
                "Medium iced",
                usd(500),
                Vec::new(),
            )
            .unwrap(),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap()
}

fn catalog_item_variant_effect(value: &str) -> Effect {
    Effect::new(
        EffectSource::System,
        EffectTarget::ConfiguredCatalogItem,
        EffectDomain::Attribute,
        EffectPayload::Standard {
            kind: "variant".to_owned(),
            value: value.to_owned(),
        },
    )
}
