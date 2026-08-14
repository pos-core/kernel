use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "catalog-item",
        title: "Catalog Item",
        description: "Described behavior tests for ordered variant dimensions, deepest matches, explicit base pricing, presentation metadata, defaults, and configured item pricing.",
        definitions: vec![
            DefinitionLink::new("Catalog item", "../src/catalog_item/catalog-item.md"),
            DefinitionLink::new(
                "Variant dimension",
                "../src/catalog_item/variant-dimension.md",
            ),
            DefinitionLink::new("Variant", "../src/catalog_item/variant.md"),
            DefinitionLink::new("Variant match", "../src/catalog_item/variant-match.md"),
            DefinitionLink::new("Media", "../src/primitives/media/media.md"),
            DefinitionLink::new(
                "Configured catalog item",
                "../src/catalog_item/configured-catalog-item.md",
            ),
        ],
        cases: vec![
            CATALOG_ITEM_REJECTS_INVALID_BASIC_AUTHORING.report_case(),
            EMPTY_MATCH_MODELS_AN_ITEM_WITH_NO_DIMENSIONS.report_case(),
            DIMENSION_ORDER_CONTROLS_SELECTION_AND_COMBINED_LABEL_ORDER.report_case(),
            DEEPEST_MATCHES_CAN_STOP_AT_DIFFERENT_DEPTHS.report_case(),
            SPARSE_MATCHES_DEFINE_ONLY_AUTHORED_COMBINATIONS.report_case(),
            DEEPEST_MATCH_USES_ONLY_ITS_OWN_EXPLICIT_PRICE.report_case(),
            FREE_VARIANTS_REQUIRE_EXPLICIT_CATALOG_ITEM_PERMISSION.report_case(),
            MATCHES_REJECT_INVALID_OR_DUPLICATE_VARIANT_SETS.report_case(),
            MATCH_PRICES_REQUIRE_ONE_CURRENCY.report_case(),
            EXPLICIT_DEFAULT_MATCH_IS_OPTIONAL_UNIQUE_AND_CONCRETE.report_case(),
            IMPLICIT_CONFIGURATION_USES_DEFAULT_OR_SOLE_DEEPEST_MATCH.report_case(),
            CATALOG_ITEMS_VARIANTS_AND_MATCHES_HAVE_INDEPENDENT_OPTIONAL_METADATA.report_case(),
            CONFIGURED_MATCH_RETURNS_EFFECTS_AND_PRICES_SHARED_MODIFIERS.report_case(),
            MATCH_CONTROLS_MODIFIER_CHOICE_AND_PROMPT_APPLICABILITY.report_case(),
            ITEM_WITH_NO_MODIFIER_PROMPTS_ACCEPTS_EMPTY_SELECTION.report_case(),
        ],
    }
}

pub const CATALOG_ITEM_REJECTS_INVALID_BASIC_AUTHORING: DescribedBehavior = DescribedBehavior::new(
    "catalog item rejects invalid basic variant authoring",
    "Catalog item, dimension, and variant labels require text; dimensions require values; and every catalog item requires at least one variant match.",
    catalog_item_rejects_invalid_basic_authoring,
);

#[test]
fn catalog_item_rejects_invalid_basic_authoring() {
    let standard = variant("01STANDARD", "Standard");
    let standard_dimension = dimension("01OPTION", "Option", vec![standard.clone()]);
    let standard_match = priced_match(&["01STANDARD"], 500);

    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01ITEM"),
            " ",
            vec![standard_dimension.clone()],
            vec![standard_match],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::EmptyCatalogItemTitle)
    );
    assert_eq!(
        Variant::new(variant_id("01EMPTY"), " "),
        Err(CatalogItemError::EmptyVariantTitle)
    );
    assert_eq!(
        VariantDimension::new(variant_dimension_id("01EMPTY"), " ", vec![standard.clone()]),
        Err(CatalogItemError::EmptyVariantDimensionTitle)
    );
    assert_eq!(
        VariantDimension::new(variant_dimension_id("01EMPTY"), "Empty", Vec::new()),
        Err(CatalogItemError::VariantDimensionHasNoVariants(
            variant_dimension_id("01EMPTY")
        ))
    );
    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01ITEM"),
            "Item",
            vec![standard_dimension],
            Vec::new(),
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::CatalogItemHasNoVariantMatches(
            catalog_item_id("01ITEM")
        ))
    );
}

