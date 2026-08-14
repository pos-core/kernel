use std::fmt;

use crate::entry::{EntryKind, OrderEntry};
use crate::order::Order;
use crate::primitives::ids::EntryId;
use crate::primitives::money::{CurrencyCode, Money, MoneyError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Total {
    category: TotalCategory,
    amount: Money,
    source_entry_ids: Vec<EntryId>,
}

impl Total {
    pub fn new(category: TotalCategory, amount: Money, source_entry_ids: Vec<EntryId>) -> Self {
        Self {
            category,
            amount,
            source_entry_ids,
        }
    }

    pub fn category(&self) -> TotalCategory {
        self.category
    }

    pub fn amount(&self) -> &Money {
        &self.amount
    }

    pub fn source_entry_ids(&self) -> &[EntryId] {
        &self.source_entry_ids
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TotalCategory {
    GrossSales,
    Discounts,
    Fees,
    ServiceCharges,
    Tax,
    TaxPaidByThirdParty,
    AmountDue,
}

pub fn calculate_order_totals(order: &Order) -> Result<Vec<Total>, TotalError> {
    let Some(currency) = first_currency(order.entries()) else {
        return Ok(Vec::new());
    };

    let mut buckets = Buckets::new(currency);

    for entry in order.entries() {
        buckets.add_entry(entry)?;
    }

    Ok(buckets.into_totals())
}

fn first_currency<'a>(entries: impl Iterator<Item = &'a OrderEntry>) -> Option<CurrencyCode> {
    entries
        .map(|entry| entry.total_amount().currency().clone())
        .next()
}

struct Buckets {
    gross_sales: TotalBucket,
    discounts: TotalBucket,
    fees: TotalBucket,
    service_charges: TotalBucket,
    tax: TotalBucket,
    tax_paid_by_third_party: TotalBucket,
    amount_due: TotalBucket,
}

impl Buckets {
    fn new(currency: CurrencyCode) -> Self {
        Self {
            gross_sales: TotalBucket::new(TotalCategory::GrossSales, currency.clone()),
            discounts: TotalBucket::new(TotalCategory::Discounts, currency.clone()),
            fees: TotalBucket::new(TotalCategory::Fees, currency.clone()),
            service_charges: TotalBucket::new(TotalCategory::ServiceCharges, currency.clone()),
            tax: TotalBucket::new(TotalCategory::Tax, currency.clone()),
            tax_paid_by_third_party: TotalBucket::new(
                TotalCategory::TaxPaidByThirdParty,
                currency.clone(),
            ),
            amount_due: TotalBucket::new(TotalCategory::AmountDue, currency),
        }
    }

    fn add_entry(&mut self, entry: &OrderEntry) -> Result<(), TotalError> {
        match entry.kind() {
            EntryKind::Item | EntryKind::ExternalItem | EntryKind::Modifier => {
                self.gross_sales.add(entry)?;
                self.amount_due.add(entry)?;
            }
            EntryKind::LineDiscount
            | EntryKind::OrderDiscount
            | EntryKind::CategoryDiscount
            | EntryKind::ThirdPartyDiscount => {
                self.discounts.add(entry)?;
                self.amount_due.add(entry)?;
            }
            EntryKind::Fee => {
                self.fees.add(entry)?;
                self.amount_due.add(entry)?;
            }
            EntryKind::ServiceCharge => {
                self.service_charges.add(entry)?;
                self.amount_due.add(entry)?;
            }
            EntryKind::Tax | EntryKind::TaxAdjustment => {
                self.tax.add(entry)?;
                self.amount_due.add(entry)?;
            }
            EntryKind::TaxPaidByThirdParty => {
                self.tax_paid_by_third_party.add(entry)?;
            }
            EntryKind::Category | EntryKind::Note | EntryKind::SystemAdjustment => {}
        }

        Ok(())
    }

    fn into_totals(self) -> Vec<Total> {
        vec![
            self.gross_sales.into_total(),
            self.discounts.into_total(),
            self.fees.into_total(),
            self.service_charges.into_total(),
            self.tax.into_total(),
            self.tax_paid_by_third_party.into_total(),
            self.amount_due.into_total(),
        ]
    }
}

struct TotalBucket {
    category: TotalCategory,
    amount: Money,
    source_entry_ids: Vec<EntryId>,
}

impl TotalBucket {
    fn new(category: TotalCategory, currency: CurrencyCode) -> Self {
        Self {
            category,
            amount: Money::zero(currency),
            source_entry_ids: Vec::new(),
        }
    }

    fn add(&mut self, entry: &OrderEntry) -> Result<(), TotalError> {
        self.amount = self
            .amount
            .checked_add(entry.total_amount())
            .map_err(TotalError::Money)?;
        self.source_entry_ids.push(entry.entry_id().clone());
        Ok(())
    }

    fn into_total(self) -> Total {
        Total::new(self.category, self.amount, self.source_entry_ids)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TotalError {
    Money(MoneyError),
}

impl fmt::Display for TotalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Money(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for TotalError {}
