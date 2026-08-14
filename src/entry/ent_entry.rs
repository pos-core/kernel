use std::fmt;

use crate::primitives::ids::{CatalogItemId, CatalogVersionId, ComponentId, EntryId, VariantId};
use crate::primitives::money::{Money, MoneyError};

#[doc = include_str!("order-entry.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrderEntry {
    entry_id: EntryId,
    kind: EntryKind,
    source: EntrySource,
    description: String,
    quantity: u32,
    unit_amount: Money,
    total_amount: Money,
    price_category: Option<PriceCategory>,
    accounting_category: Option<AccountingCategory>,
}

impl OrderEntry {
    pub fn builder(
        entry_id: EntryId,
        kind: EntryKind,
        source: EntrySource,
        description: impl Into<String>,
        quantity: u32,
        unit_amount: Money,
    ) -> OrderEntryBuilder {
        OrderEntryBuilder::new(entry_id, kind, source, description, quantity, unit_amount)
    }

    pub fn entry_id(&self) -> &EntryId {
        &self.entry_id
    }

    pub fn kind(&self) -> EntryKind {
        self.kind
    }

    pub fn source(&self) -> &EntrySource {
        &self.source
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn unit_amount(&self) -> &Money {
        &self.unit_amount
    }

    pub fn total_amount(&self) -> &Money {
        &self.total_amount
    }

    pub fn price_category(&self) -> Option<PriceCategory> {
        self.price_category
    }
}

#[derive(Debug, Clone)]
pub struct OrderEntryBuilder {
    entry_id: EntryId,
    kind: EntryKind,
    source: EntrySource,
    description: String,
    quantity: u32,
    unit_amount: Money,
    price_category: Option<PriceCategory>,
    accounting_category: Option<AccountingCategory>,
}

impl OrderEntryBuilder {
    fn new(
        entry_id: EntryId,
        kind: EntryKind,
        source: EntrySource,
        description: impl Into<String>,
        quantity: u32,
        unit_amount: Money,
    ) -> Self {
        Self {
            entry_id,
            kind,
            source,
            description: description.into(),
            quantity,
            unit_amount,
            price_category: None,
            accounting_category: None,
        }
    }

    pub fn with_price_category(mut self, price_category: PriceCategory) -> Self {
        self.price_category = Some(price_category);
        self
    }

    pub fn with_accounting_category(mut self, accounting_category: AccountingCategory) -> Self {
        self.accounting_category = Some(accounting_category);
        self
    }

    pub fn build(self) -> Result<OrderEntry, EntryError> {
        if self.quantity == 0 {
            return Err(EntryError::ZeroQuantity);
        }

        if self.description.trim().is_empty() {
            return Err(EntryError::EmptyDescription);
        }

        let total_amount = self
            .unit_amount
            .checked_mul_quantity(self.quantity)
            .map_err(EntryError::Money)?;

        Ok(OrderEntry {
            entry_id: self.entry_id,
            kind: self.kind,
            source: self.source,
            description: self.description,
            quantity: self.quantity,
            unit_amount: self.unit_amount,
            total_amount,
            price_category: self.price_category,
            accounting_category: self.accounting_category,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EntryKind {
    Category,
    Item,
    ExternalItem,
    Modifier,
    LineDiscount,
    OrderDiscount,
    CategoryDiscount,
    ThirdPartyDiscount,
    Fee,
    ServiceCharge,
    Tax,
    TaxAdjustment,
    TaxPaidByThirdParty,
    Note,
    SystemAdjustment,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntrySource {
    CatalogItem {
        catalog_version_id: CatalogVersionId,
        catalog_item_id: CatalogItemId,
        variant_id: VariantId,
    },
    Catalog {
        catalog_version_id: CatalogVersionId,
        component_id: ComponentId,
    },
    External {
        system: String,
        external_id: Option<String>,
        mapped_component_id: Option<ComponentId>,
    },
    Manual,
    System,
}

impl EntrySource {
    pub fn status(&self) -> EntrySourceStatus {
        match self {
            Self::CatalogItem { .. } => EntrySourceStatus::CatalogBacked,
            Self::Catalog { .. } => EntrySourceStatus::CatalogBacked,
            Self::External {
                mapped_component_id,
                ..
            } if mapped_component_id.is_some() => EntrySourceStatus::ExternalMapped,
            Self::External { .. } => EntrySourceStatus::ExternalUnmapped,
            Self::Manual => EntrySourceStatus::Manual,
            Self::System => EntrySourceStatus::System,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EntrySourceStatus {
    CatalogBacked,
    ExternalMapped,
    ExternalUnmapped,
    Manual,
    System,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PriceCategory {
    BaseItem,
    Modifier,
    Upgrade,
    Included,
    Deposit,
    Package,
    Fee,
    ServiceCharge,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AccountingCategory {
    Food,
    Beverage,
    Tax,
    Discount,
    Fee,
    ServiceCharge,
    Tip,
    Other,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntryError {
    ZeroQuantity,
    EmptyDescription,
    Money(MoneyError),
}

impl fmt::Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroQuantity => f.write_str("entry quantity must be greater than zero"),
            Self::EmptyDescription => f.write_str("entry description cannot be empty"),
            Self::Money(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for EntryError {}
