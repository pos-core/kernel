use pos_core_kernel::prelude::*;

#[test]
fn catalog_item_rejects_empty_titles_and_missing_variants() {
    assert_eq!(
        CatalogItem::new(
            catalog_item_id("01PIZZA"),
            " ",
            vec![large_pizza_variant()],
            pizza_modifiers(),
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
            pizza_modifiers(),
        ),
        Err(CatalogItemError::CatalogItemHasNoVariants(catalog_item_id(
            "01PIZZA"
        )))
    );
}

#[test]
fn catalog_item_rejects_duplicate_variant_ids_and_currency_mismatches() {
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
            pizza_modifiers(),
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
                    money(500, "CAD"),
                    Vec::new()
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

#[test]
fn catalog_item_configures_known_variant_and_prices_shared_modifiers() {
    let catalog_item = pizza_catalog_item();
    let large_id = variant_id("01LARGE-THIN");
    let pepperoni_id = component_id("01PEPPER");
    let toppings_id = component_id("01TOPPING");
    let placement_id = component_id("01PLACE");
    let selections = Selections::new().with_prompt(
        toppings_id.clone(),
        vec![
            ChoiceSelection::new(pepperoni_id.clone(), 1).with_modifiers(
                Selections::new().with_prompt(
                    placement_id.clone(),
                    vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                ),
            ),
        ],
    );

    let configured = catalog_item
        .configure_variant(&large_id, &selections)
        .unwrap();

    assert_eq!(configured.catalog_item_id(), catalog_item.catalog_item_id());
    assert_eq!(configured.catalog_item_label().value(), "Pizza");
    assert_eq!(
        configured.catalog_item_label().label_id(),
        Some(&label_id("01PIZZA-TITLE"))
    );
    assert_eq!(configured.variant_id(), &large_id);
    assert_eq!(configured.variant_label().value(), "Large thin");
    assert_eq!(
        configured.variant_label().label_id(),
        Some(&label_id("01LARGE-THIN-TITLE"))
    );
    assert_eq!(configured.effects().len(), 1);
    assert_eq!(configured.invariant_price().amount_minor(), 1800);
    assert_eq!(configured.modifier_price().total().amount_minor(), 100);
    assert_eq!(configured.total_price().amount_minor(), 1900);
    assert_eq!(
        configured.modifier_price().contributions()[0].choice_id(),
        &pepperoni_id
    );
    assert_eq!(
        configured
            .modifiers()
            .prompt(&toppings_id)
            .unwrap()
            .choices()[0]
            .modifiers()
            .unwrap()
            .prompt(&placement_id)
            .unwrap()
            .choices()[0]
            .choice_id(),
        &component_id("01LEFT")
    );
}

#[test]
fn catalog_item_rejects_unknown_variant() {
    let catalog_item = pizza_catalog_item();
    let unknown_id = variant_id("01SMALL-ICED");

    assert_eq!(
        catalog_item.configure_variant(&unknown_id, &Selections::new()),
        Err(CatalogItemError::UnknownVariant(unknown_id))
    );
}

#[test]
fn selected_variant_controls_modifier_choice_applicability() {
    let catalog_item = pizza_catalog_item();
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

#[test]
fn selected_variant_controls_modifier_prompt_applicability() {
    let catalog_item = pizza_catalog_item();
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

#[test]
fn item_with_no_modifier_prompts_accepts_empty_selection_and_rejects_unknown_prompt_selection() {
    let catalog_item = coffee_catalog_item();
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

#[test]
fn resolved_variant_combinations_are_modeled_by_variant_existence() {
    let catalog_item = coffee_catalog_item();

    assert!(catalog_item.variant(&variant_id("01SMALL-HOT")).is_some());
    assert!(catalog_item.variant(&variant_id("01MEDIUM-ICED")).is_some());
    assert!(catalog_item.variant(&variant_id("01SMALL-ICED")).is_none());
    assert_eq!(
        catalog_item.configure_variant(&variant_id("01SMALL-ICED"), &Selections::new()),
        Err(CatalogItemError::UnknownVariant(variant_id("01SMALL-ICED")))
    );
}

fn pizza_catalog_item() -> CatalogItem {
    CatalogItem::new(
        catalog_item_id("01PIZZA"),
        "Pizza",
        vec![
            small_pizza_variant(),
            large_pizza_variant(),
            cheese_only_pizza_variant(),
        ],
        pizza_modifiers(),
    )
    .unwrap()
}

fn small_pizza_variant() -> Variant {
    Variant::new(
        variant_id("01SMALL-THIN"),
        "Small thin",
        usd(1200),
        vec![variant_effect("small-thin")],
    )
    .unwrap()
    .with_modifier_applicability(
        ModifierApplicability::all().without_choice(component_id("01BACON")),
    )
}

fn large_pizza_variant() -> Variant {
    Variant::new(
        variant_id("01LARGE-THIN"),
        "Large thin",
        usd(1800),
        vec![variant_effect("large-thin")],
    )
    .unwrap()
}

fn cheese_only_pizza_variant() -> Variant {
    Variant::new(
        variant_id("01CHEESE-ONLY"),
        "Cheese only",
        usd(1000),
        vec![variant_effect("cheese-only")],
    )
    .unwrap()
    .with_modifier_applicability(
        ModifierApplicability::all().without_prompt(component_id("01TOPPING")),
    )
}

fn pizza_modifiers() -> Modifiers {
    let placement = placement_modifiers();
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

fn placement_modifiers() -> Modifiers {
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

fn coffee_catalog_item() -> CatalogItem {
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

fn variant_effect(value: &str) -> Effect {
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

fn usd(amount_minor: i64) -> Money {
    money(amount_minor, "USD")
}

fn money(amount_minor: i64, currency: &str) -> Money {
    Money::new(amount_minor, CurrencyCode::parse(currency).unwrap())
}

fn catalog_item_id(suffix: &str) -> CatalogItemId {
    CatalogItemId::from_suffix(suffix).unwrap()
}

fn variant_id(suffix: &str) -> VariantId {
    VariantId::from_suffix(suffix).unwrap()
}

fn component_id(suffix: &str) -> ComponentId {
    ComponentId::from_suffix(suffix).unwrap()
}

fn label_id(suffix: &str) -> LabelId {
    LabelId::from_suffix(suffix).unwrap()
}
