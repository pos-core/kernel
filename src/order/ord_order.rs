use std::collections::BTreeMap;
use std::fmt;

use crate::entry::OrderEntry;
use crate::event::EventEnvelope;
use crate::primitives::ids::{EntryId, OrderId};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Order {
    order_id: OrderId,
    status: OrderStatus,
    entries: BTreeMap<EntryId, OrderEntry>,
}

impl Order {
    pub fn new(order_id: OrderId) -> Self {
        Self {
            order_id,
            status: OrderStatus::Open,
            entries: BTreeMap::new(),
        }
    }

    pub fn order_id(&self) -> &OrderId {
        &self.order_id
    }

    pub fn status(&self) -> OrderStatus {
        self.status
    }

    pub fn entries(&self) -> impl Iterator<Item = &OrderEntry> {
        self.entries.values()
    }

    pub fn get_entry(&self, entry_id: &EntryId) -> Option<&OrderEntry> {
        self.entries.get(entry_id)
    }

    pub fn open_event(order_id: OrderId) -> OrderEvent {
        OrderEvent::OrderOpened { order_id }
    }

    pub fn add_entry_event(&self, entry: OrderEntry) -> Result<OrderEvent, OrderError> {
        if self.status != OrderStatus::Open {
            return Err(OrderError::OrderNotOpen);
        }

        if self.entries.contains_key(entry.entry_id()) {
            return Err(OrderError::DuplicateEntry(entry.entry_id().clone()));
        }

        Ok(OrderEvent::EntryAdded {
            order_id: self.order_id.clone(),
            entry,
        })
    }

    pub fn apply(&mut self, event: &OrderEvent) -> Result<(), OrderError> {
        match event {
            OrderEvent::OrderOpened { order_id } => {
                if &self.order_id != order_id {
                    return Err(OrderError::WrongOrder {
                        expected: self.order_id.clone(),
                        actual: order_id.clone(),
                    });
                }

                self.status = OrderStatus::Open;
                Ok(())
            }
            OrderEvent::EntryAdded { order_id, entry } => {
                self.ensure_order(order_id)?;

                if self.entries.contains_key(entry.entry_id()) {
                    return Err(OrderError::DuplicateEntry(entry.entry_id().clone()));
                }

                self.entries.insert(entry.entry_id().clone(), entry.clone());
                Ok(())
            }
        }
    }

    pub fn replay(events: &[EventEnvelope<OrderEvent>]) -> Result<Self, OrderError> {
        let mut order = None;

        for envelope in events {
            match envelope.payload() {
                OrderEvent::OrderOpened { order_id } => {
                    if order.is_some() {
                        return Err(OrderError::AlreadyOpened);
                    }

                    order = Some(Self::new(order_id.clone()));
                }
                event => {
                    let order = order.as_mut().ok_or(OrderError::MissingOpenEvent)?;
                    order.apply(event)?;
                }
            }
        }

        order.ok_or(OrderError::MissingOpenEvent)
    }

    fn ensure_order(&self, order_id: &OrderId) -> Result<(), OrderError> {
        if &self.order_id == order_id {
            Ok(())
        } else {
            Err(OrderError::WrongOrder {
                expected: self.order_id.clone(),
                actual: order_id.clone(),
            })
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum OrderStatus {
    Open,
    Voided,
    Closed,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrderEvent {
    OrderOpened {
        order_id: OrderId,
    },
    EntryAdded {
        order_id: OrderId,
        entry: OrderEntry,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrderError {
    MissingOpenEvent,
    AlreadyOpened,
    WrongOrder { expected: OrderId, actual: OrderId },
    OrderNotOpen,
    DuplicateEntry(EntryId),
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOpenEvent => f.write_str("order replay is missing an open event"),
            Self::AlreadyOpened => f.write_str("order replay contains more than one open event"),
            Self::WrongOrder { expected, actual } => {
                write!(
                    f,
                    "event for order `{actual}` cannot be applied to `{expected}`"
                )
            }
            Self::OrderNotOpen => f.write_str("order is not open"),
            Self::DuplicateEntry(entry_id) => write!(f, "duplicate order entry `{entry_id}`"),
        }
    }
}

impl std::error::Error for OrderError {}
