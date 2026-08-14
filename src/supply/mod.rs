use std::collections::BTreeMap;
use std::fmt;

use crate::primitives::ids::{CatalogItemId, ComponentId, SupplyClaimId};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SupplyTarget {
    Choice(ComponentId),
    CatalogItem(CatalogItemId),
    Custom(SupplyKey),
}

impl SupplyTarget {
    pub fn choice(choice_id: ComponentId) -> Self {
        Self::Choice(choice_id)
    }

    pub fn catalog_item(catalog_item_id: CatalogItemId) -> Self {
        Self::CatalogItem(catalog_item_id)
    }

    pub fn custom(value: impl Into<String>) -> Result<Self, SupplyError> {
        Ok(Self::Custom(SupplyKey::new(value)?))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SupplyKey(String);

impl SupplyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, SupplyError> {
        let value = value.into();
        let value = value.trim();

        if value.is_empty() {
            return Err(SupplyError::EmptySupplyKey);
        }

        let valid = value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        });

        if !valid {
            return Err(SupplyError::InvalidSupplyKey(value.to_owned()));
        }

        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SupplyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[doc = include_str!("supply-bucket.md")]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct SupplyBucket {
    dimensions: BTreeMap<String, String>,
}

impl SupplyBucket {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_dimension(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, SupplyError> {
        let key = key.into();
        let value = value.into();
        let key = key.trim();
        let value = value.trim();

        if key.is_empty() {
            return Err(SupplyError::EmptyBucketDimensionKey);
        }

        if value.is_empty() {
            return Err(SupplyError::EmptyBucketDimensionValue {
                key: key.to_owned(),
            });
        }

        if self
            .dimensions
            .insert(key.to_owned(), value.to_owned())
            .is_some()
        {
            return Err(SupplyError::DuplicateBucketDimension(key.to_owned()));
        }

        Ok(self)
    }

    pub fn dimensions(&self) -> &BTreeMap<String, String> {
        &self.dimensions
    }

    pub fn is_empty(&self) -> bool {
        self.dimensions.is_empty()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SupplyRequest {
    target: SupplyTarget,
    quantity: u32,
    bucket: SupplyBucket,
}

impl SupplyRequest {
    pub fn new(target: SupplyTarget, quantity: u32) -> Result<Self, SupplyError> {
        if quantity == 0 {
            return Err(SupplyError::ZeroSupplyQuantity);
        }

        Ok(Self {
            target,
            quantity,
            bucket: SupplyBucket::empty(),
        })
    }

    pub fn with_bucket(mut self, bucket: SupplyBucket) -> Self {
        self.bucket = bucket;
        self
    }

    pub fn target(&self) -> &SupplyTarget {
        &self.target
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn bucket(&self) -> &SupplyBucket {
        &self.bucket
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AvailableSupply {
    target: SupplyTarget,
    bucket: SupplyBucket,
    quantity: u32,
}

impl AvailableSupply {
    pub fn new(target: SupplyTarget, bucket: SupplyBucket, quantity: u32) -> Self {
        Self {
            target,
            bucket,
            quantity,
        }
    }

    pub fn target(&self) -> &SupplyTarget {
        &self.target
    }

    pub fn bucket(&self) -> &SupplyBucket {
        &self.bucket
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }
}

pub trait SupplyProvider {
    fn resolve_supply(&self, request: &SupplyRequest) -> SupplyResolution;
}

#[doc = include_str!("supply-view.md")]
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct SupplyView {
    available: BTreeMap<SupplyAvailabilityKey, AvailableSupply>,
}

impl SupplyView {
    pub fn new(available: Vec<AvailableSupply>) -> Result<Self, SupplyError> {
        let mut view = Self::default();

        for available_supply in available {
            view = view.with_available(available_supply)?;
        }

        Ok(view)
    }

    pub fn with_available(mut self, available: AvailableSupply) -> Result<Self, SupplyError> {
        let key = SupplyAvailabilityKey::new(available.target.clone(), available.bucket.clone());

        if self.available.insert(key.clone(), available).is_some() {
            return Err(SupplyError::DuplicateAvailableSupply {
                target: key.target,
                bucket: key.bucket,
            });
        }

        Ok(self)
    }

    pub fn available(&self) -> impl Iterator<Item = &AvailableSupply> {
        self.available.values()
    }
}

impl SupplyProvider for SupplyView {
    fn resolve_supply(&self, request: &SupplyRequest) -> SupplyResolution {
        let key = SupplyAvailabilityKey::new(request.target.clone(), request.bucket.clone());
        let Some(available) = self.available.get(&key) else {
            return SupplyResolution::Unresolved {
                reason: SupplyUnresolvedReason::MissingAvailableSupply,
            };
        };

        if request.quantity <= available.quantity {
            SupplyResolution::Available {
                requested: request.quantity,
                available: available.quantity,
            }
        } else {
            SupplyResolution::Unavailable {
                requested: request.quantity,
                available: available.quantity,
                reason: SupplyUnavailableReason::InsufficientSupply,
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SupplyResolution {
    Available {
        requested: u32,
        available: u32,
    },
    Unavailable {
        requested: u32,
        available: u32,
        reason: SupplyUnavailableReason,
    },
    Unresolved {
        reason: SupplyUnresolvedReason,
    },
}

impl SupplyResolution {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }

    pub fn is_unresolved(&self) -> bool {
        matches!(self, Self::Unresolved { .. })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SupplyUnavailableReason {
    InsufficientSupply,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SupplyUnresolvedReason {
    MissingAvailableSupply,
    ProviderUnavailable,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SupplyOperation {
    Reserve(SupplyReserve),
    Unreserve(SupplyUnreserve),
    Consume(SupplyConsume),
    Unconsume(SupplyUnconsume),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SupplyReserve {
    claim_id: SupplyClaimId,
    request: SupplyRequest,
}

impl SupplyReserve {
    pub fn new(claim_id: SupplyClaimId, request: SupplyRequest) -> Self {
        Self { claim_id, request }
    }

    pub fn claim_id(&self) -> &SupplyClaimId {
        &self.claim_id
    }

    pub fn request(&self) -> &SupplyRequest {
        &self.request
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SupplyUnreserve {
    claim_id: SupplyClaimId,
}

impl SupplyUnreserve {
    pub fn new(claim_id: SupplyClaimId) -> Self {
        Self { claim_id }
    }

    pub fn claim_id(&self) -> &SupplyClaimId {
        &self.claim_id
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SupplyConsume {
    claim_id: SupplyClaimId,
    request: SupplyRequest,
}

impl SupplyConsume {
    pub fn new(claim_id: SupplyClaimId, request: SupplyRequest) -> Self {
        Self { claim_id, request }
    }

    pub fn claim_id(&self) -> &SupplyClaimId {
        &self.claim_id
    }

    pub fn request(&self) -> &SupplyRequest {
        &self.request
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SupplyUnconsume {
    claim_id: SupplyClaimId,
}

impl SupplyUnconsume {
    pub fn new(claim_id: SupplyClaimId) -> Self {
        Self { claim_id }
    }

    pub fn claim_id(&self) -> &SupplyClaimId {
        &self.claim_id
    }
}

#[doc = include_str!("supply-ledger.md")]
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct SupplyLedger {
    claims: BTreeMap<SupplyClaimId, SupplyClaimState>,
}

impl SupplyLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, operation: SupplyOperation) -> Result<(), SupplyError> {
        match operation {
            SupplyOperation::Reserve(operation) => self.reserve(operation),
            SupplyOperation::Unreserve(operation) => self.unreserve(operation),
            SupplyOperation::Consume(operation) => self.consume(operation),
            SupplyOperation::Unconsume(operation) => self.unconsume(operation),
        }
    }

    pub fn claim(&self, claim_id: &SupplyClaimId) -> Option<&SupplyClaimState> {
        self.claims.get(claim_id)
    }

    pub fn reserved_quantity(&self, target: &SupplyTarget, bucket: &SupplyBucket) -> u32 {
        self.claims
            .values()
            .filter_map(|claim| match claim {
                SupplyClaimState::Reserved { request } if request.matches(target, bucket) => {
                    Some(request.quantity())
                }
                _ => None,
            })
            .sum()
    }

    pub fn consumed_quantity(&self, target: &SupplyTarget, bucket: &SupplyBucket) -> u32 {
        self.claims
            .values()
            .filter_map(|claim| match claim {
                SupplyClaimState::Consumed { request } if request.matches(target, bucket) => {
                    Some(request.quantity())
                }
                _ => None,
            })
            .sum()
    }

    fn reserve(&mut self, operation: SupplyReserve) -> Result<(), SupplyError> {
        if self.claims.contains_key(operation.claim_id()) {
            return Err(SupplyError::DuplicateSupplyClaim(
                operation.claim_id().clone(),
            ));
        }

        self.claims.insert(
            operation.claim_id,
            SupplyClaimState::Reserved {
                request: operation.request,
            },
        );

        Ok(())
    }

    fn unreserve(&mut self, operation: SupplyUnreserve) -> Result<(), SupplyError> {
        let claim = self
            .claims
            .get_mut(operation.claim_id())
            .ok_or_else(|| SupplyError::UnknownSupplyClaim(operation.claim_id().clone()))?;

        match claim.clone() {
            SupplyClaimState::Reserved { request } => {
                *claim = SupplyClaimState::Unreserved { request };
                Ok(())
            }
            current => Err(SupplyError::InvalidSupplyTransition {
                claim_id: operation.claim_id().clone(),
                current,
                operation: SupplyOperationKind::Unreserve,
            }),
        }
    }

    fn consume(&mut self, operation: SupplyConsume) -> Result<(), SupplyError> {
        if let Some(claim) = self.claims.get_mut(operation.claim_id()) {
            return match claim.clone() {
                SupplyClaimState::Reserved { request } if request == operation.request => {
                    *claim = SupplyClaimState::Consumed { request };
                    Ok(())
                }
                SupplyClaimState::Reserved { request } => Err(SupplyError::MismatchedSupplyClaim {
                    claim_id: operation.claim_id().clone(),
                    expected: request,
                    actual: operation.request,
                }),
                current => Err(SupplyError::InvalidSupplyTransition {
                    claim_id: operation.claim_id().clone(),
                    current,
                    operation: SupplyOperationKind::Consume,
                }),
            };
        }

        self.claims.insert(
            operation.claim_id,
            SupplyClaimState::Consumed {
                request: operation.request,
            },
        );

        Ok(())
    }

    fn unconsume(&mut self, operation: SupplyUnconsume) -> Result<(), SupplyError> {
        let claim = self
            .claims
            .get_mut(operation.claim_id())
            .ok_or_else(|| SupplyError::UnknownSupplyClaim(operation.claim_id().clone()))?;

        match claim.clone() {
            SupplyClaimState::Consumed { request } => {
                *claim = SupplyClaimState::Unconsumed { request };
                Ok(())
            }
            current => Err(SupplyError::InvalidSupplyTransition {
                claim_id: operation.claim_id().clone(),
                current,
                operation: SupplyOperationKind::Unconsume,
            }),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SupplyClaimState {
    Reserved { request: SupplyRequest },
    Unreserved { request: SupplyRequest },
    Consumed { request: SupplyRequest },
    Unconsumed { request: SupplyRequest },
}

impl SupplyClaimState {
    pub fn request(&self) -> &SupplyRequest {
        match self {
            Self::Reserved { request }
            | Self::Unreserved { request }
            | Self::Consumed { request }
            | Self::Unconsumed { request } => request,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SupplyOperationKind {
    Reserve,
    Unreserve,
    Consume,
    Unconsume,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SupplyError {
    EmptySupplyKey,
    InvalidSupplyKey(String),
    EmptyBucketDimensionKey,
    EmptyBucketDimensionValue {
        key: String,
    },
    DuplicateBucketDimension(String),
    ZeroSupplyQuantity,
    DuplicateAvailableSupply {
        target: SupplyTarget,
        bucket: SupplyBucket,
    },
    DuplicateSupplyClaim(SupplyClaimId),
    UnknownSupplyClaim(SupplyClaimId),
    InvalidSupplyTransition {
        claim_id: SupplyClaimId,
        current: SupplyClaimState,
        operation: SupplyOperationKind,
    },
    MismatchedSupplyClaim {
        claim_id: SupplyClaimId,
        expected: SupplyRequest,
        actual: SupplyRequest,
    },
}

impl fmt::Display for SupplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySupplyKey => f.write_str("supply key cannot be empty"),
            Self::InvalidSupplyKey(value) => write!(f, "invalid supply key `{value}`"),
            Self::EmptyBucketDimensionKey => f.write_str("supply bucket key cannot be empty"),
            Self::EmptyBucketDimensionValue { key } => {
                write!(f, "supply bucket value for `{key}` cannot be empty")
            }
            Self::DuplicateBucketDimension(key) => {
                write!(f, "duplicate supply bucket dimension `{key}`")
            }
            Self::ZeroSupplyQuantity => f.write_str("supply quantity cannot be zero"),
            Self::DuplicateAvailableSupply { .. } => {
                f.write_str("duplicate available supply for target and bucket")
            }
            Self::DuplicateSupplyClaim(claim_id) => {
                write!(f, "duplicate supply claim `{claim_id}`")
            }
            Self::UnknownSupplyClaim(claim_id) => {
                write!(f, "unknown supply claim `{claim_id}`")
            }
            Self::InvalidSupplyTransition {
                claim_id,
                current,
                operation,
            } => write!(
                f,
                "cannot apply supply operation `{operation:?}` to claim `{claim_id}` in state `{current:?}`"
            ),
            Self::MismatchedSupplyClaim { claim_id, .. } => {
                write!(f, "supply claim `{claim_id}` request does not match")
            }
        }
    }
}

impl std::error::Error for SupplyError {}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct SupplyAvailabilityKey {
    target: SupplyTarget,
    bucket: SupplyBucket,
}

impl SupplyAvailabilityKey {
    fn new(target: SupplyTarget, bucket: SupplyBucket) -> Self {
        Self { target, bucket }
    }
}

impl SupplyRequest {
    fn matches(&self, target: &SupplyTarget, bucket: &SupplyBucket) -> bool {
        self.target() == target && self.bucket() == bucket
    }
}
