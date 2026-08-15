use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "order-item",
        title: "Order Item",
        description: "Described behavior tests for order-owned catalog facts, modifier snapshots, and entry expansion.",
        definitions: vec![
            DefinitionLink::new("Order item", "../src/order_item/order-item.md"),
            DefinitionLink::new(
                "Order-item modifier snapshot",
                "../src/order_item/order-item-modifier-snapshot.md",
            ),
            DefinitionLink::new("Choice inputs", "../src/modifier/choice-inputs.md"),
            DefinitionLink::new("Order entry", "../src/entry/order-entry.md"),
        ],
        cases: vec![
            CATALOG_BACKED_ORDER_ITEM_PRESERVES_CONFIGURED_CATALOG_FACTS.report_case(),
            EMPTY_VARIANT_MATCH_DOES_NOT_DUPLICATE_ITEM_DESCRIPTION.report_case(),
            UNCONNECTED_ORDER_ITEM_SUPPORTS_NONE_IDS_DOWN_TO_MODIFIERS.report_case(),
            ORDER_ITEM_EXPANDS_TO_BASE_AND_MODIFIER_ENTRIES.report_case(),
            ORDER_ITEM_REJECTS_ZERO_QUANTITY_AND_WRONG_MODIFIER_ENTRY_ID_COUNT.report_case(),
        ],
    }
}
pub const CATALOG_BACKED_ORDER_ITEM_PRESERVES_CONFIGURED_CATALOG_FACTS: DescribedBehavior =
    DescribedBehavior::new(
        "catalog-backed order item preserves configured catalog facts",
        "A catalog-backed order item preserves item, exact match, and component variant labels, effects, entered choice inputs, its order-item modifier snapshot, unit prices, and total price.",
        catalog_backed_order_item_preserves_configured_catalog_facts,
    );

#[test]
fn catalog_backed_order_item_preserves_configured_catalog_facts() {
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
    assert_eq!(
        order_item.variant_ids(),
        Some(&[variant_id("01LARGE"), variant_id("01THIN")][..])
    );
    assert_eq!(order_item.variant_labels()[0].default_text(), "Large");
    assert_eq!(order_item.variant_labels()[1].default_text(), "Thin");
    assert_eq!(
        order_item.variant_match_label().unwrap().default_text(),
        "Large thin"
    );
    assert_eq!(
        order_item.variant_match_label().unwrap().label_id(),
        Some(&label_id("01LARGE-THIN-TITLE"))
    );
    assert_eq!(order_item.variant_title(), Some("Large thin".to_owned()));
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
    assert_eq!(
        pepperoni.inputs()[0].input_id(),
        Some(&component_id("01REQUEST"))
    );
    assert_eq!(pepperoni.inputs()[0].title(), "Any special requests?");
    assert_eq!(pepperoni.inputs()[0].unit(), None);
    assert_eq!(pepperoni.inputs()[0].value(), "Cook it well done");
    assert_eq!(contribution.choice_id(), Some(&component_id("01PEPPER")));
    assert_eq!(contribution.title(), "Pepperoni");
    assert_eq!(contribution.amount().amount_minor(), 100);
}

pub const EMPTY_VARIANT_MATCH_DOES_NOT_DUPLICATE_ITEM_DESCRIPTION: DescribedBehavior =
    DescribedBehavior::new(
        "empty variant match does not duplicate the item description",
        "A catalog-backed order item preserves the empty concrete match and its price while rendering only the catalog item label when the item has no dimensions.",
        empty_variant_match_does_not_duplicate_item_description,
    );

#[test]
fn empty_variant_match_does_not_duplicate_item_description() {
    let configured = CatalogItem::new(
        catalog_item_id("01CHIPS"),
        "Bag of chips",
        Vec::new(),
        vec![VariantMatch::new(Vec::new(), usd(199), Vec::new()).unwrap()],
        Modifiers::new(Vec::new()),
    )
    .unwrap()
    .configure(&Selections::new())
    .unwrap();
    let order_item = OrderItem::from_configured_catalog_item(
        order_item_id("01CHIPSITEM"),
        catalog_version_id("01CATALOGVERSION"),
        1,
        &configured,
    )
    .unwrap();

    assert_eq!(order_item.variant_ids(), Some(&[][..]));
    assert!(order_item.variant_labels().is_empty());
    assert_eq!(order_item.variant_title(), None);
    assert_eq!(order_item.description(), "Bag of chips");
    assert_eq!(order_item.unit_price().amount_minor(), 199);

    let entries = order_item
        .entries(entry_id("01CHIPSENTRY"), Vec::new())
        .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].description(), "Bag of chips");
    assert_eq!(entries[0].unit_amount().amount_minor(), 199);
}