pub const EMPTY_MATCH_MODELS_AN_ITEM_WITH_NO_DIMENSIONS: DescribedBehavior = DescribedBehavior::new(
    "empty match models an item with no dimensions",
    "A catalog item with no dimensions still has one required concrete selection: an empty deepest match that resolves its price without inventing an unnamed variant.",
    empty_match_models_an_item_with_no_dimensions,
);

#[test]
fn empty_match_models_an_item_with_no_dimensions() {
    let catalog_item = CatalogItem::new(
        catalog_item_id("01CHIPS"),
        "Bag of chips",
        Vec::new(),
        vec![priced_match(&[], 199)],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert!(catalog_item.dimensions().is_empty());
    assert_eq!(catalog_item.default_variant_ids(), Some(&[][..]));

    let configured = catalog_item.configure(&Selections::new()).unwrap();
    assert!(configured.variant_ids().is_empty());
    assert!(configured.variant_labels().is_empty());
    assert_eq!(configured.variant_title(), None);
    assert_eq!(configured.invariant_price().amount_minor(), 199);
    assert_eq!(configured.total_price().amount_minor(), 199);
}

pub const DIMENSION_ORDER_CONTROLS_SELECTION_AND_COMBINED_LABEL_ORDER: DescribedBehavior =
    DescribedBehavior::new(
        "dimension order controls selection and combined label order",
        "Variant IDs inside a match are unordered, but the catalog stores and resolves them in authored dimension order so a combined label is deterministic.",
        dimension_order_controls_selection_and_combined_label_order,
    );

#[test]
fn dimension_order_controls_selection_and_combined_label_order() {
    let catalog_item = CatalogItem::new(
        catalog_item_id("01PIZZA"),
        "Pizza",
        vec![
            dimension("01SIZE", "Size", vec![variant("01SMALL", "Small")]),
            dimension("01CRUST", "Crust", vec![variant("01THIN", "Thin")]),
        ],
        vec![priced_match(&["01THIN", "01SMALL"], 1200)],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert_eq!(
        catalog_item.variant_matches()[0].variant_ids(),
        &[variant_id("01SMALL"), variant_id("01THIN")]
    );

    let configured = catalog_item
        .configure_variants(
            &[variant_id("01THIN"), variant_id("01SMALL")],
            &Selections::new(),
        )
        .unwrap();

    assert_eq!(
        configured.variant_ids(),
        &[variant_id("01SMALL"), variant_id("01THIN")]
    );
    assert_eq!(configured.variant_labels()[0].value(), "Small");
    assert_eq!(configured.variant_labels()[1].value(), "Thin");
    assert_eq!(configured.variant_title(), Some("Small, Thin".to_owned()));
}

pub const DEEPEST_MATCHES_CAN_STOP_AT_DIFFERENT_DEPTHS: DescribedBehavior = DescribedBehavior::new(
    "deepest matches can stop at different depths",
    "Deepest is relative to authored supersets rather than the number of dimensions, so unrelated Crust and Size selections can each be concrete one-value matches.",
    deepest_matches_can_stop_at_different_depths,
);

#[test]
fn deepest_matches_can_stop_at_different_depths() {
    let catalog_item = CatalogItem::new(
        catalog_item_id("01PIZZA"),
        "Pizza",
        vec![
            dimension(
                "01CRUST",
                "Crust",
                vec![variant("01THICK", "Thick"), variant("01THIN", "Thin")],
            ),
            dimension("01SIZE", "Size", vec![variant("01SMALL", "Small")]),
        ],
        vec![
            priced_match(&["01THICK"], 1400),
            priced_match(&["01THIN"], 1200),
            priced_match(&["01SMALL"], 1000),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert!(
        catalog_item
            .configure_variants(&[variant_id("01THICK")], &Selections::new())
            .is_ok()
    );
    assert!(
        catalog_item
            .configure_variants(&[variant_id("01SMALL")], &Selections::new())
            .is_ok()
    );
    assert_eq!(
        catalog_item.configure_variants(
            &[variant_id("01THIN"), variant_id("01SMALL")],
            &Selections::new(),
        ),
        Err(CatalogItemError::VariantCombinationDoesNotExist(vec![
            variant_id("01THIN"),
            variant_id("01SMALL"),
        ]))
    );
}

pub const SPARSE_MATCHES_DEFINE_ONLY_AUTHORED_COMBINATIONS: DescribedBehavior =
    DescribedBehavior::new(
        "sparse matches define only authored combinations",
        "Dimensions do not imply a Cartesian product; pizza size and crust values form concrete selections only where a deepest match is authored.",
        sparse_matches_define_only_authored_combinations,
    );

#[test]
fn sparse_matches_define_only_authored_combinations() {
    let catalog_item = sparse_pizza();

    for pair in [
        ["01PERSONAL", "01VEGAN"],
        ["01SMALL", "01THIN"],
        ["01SMALL", "01THICK"],
        ["01MEDIUM", "01THICK"],
        ["01LARGE", "01THIN"],
        ["01LARGE", "01THICK"],
    ] {
        assert!(
            catalog_item
                .configure_variants(
                    &[variant_id(pair[0]), variant_id(pair[1])],
                    &Selections::new(),
                )
                .is_ok()
        );
    }

    assert_eq!(
        catalog_item.configure_variants(
            &[variant_id("01MEDIUM"), variant_id("01THIN")],
            &Selections::new(),
        ),
        Err(CatalogItemError::VariantCombinationDoesNotExist(vec![
            variant_id("01MEDIUM"),
            variant_id("01THIN"),
        ]))
    );
}

pub const DEEPEST_MATCH_USES_ONLY_ITS_OWN_EXPLICIT_PRICE: DescribedBehavior =
    DescribedBehavior::new(
        "deepest match uses only its own explicit price",
        "Every match owns a required price, and configuration uses the selected deepest match's price without inheriting from a shallower match.",
        deepest_match_uses_only_its_own_explicit_price,
    );

#[test]
fn deepest_match_uses_only_its_own_explicit_price() {
    let catalog_item = CatalogItem::new(
        catalog_item_id("01SHIRT"),
        "Shirt",
        vec![
            dimension("01SIZE", "Size", vec![variant("01SMALL", "Small")]),
            dimension("01COLOR", "Color", vec![variant("01GREEN", "Green")]),
        ],
        vec![
            priced_match(&["01SMALL"], 2000),
            priced_match(&["01SMALL", "01GREEN"], 2300),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    let configured = catalog_item
        .configure_variants(
            &[variant_id("01SMALL"), variant_id("01GREEN")],
            &Selections::new(),
        )
        .unwrap();
    assert_eq!(configured.invariant_price().amount_minor(), 2300);
}

pub const FREE_VARIANTS_REQUIRE_EXPLICIT_CATALOG_ITEM_PERMISSION: DescribedBehavior =
    DescribedBehavior::new(
        "free variants require explicit catalog item permission",
        "The allow_free_variant setting defaults to false, so a zero-priced deepest match is rejected unless the catalog item explicitly enables free variants.",
        free_variants_require_explicit_catalog_item_permission,
    );

#[test]
fn free_variants_require_explicit_catalog_item_permission() {
    let free_match = priced_match(&[], 0);
    assert!(!VariantSettings::default().allow_free_variant());
    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01SAMPLE"),
            "Sample",
            Vec::new(),
            vec![free_match.clone()],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::FreeVariantNotAllowed(Vec::new()))
    );

    let settings = VariantSettings::new().with_allow_free_variant(true);
    let catalog_item = CatalogItem::with_variant_settings(
        catalog_item_id("01SAMPLE"),
        "Sample",
        Vec::new(),
        vec![free_match],
        settings,
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert!(catalog_item.variant_settings().allow_free_variant());
    assert_eq!(
        catalog_item
            .configure(&Selections::new())
            .unwrap()
            .invariant_price()
            .amount_minor(),
        0
    );
}

pub const MATCHES_REJECT_INVALID_OR_DUPLICATE_VARIANT_SETS: DescribedBehavior =
    DescribedBehavior::new(
        "matches reject invalid or duplicate variant sets",
        "Variant IDs are unique across dimensions; a match cannot use two values from one dimension, reference an unknown value, or duplicate another unordered match.",
        matches_reject_invalid_or_duplicate_variant_sets,
    );

#[test]
fn matches_reject_invalid_or_duplicate_variant_sets() {
    let dimensions = vec![
        dimension(
            "01SIZE",
            "Size",
            vec![variant("01SMALL", "Small"), variant("01LARGE", "Large")],
        ),
        dimension("01CRUST", "Crust", vec![variant("01THIN", "Thin")]),
    ];

    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01PIZZA"),
            "Pizza",
            dimensions.clone(),
            vec![priced_match(&["01SMALL", "01LARGE"], 1200)],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::MultipleVariantsForDimension {
            variant_dimension_id: variant_dimension_id("01SIZE"),
            left: variant_id("01SMALL"),
            right: variant_id("01LARGE"),
        })
    );
    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01PIZZA"),
            "Pizza",
            dimensions.clone(),
            vec![priced_match(&["01UNKNOWN"], 1200)],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::UnknownVariant(variant_id("01UNKNOWN")))
    );
    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01PIZZA"),
            "Pizza",
            dimensions,
            vec![
                priced_match(&["01SMALL", "01THIN"], 1200),
                priced_match(&["01THIN", "01SMALL"], 1200),
            ],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::DuplicateVariantMatch(vec![
            variant_id("01SMALL"),
            variant_id("01THIN"),
        ]))
    );
}

