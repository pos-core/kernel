use pos_core_kernel::prelude::*;

#[test]
fn order_events_replay_to_same_state_and_totals() {
    let usd = CurrencyCode::parse("USD").unwrap();
    let actor = ActorContext::new(id::<ActorId>("01ACTOR"), ActorKind::StaffUser);
    let order_id = id::<OrderId>("01ORDER");
    let catalog_version_id = id::<CatalogVersionId>("01CATALOGVERSION");

    let open = envelope(
        "01EVENTOPEN",
        actor.clone(),
        1,
        Order::open_event(order_id.clone()),
    );

    let mut order = Order::replay(&[open.clone()]).unwrap();

    let burger = OrderEntry::builder(
        id::<EntryId>("01BURGERENTRY"),
        EntryKind::Item,
        EntrySource::Catalog {
            catalog_version_id: catalog_version_id.clone(),
            component_id: id::<ComponentId>("01BURGERCOMPONENT"),
        },
        "Burger",
        1,
        Money::new(1000, usd.clone()),
    )
    .with_price_category(PriceCategory::BaseItem)
    .build()
    .unwrap();

    let add_burger = order.add_entry_event(burger).unwrap();
    order.apply(&add_burger).unwrap();

    let cheese = OrderEntry::builder(
        id::<EntryId>("01CHEESEENTRY"),
        EntryKind::Modifier,
        EntrySource::Catalog {
            catalog_version_id,
            component_id: id::<ComponentId>("01CHEESECOMPONENT"),
        },
        "Cheese",
        1,
        Money::new(100, usd.clone()),
    )
    .with_price_category(PriceCategory::Modifier)
    .build()
    .unwrap();

    let add_cheese = order.add_entry_event(cheese).unwrap();
    order.apply(&add_cheese).unwrap();

    let external_item = OrderEntry::builder(
        id::<EntryId>("01EXTERNALENTRY"),
        EntryKind::ExternalItem,
        EntrySource::External {
            system: "marketplace".to_owned(),
            external_id: Some("outside-123".to_owned()),
            mapped_component_id: None,
        },
        "Marketplace Special",
        1,
        Money::new(500, usd.clone()),
    )
    .with_price_category(PriceCategory::BaseItem)
    .build()
    .unwrap();

    assert_eq!(
        external_item.source().status(),
        EntrySourceStatus::ExternalUnmapped
    );

    let add_external = order.add_entry_event(external_item).unwrap();
    order.apply(&add_external).unwrap();

    let discount = OrderEntry::builder(
        id::<EntryId>("01DISCOUNTENTRY"),
        EntryKind::LineDiscount,
        EntrySource::System,
        "Item Discount",
        1,
        Money::new(-200, usd),
    )
    .build()
    .unwrap();

    let add_discount = order.add_entry_event(discount).unwrap();
    order.apply(&add_discount).unwrap();

    let events = vec![
        open,
        envelope("01EVENTBURGER", actor.clone(), 2, add_burger),
        envelope("01EVENTCHEESE", actor.clone(), 3, add_cheese),
        envelope("01EVENTEXTERNAL", actor.clone(), 4, add_external),
        envelope("01EVENTDISCOUNT", actor, 5, add_discount),
    ];

    let replayed = Order::replay(&events).unwrap();

    assert_eq!(order, replayed);

    let totals = calculate_order_totals(&replayed).unwrap();
    let amount_due = totals
        .iter()
        .find(|total| total.category() == TotalCategory::AmountDue)
        .unwrap();

    assert_eq!(amount_due.amount().amount_minor(), 1400);
    assert_eq!(amount_due.source_entry_ids().len(), 4);
}

fn envelope(
    suffix: &str,
    actor: ActorContext,
    occurred_at: i64,
    payload: OrderEvent,
) -> EventEnvelope<OrderEvent> {
    EventEnvelope::new(
        EventId::from_suffix(suffix).unwrap(),
        actor,
        UtcTime::from_unix_millis(occurred_at),
        payload,
    )
}

fn id<T>(suffix: &str) -> T
where
    T: FromSuffix,
{
    T::from_suffix(suffix)
}

trait FromSuffix: Sized {
    fn from_suffix(suffix: &str) -> Self;
}

macro_rules! impl_from_suffix {
    ($($name:ty),+ $(,)?) => {
        $(
            impl FromSuffix for $name {
                fn from_suffix(suffix: &str) -> Self {
                    <$name>::from_suffix(suffix).unwrap()
                }
            }
        )+
    };
}

impl_from_suffix!(ActorId, CatalogVersionId, ComponentId, EntryId, OrderId,);
