use pos_core_kernel::prelude::*;

pub fn order_item_snapshots_configured_catalog_item() {
    let configured = configured_pizza();
    let order_item = OrderItem::from_configured_catalog_item(
        order_item_id("01ORDERITEM"),
        catalog_version_id("01CATALOGVERSION"),
        2,
        &configured,
    )
    .unwrap();

    assert_eq!(
        order_item.catalog_item_id(),
        Some(&catalog_item_id("01PIZZA"))
    );
    assert_eq!(order_item.catalog_item_label().default_text(), "Pizza");
    assert_eq!(
        order_item.catalog_item_label().label_id(),
        Some(&label_id("01PIZZA-TITLE"))
    );
    assert_eq!(order_item.variant_id(), Some(&variant_id("01LARGE-THIN")));
    assert_eq!(
        order_item.variant_label().unwrap().default_text(),
        "Large thin"
    );
    assert_eq!(order_item.quantity(), 2);
    assert_eq!(order_item.invariant_unit_price().amount_minor(), 1800);
    assert_eq!(order_item.modifier_unit_price().amount_minor(), 100);
    assert_eq!(order_item.unit_price().amount_minor(), 1900);
    assert_eq!(order_item.total_price().amount_minor(), 3800);
    assert_eq!(order_item.effects().len(), 1);

    let toppings = order_item
        .modifiers()
        .prompt(&component_id("01TOPPING"))
        .unwrap();
    let pepperoni = &toppings.choices()[0];
    let contribution = &order_item.modifiers().price().contributions()[0];

    assert_eq!(pepperoni.choice_id(), Some(&component_id("01PEPPER")));
    assert_eq!(pepperoni.title(), "Pepperoni");
    assert_eq!(contribution.choice_id(), Some(&component_id("01PEPPER")));
    assert_eq!(contribution.title(), "Pepperoni");
    assert_eq!(contribution.amount().amount_minor(), 100);
}

pub fn unconnected_order_item_supports_none_ids_down_to_modifiers() {
    let modifier_price = OrderItemModifierPrice::new(
        vec![
            OrderItemPriceContribution::unconnected(
                Label::without_id("Rush prep").unwrap(),
                1,
                usd(50),
            )
            .unwrap(),
        ],
        usd(50),
    )
    .unwrap();
    let modifiers = OrderItemModifierSnapshot::new(
        vec![OrderItemPromptSnapshot::new(
            None,
            Label::without_id("Preparation").unwrap(),
            None,
            Vec::new(),
            vec![
                OrderItemChoiceSnapshot::new(
                    None,
                    Label::without_id("No onions").unwrap(),
                    1,
                    SelectionSource::Explicit,
                    Vec::new(),
                    ChoicePrice::none(),
                    Vec::new(),
                )
                .unwrap(),
            ],
        )],
        modifier_price,
    )
    .unwrap();
    let order_item = OrderItem::manual(
        order_item_id("01ORDERITEM"),
        Label::without_id("Counter special").unwrap(),
        None,
        1,
        usd(1000),
        modifiers,
    )
    .unwrap();

    assert_eq!(order_item.catalog_version_id(), None);
    assert_eq!(order_item.catalog_item_id(), None);
    assert_eq!(order_item.item_label().label_id(), None);
    assert_eq!(order_item.variant_id(), None);
    assert_eq!(order_item.variant_label(), None);
    assert_eq!(order_item.modifier_unit_price().amount_minor(), 50);
    assert_eq!(order_item.unit_price().amount_minor(), 1050);

    let prompt = &order_item.modifiers().prompts()[0];
    let choice = &prompt.choices()[0];
    let contribution = &order_item.modifiers().price().contributions()[0];

    assert_eq!(prompt.prompt_id(), None);
    assert_eq!(prompt.label().label_id(), None);
    assert_eq!(choice.choice_id(), None);
    assert_eq!(choice.label().label_id(), None);
    assert_eq!(contribution.choice_id(), None);
    assert_eq!(contribution.label().label_id(), None);

    let entries = order_item
        .entries(entry_id("01ITEMENTRY"), vec![entry_id("01MODENTRY")])
        .unwrap();

    assert_eq!(entries[0].source(), &EntrySource::Manual);
    assert_eq!(entries[0].description(), "Counter special");
    assert_eq!(entries[0].total_amount().amount_minor(), 1000);
    assert_eq!(entries[1].source(), &EntrySource::Manual);
    assert_eq!(entries[1].description(), "Rush prep");
    assert_eq!(entries[1].total_amount().amount_minor(), 50);
}