pub const MATCH_PRICES_REQUIRE_ONE_CURRENCY: DescribedBehavior = DescribedBehavior::new(
    "match prices require one currency",
    "All declared variant-match prices on one catalog item use the same currency.",
    match_prices_require_one_currency,
);

#[test]
fn match_prices_require_one_currency() {
    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01COFFEE"),
            "Coffee",
            vec![dimension(
                "01SIZE",
                "Size",
                vec![variant("01SMALL", "Small"), variant("01LARGE", "Large"),]
            )],
            vec![
                priced_match(&["01SMALL"], 300),
                VariantMatch::new(
                    vec![variant_id("01LARGE")],
                    Money::new(500, CurrencyCode::parse("CAD").unwrap()),
                    Vec::new(),
                )
                .unwrap(),
            ],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::VariantCurrencyMismatch {
            left: vec![variant_id("01SMALL")],
            right: vec![variant_id("01LARGE")],
        })
    );
}

pub const EXPLICIT_DEFAULT_MATCH_IS_OPTIONAL_UNIQUE_AND_CONCRETE: DescribedBehavior =
    DescribedBehavior::new(
        "explicit default match is optional unique and concrete",
        "A multi-selection item accepts at most one default marker, and only a deepest match can carry it so removing the match cannot leave a dangling reference.",
        explicit_default_match_is_optional_unique_and_concrete,
    );

