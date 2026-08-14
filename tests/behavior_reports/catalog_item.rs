use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "catalog-item",
        title: "Catalog Item",
        description: "Described behavior tests for variants, shared item modifiers, variant applicability, and configured item pricing.",
        definitions: vec![
            DefinitionLink::new("Catalog item", "../src/catalog_item/catalog-item.md"),
            DefinitionLink::new("Variant", "../src/catalog_item/variant.md"),
            DefinitionLink::new(
                "Configured catalog item",
                "../src/catalog_item/configured-catalog-item.md",
            ),
        ],
        cases: vec![
            CATALOG_ITEM_REJECTS_EMPTY_TITLES_AND_MISSING_VARIANTS.report_case(),
            SOLE_VARIANT_MAY_BE_UNLABELED_BUT_MULTIPLE_VARIANTS_REQUIRE_LABELS.report_case(),
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