pub fn order_item_expands_to_base_and_modifier_entries() {
    let configured = configured_pizza();
    let order_item = OrderItem::from_configured_catalog_item(
        order_item_id("01ORDERITEM"),
        catalog_version_id("01CATALOGVERSION"),
        2,
        &configured,
    )
    .unwrap();
    let entries = order_item
        .entries(entry_id("01PIZZAENTRY"), vec![entry_id("01PEPPERONIENTRY")])
        .unwrap();

    assert_eq!(entries.len(), 2);

    let item = &entries[0];
    assert_eq!(item.kind(), EntryKind::Item);
    assert_eq!(item.description(), "Pizza (Large thin)");
    assert_eq!(item.quantity(), 2);
    assert_eq!(item.unit_amount().amount_minor(), 1800);
    assert_eq!(item.total_amount().amount_minor(), 3600);
    assert_eq!(item.price_category(), Some(PriceCategory::BaseItem));
    assert_eq!(
        item.source(),
        &EntrySource::CatalogItem {
            catalog_version_id: catalog_version_id("01CATALOGVERSION"),
            catalog_item_id: catalog_item_id("01PIZZA"),
            variant_id: variant_id("01LARGE-THIN"),
        }
    );

    let modifier = &entries[1];
    assert_eq!(modifier.kind(), EntryKind::Modifier);
    assert_eq!(modifier.description(), "Pepperoni");
    assert_eq!(modifier.quantity(), 2);
    assert_eq!(modifier.unit_amount().amount_minor(), 100);
    assert_eq!(modifier.total_amount().amount_minor(), 200);
    assert_eq!(modifier.price_category(), Some(PriceCategory::Modifier));
    assert_eq!(
        modifier.source(),
        &EntrySource::Catalog {
            catalog_version_id: catalog_version_id("01CATALOGVERSION"),
            component_id: component_id("01PEPPER"),
        }
    );
}

pub fn order_item_rejects_zero_quantity_and_wrong_modifier_entry_id_count() {
    let configured = configured_pizza();

    assert_eq!(
        OrderItem::from_configured_catalog_item(
            order_item_id("01ORDERITEM"),
            catalog_version_id("01CATALOGVERSION"),
            0,
            &configured,
        ),
        Err(OrderItemError::ZeroQuantity)
    );

    let order_item = OrderItem::from_configured_catalog_item(
        order_item_id("01ORDERITEM"),
        catalog_version_id("01CATALOGVERSION"),
        1,
        &configured,
    )
    .unwrap();

    assert_eq!(
        order_item.entries(entry_id("01PIZZAENTRY"), Vec::new()),
        Err(OrderItemError::ModifierEntryIdCountMismatch {
            expected: 1,
            actual: 0
        })
    );
}

fn configured_pizza() -> ConfiguredCatalogItem {
    let selections = Selections::new().with_prompt(
        component_id("01TOPPING"),
        vec![
            ChoiceSelection::new(component_id("01PEPPER"), 1).with_modifiers(
                Selections::new().with_prompt(
                    component_id("01PLACE"),
                    vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                ),
            ),
        ],
    );

    pizza_catalog_item()
        .configure_variant(&variant_id("01LARGE-THIN"), &selections)
        .unwrap()
}

fn pizza_catalog_item() -> CatalogItem {
    CatalogItem::new(
        catalog_item_id("01PIZZA"),
        "Pizza",
        vec![large_pizza_variant()],
        pizza_modifiers(),
    )
    .unwrap()
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

fn pizza_modifiers() -> Modifiers {
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
                .with_modifiers(placement_modifiers()),
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
    Money::new(amount_minor, CurrencyCode::parse("USD").unwrap())
}

fn order_item_id(suffix: &str) -> OrderItemId {
    OrderItemId::from_suffix(suffix).unwrap()
}

fn catalog_version_id(suffix: &str) -> CatalogVersionId {
    CatalogVersionId::from_suffix(suffix).unwrap()
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

fn entry_id(suffix: &str) -> EntryId {
    EntryId::from_suffix(suffix).unwrap()
}

fn label_id(suffix: &str) -> LabelId {
    LabelId::from_suffix(suffix).unwrap()
}