#[test]
fn explicit_default_match_is_optional_unique_and_concrete() {
    let dimensions = vec![dimension(
        "01SIZE",
        "Size",
        vec![variant("01SMALL", "Small"), variant("01LARGE", "Large")],
    )];
    let small = priced_match(&["01SMALL"], 199);
    let large = priced_match(&["01LARGE"], 349).with_default();
    let catalog_item = CatalogItem::new(
        catalog_item_id("01CHIPS"),
        "Bag of chips",
        dimensions.clone(),
        vec![small.clone(), large.clone()],
        Modifiers::new(Vec::new()),
    )
    .unwrap();

    assert_eq!(
        catalog_item.default_variant_ids(),
        Some(&[variant_id("01LARGE")][..])
    );

    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01CHIPS"),
            "Bag of chips",
            dimensions.clone(),
            vec![small.clone().with_default(), large],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::MultipleDefaultVariantMatches {
            left: vec![variant_id("01SMALL")],
            right: vec![variant_id("01LARGE")],
        })
    );

    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01SHIRT"),
            "Shirt",
            vec![
                dimension("01SIZE", "Size", vec![variant("01SMALL", "Small")]),
                dimension("01COLOR", "Color", vec![variant("01GREEN", "Green")]),
            ],
            vec![
                priced_match(&["01SMALL"], 2000).with_default(),
                priced_match(&["01SMALL", "01GREEN"], 2300),
            ],
            Modifiers::new(Vec::new()),
        ),
        Err(CatalogItemError::DefaultRequiresConcreteVariantMatch(vec![
            variant_id("01SMALL")
        ]))
    );
}