pub const UNCONNECTED_ORDER_ITEM_SUPPORTS_NONE_IDS_DOWN_TO_MODIFIERS: DescribedBehavior =
    DescribedBehavior::new(
        "unconnected order item supports none ids down to modifiers",
        "Manual order items can preserve labels, prompts, choices, entered inputs, and modifier price contributions without catalog IDs.",
        unconnected_order_item_supports_none_ids_down_to_modifiers,
    );

#[test]
fn unconnected_order_item_supports_none_ids_down_to_modifiers() {
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
                .unwrap()
                .with_inputs(vec![OrderItemChoiceInputSnapshot::new(
                    None,
                    Label::without_id("Any special requests?").unwrap(),
                    None,
                    "No onions",
                )]),
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
    assert_eq!(order_item.variant_ids(), None);
    assert!(order_item.variant_labels().is_empty());
    assert_eq!(order_item.variant_title(), None);
    assert_eq!(order_item.modifier_unit_price().amount_minor(), 50);
    assert_eq!(order_item.unit_price().amount_minor(), 1050);

    let prompt = &order_item.modifiers().prompts()[0];
    let choice = &prompt.choices()[0];
    let contribution = &order_item.modifiers().price().contributions()[0];

    assert_eq!(prompt.prompt_id(), None);
    assert_eq!(prompt.label().label_id(), None);
    assert_eq!(choice.choice_id(), None);
    assert_eq!(choice.label().label_id(), None);
    assert_eq!(choice.inputs()[0].input_id(), None);
    assert_eq!(choice.inputs()[0].label().label_id(), None);
    assert_eq!(choice.inputs()[0].value(), "No onions");
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

pub const ORDER_ITEM_EXPANDS_TO_BASE_AND_MODIFIER_ENTRIES: DescribedBehavior =
    DescribedBehavior::new(
        "order item expands to base and modifier entries",
        "A catalog-backed order item expands into one base item entry and one entry for each priced modifier contribution.",
        order_item_expands_to_base_and_modifier_entries,
    );

#[test]
fn order_item_expands_to_base_and_modifier_entries() {
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
            variant_ids: vec![variant_id("01LARGE"), variant_id("01THIN")],
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

pub const ORDER_ITEM_REJECTS_ZERO_QUANTITY_AND_WRONG_MODIFIER_ENTRY_ID_COUNT: DescribedBehavior =
    DescribedBehavior::new(
        "order item rejects zero quantity and wrong modifier entry id count",
        "Order item construction rejects zero quantity and entry expansion requires one modifier entry ID per priced contribution.",
        order_item_rejects_zero_quantity_and_wrong_modifier_entry_id_count,
    );

#[test]
fn order_item_rejects_zero_quantity_and_wrong_modifier_entry_id_count() {
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
            ChoiceSelection::new(component_id("01PEPPER"), 1)
                .with_inputs(vec![ChoiceInputValue::once(
                    component_id("01REQUEST"),
                    "Cook it well done",
                )])
                .with_modifiers(Selections::new().with_prompt(
                    component_id("01PLACE"),
                    vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                )),
        ],
    );

    pizza_catalog_item()
        .configure_variants(&[variant_id("01LARGE"), variant_id("01THIN")], &selections)
        .unwrap()
}

fn pizza_catalog_item() -> CatalogItem {
    CatalogItem::new(
        catalog_item_id("01PIZZA"),
        "Pizza",
        vec![
            VariantDimension::new(
                variant_dimension_id("01SIZE"),
                "Size",
                vec![Variant::new(variant_id("01LARGE"), "Large").unwrap()],
            )
            .unwrap(),
            VariantDimension::new(
                variant_dimension_id("01CRUST"),
                "Crust",
                vec![Variant::new(variant_id("01THIN"), "Thin").unwrap()],
            )
            .unwrap(),
        ],
        vec![large_pizza_match()],
        pizza_modifiers(),
    )
    .unwrap()
}

fn large_pizza_match() -> VariantMatch {
    VariantMatch::new(
        vec![variant_id("01LARGE"), variant_id("01THIN")],
        usd(1800),
        vec![variant_effect("large-thin")],
    )
    .unwrap()
    .with_label(Label::new(label_id("01LARGE-THIN-TITLE"), "Large thin").unwrap())
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
                .with_inputs(vec![
                    ChoiceInput::new(
                        component_id("01REQUEST"),
                        "Any special requests?",
                        false,
                        None,
                        Some(500),
                        false,
                    )
                    .unwrap(),
                ])
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

fn variant_dimension_id(suffix: &str) -> VariantDimensionId {
    VariantDimensionId::from_suffix(suffix).unwrap()
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