pub const IMPLICIT_CONFIGURATION_USES_DEFAULT_OR_SOLE_DEEPEST_MATCH: DescribedBehavior =
    DescribedBehavior::new(
        "implicit configuration uses the default or sole deepest match",
        "Configuration without explicit variant IDs uses the marked default or the sole deepest match; multiple unmarked deepest matches require a selection.",
        implicit_configuration_uses_default_or_sole_deepest_match,
    );

#[test]
fn implicit_configuration_uses_default_or_sole_deepest_match() {
    let dimensions = vec![dimension(
        "01SIZE",
        "Size",
        vec![variant("01SMALL", "Small"), variant("01LARGE", "Large")],
    )];
    let chips_id = catalog_item_id("01CHIPS");
    let with_default = CatalogItem::new(
        chips_id.clone(),
        "Bag of chips",
        dimensions.clone(),
        vec![
            priced_match(&["01SMALL"], 199),
            priced_match(&["01LARGE"], 349).with_default(),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap();
    assert_eq!(
        with_default
            .configure(&Selections::new())
            .unwrap()
            .variant_ids(),
        &[variant_id("01LARGE")]
    );

    let without_default = CatalogItem::new(
        chips_id.clone(),
        "Bag of chips",
        dimensions.clone(),
        vec![
            priced_match(&["01SMALL"], 199),
            priced_match(&["01LARGE"], 349),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap();
    assert_eq!(
        without_default.configure(&Selections::new()),
        Err(CatalogItemError::VariantSelectionRequired(chips_id))
    );

    let sole = CatalogItem::new(
        catalog_item_id("01CANDY"),
        "Candy bar",
        dimensions,
        vec![priced_match(&["01SMALL"], 149)],
        Modifiers::new(Vec::new()),
    )
    .unwrap();
    assert_eq!(
        sole.configure(&Selections::new()).unwrap().variant_ids(),
        &[variant_id("01SMALL")]
    );
}

pub const CATALOG_ITEMS_VARIANTS_AND_MATCHES_HAVE_INDEPENDENT_OPTIONAL_METADATA: DescribedBehavior =
    DescribedBehavior::new(
        "catalog items variants and matches have independent optional metadata",
        "Catalog items and variant values independently own labels, optional descriptions, and media; matches own an optional exact label, description, and media without inheriting between scopes.",
        catalog_items_variants_and_matches_have_independent_optional_metadata,
    );

#[test]
fn catalog_items_variants_and_matches_have_independent_optional_metadata() {
    let item_description = Label::new(
        label_id("01CHIPS-DESCRIPTION"),
        "Kettle-cooked potato chips",
    )
    .unwrap();
    let variant_description =
        Label::new(label_id("01LARGE-DESCRIPTION"), "A shareable bag").unwrap();
    let match_label = Label::new(label_id("01PARTY-BAG-TITLE"), "Party bag").unwrap();
    let match_description =
        Label::new(label_id("01PARTY-BAG-DESCRIPTION"), "Large party-size bag").unwrap();
    let front_id = media_id("01CHIPS-FRONT");
    let nutrition_id = media_id("01CHIPS-NUTRITION");
    let media = MediaCollection::new(vec![
        Media::new(front_id.clone(), mime("image/png")),
        Media::new(nutrition_id.clone(), mime("image/jpeg")),
    ])
    .unwrap();
    let match_media_id = media_id("01CHIPS-PARTY");
    let match_media =
        MediaCollection::new(vec![Media::new(match_media_id.clone(), mime("image/webp"))]).unwrap();
    let large = variant("01LARGE", "Large")
        .with_description(variant_description.clone())
        .with_media(media.clone());
    let catalog_item = CatalogItem::new(
        catalog_item_id("01CHIPS"),
        "Bag of chips",
        vec![dimension("01SIZE", "Size", vec![large])],
        vec![
            priced_match(&["01LARGE"], 349)
                .with_label(match_label.clone())
                .with_description(match_description.clone())
                .with_media(match_media),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap()
    .with_description(item_description.clone())
    .with_media(media);

    assert_eq!(catalog_item.description_label(), Some(&item_description));
    assert_eq!(catalog_item.media().len(), 2);
    let large = catalog_item.variant(&variant_id("01LARGE")).unwrap();
    assert_eq!(large.description_label(), Some(&variant_description));
    assert_eq!(large.media().media()[0].media_id(), &front_id);
    assert_eq!(large.media().media()[1].media_id(), &nutrition_id);

    let exact_match = &catalog_item.variant_matches()[0];
    assert_eq!(exact_match.label(), Some(&match_label));
    assert_eq!(exact_match.description_label(), Some(&match_description));
    assert_eq!(exact_match.media().media()[0].media_id(), &match_media_id);

    let configured = catalog_item.configure(&Selections::new()).unwrap();
    assert_eq!(configured.variant_title(), Some("Party bag".to_owned()));
    assert_eq!(
        configured.variant_match_label_definition(),
        Some(&match_label)
    );
    assert_eq!(configured.variant_labels()[0].value(), "Large");
}

pub const CONFIGURED_MATCH_RETURNS_EFFECTS_AND_PRICES_SHARED_MODIFIERS: DescribedBehavior =
    DescribedBehavior::new(
        "configured match returns effects and prices shared modifiers",
        "Configuring a deepest match returns its effects, resolved invariant price, hydrated shared modifiers, modifier contributions, and total price.",
        configured_match_returns_effects_and_prices_shared_modifiers,
    );

#[test]
fn configured_match_returns_effects_and_prices_shared_modifiers() {
    let catalog_item = catalog_item_pizza();
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
        .configure_variants(&[variant_id("01LARGE"), variant_id("01THIN")], &selections)
        .unwrap();

    assert_eq!(configured.catalog_item_id(), catalog_item.catalog_item_id());
    assert_eq!(configured.variant_title(), Some("Large, Thin".to_owned()));
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

pub const MATCH_CONTROLS_MODIFIER_CHOICE_AND_PROMPT_APPLICABILITY: DescribedBehavior =
    DescribedBehavior::new(
        "deepest match controls modifier applicability",
        "Different deepest matches can restrict shared modifier choices or prompts without duplicating the modifier tree.",
        match_controls_modifier_choice_and_prompt_applicability,
    );

#[test]
fn match_controls_modifier_choice_and_prompt_applicability() {
    let catalog_item = catalog_item_pizza();
    let toppings_id = component_id("01TOPPING");
    let bacon_id = component_id("01BACON");
    let bacon_selection = Selections::new().with_prompt(
        toppings_id.clone(),
        vec![ChoiceSelection::new(bacon_id.clone(), 1).with_modifiers(
            Selections::new().with_prompt(
                component_id("01PLACE"),
                vec![ChoiceSelection::new(component_id("01WHOLE"), 1)],
            ),
        )],
    );

    assert_eq!(
        catalog_item.configure_variants(
            &[variant_id("01SMALL"), variant_id("01THIN")],
            &bacon_selection,
        ),
        Err(CatalogItemError::Modifier(
            ModifierError::InapplicableChoiceSelection(bacon_id)
        ))
    );
    assert!(
        catalog_item
            .configure_variants(
                &[variant_id("01LARGE"), variant_id("01THIN")],
                &bacon_selection,
            )
            .is_ok()
    );

    let topping_selection = Selections::new().with_prompt(
        toppings_id.clone(),
        vec![ChoiceSelection::new(component_id("01PEPPER"), 1)],
    );
    assert_eq!(
        catalog_item.configure_variants(&[variant_id("01CHEESE-ONLY")], &topping_selection,),
        Err(CatalogItemError::Modifier(
            ModifierError::InapplicablePromptSelection(toppings_id)
        ))
    );
}

pub const ITEM_WITH_NO_MODIFIER_PROMPTS_ACCEPTS_EMPTY_SELECTION: DescribedBehavior =
    DescribedBehavior::new(
        "item with no modifier prompts accepts only empty selection",
        "An item with no shared modifiers hydrates empty selections and rejects unexpected prompt selections.",
        item_with_no_modifier_prompts_accepts_empty_selection,
    );

#[test]
fn item_with_no_modifier_prompts_accepts_empty_selection() {
    let catalog_item = CatalogItem::new(
        catalog_item_id("01LATTE"),
        "Latte",
        Vec::new(),
        vec![priced_match(&[], 300)],
        Modifiers::new(Vec::new()),
    )
    .unwrap();
    let configured = catalog_item.configure(&Selections::new()).unwrap();

    assert!(configured.modifiers().prompts().is_empty());
    assert_eq!(configured.total_price().amount_minor(), 300);
    assert_eq!(
        catalog_item.configure(&Selections::new().with_prompt(
            component_id("01MILK"),
            vec![ChoiceSelection::new(component_id("01OAT"), 1)]
        ),),
        Err(CatalogItemError::Modifier(
            ModifierError::UnknownPromptSelection(component_id("01MILK"))
        ))
    );
}

fn sparse_pizza() -> CatalogItem {
    CatalogItem::new(
        catalog_item_id("01SPARSE-PIZZA"),
        "Pizza",
        vec![
            dimension(
                "01SIZE",
                "Size",
                vec![
                    variant("01PERSONAL", "Personal"),
                    variant("01SMALL", "Small"),
                    variant("01MEDIUM", "Medium"),
                    variant("01LARGE", "Large"),
                ],
            ),
            dimension(
                "01CRUST",
                "Crust",
                vec![
                    variant("01THIN", "Thin"),
                    variant("01THICK", "Thick"),
                    variant("01VEGAN", "Vegan"),
                ],
            ),
        ],
        vec![
            priced_match(&["01PERSONAL", "01VEGAN"], 1000),
            priced_match(&["01SMALL", "01THIN"], 1200),
            priced_match(&["01SMALL", "01THICK"], 1300),
            priced_match(&["01MEDIUM", "01THICK"], 1500),
            priced_match(&["01LARGE", "01THIN"], 1800),
            priced_match(&["01LARGE", "01THICK"], 1900),
        ],
        Modifiers::new(Vec::new()),
    )
    .unwrap()
}

fn catalog_item_pizza() -> CatalogItem {
    CatalogItem::new(
        catalog_item_id("01PIZZA"),
        "Pizza",
        vec![
            dimension(
                "01SIZE",
                "Size",
                vec![variant("01SMALL", "Small"), variant("01LARGE", "Large")],
            ),
            dimension(
                "01STYLE",
                "Style",
                vec![
                    variant("01THIN", "Thin"),
                    variant("01CHEESE-ONLY", "Cheese only"),
                ],
            ),
        ],
        vec![
            VariantMatch::new(
                vec![variant_id("01SMALL"), variant_id("01THIN")],
                usd(1200),
                vec![catalog_item_variant_effect("small-thin")],
            )
            .unwrap()
            .with_modifier_applicability(
                ModifierApplicability::all().without_choice(component_id("01BACON")),
            ),
            VariantMatch::new(
                vec![variant_id("01LARGE"), variant_id("01THIN")],
                usd(1800),
                vec![catalog_item_variant_effect("large-thin")],
            )
            .unwrap(),
            VariantMatch::new(
                vec![variant_id("01CHEESE-ONLY")],
                usd(1000),
                vec![catalog_item_variant_effect("cheese-only")],
            )
            .unwrap()
            .with_modifier_applicability(
                ModifierApplicability::all().without_prompt(component_id("01TOPPING")),
            ),
        ],
        catalog_item_pizza_modifiers(),
    )
    .unwrap()
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

fn dimension(suffix: &str, title: &str, variants: Vec<Variant>) -> VariantDimension {
    VariantDimension::new(variant_dimension_id(suffix), title, variants).unwrap()
}

fn variant(suffix: &str, title: &str) -> Variant {
    Variant::new(variant_id(suffix), title).unwrap()
}

fn priced_match(variant_suffixes: &[&str], amount_minor: i64) -> VariantMatch {
    VariantMatch::new(
        variant_suffixes
            .iter()
            .map(|suffix| variant_id(suffix))
            .collect(),
        usd(amount_minor),
        Vec::new(),
    )
    .unwrap()
}

fn variant_dimension_id(suffix: &str) -> VariantDimensionId {
    VariantDimensionId::from_suffix(suffix).unwrap()
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
