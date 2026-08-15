//! POS Core Kernel.
//!
//! The core crate owns deterministic domain behavior only. It does not know
//! about UI, storage, payment processors, concrete surfaces, or concrete
//! fulfillment workflows.

pub mod actor;
pub mod catalog_item;
pub mod effect;
pub mod entry;
pub mod event;
pub mod modifier;
pub mod order;
pub mod order_item;
pub mod primitives;
pub mod supply;
pub mod totals;

pub mod prelude {
    pub use crate::actor::{ActorContext, ActorKind};
    pub use crate::catalog_item::{
        CatalogItem, CatalogItemError, ConfiguredCatalogItem, Variant, VariantDimension,
        VariantMatch, VariantSelectionStep, VariantSettings,
    };
    pub use crate::effect::{
        Effect, EffectDomain, EffectPayload, EffectRequirement, EffectSource, EffectTarget,
    };
    pub use crate::entry::{
        AccountingCategory, EntryError, EntryKind, EntrySource, EntrySourceStatus, OrderEntry,
        OrderEntryBuilder, PriceCategory,
    };
    pub use crate::event::EventEnvelope;
    pub use crate::modifier::{
        Choice, ChoiceConfiguration, ChoiceInput, ChoiceInputConfiguration, ChoiceInputError,
        ChoiceInputSnapshot, ChoiceInputValue, ChoicePrice, ChoiceSelection, ChoiceSnapshot,
        Configuration, ConfigurationSnapshot, ModifierApplicability, ModifierError, ModifierNode,
        ModifierPricingPolicy, Modifiers, PriceContribution, PriceFactor, PricedConfiguration,
        Prompt, PromptConfiguration, PromptSelection, PromptSnapshot, Rule, RuleKind,
        SelectionSource, Selections, ValidatedChoiceSelection,
    };
    pub use crate::order::{Order, OrderError, OrderEvent, OrderStatus};
    pub use crate::order_item::{
        OrderItem, OrderItemChoiceInputSnapshot, OrderItemChoiceSnapshot, OrderItemError,
        OrderItemModifierPrice, OrderItemModifierSnapshot, OrderItemPriceContribution,
        OrderItemPriceFactor, OrderItemPromptSnapshot, OrderItemSource,
    };
    pub use crate::primitives::calendar::{
        CalendarError, CalendarMoment, DayOfWeek, DaysOfWeek, LocalTimeOfDay, LocalTimeRange,
        LogicalDate, LogicalDateRange,
    };
    pub use crate::primitives::consumer::{
        ConsumerAttribute, ConsumerAttributeError, ConsumerProfile, ConsumerProfileError,
    };
    pub use crate::primitives::ids::{
        ActorId, AllocationId, BagId, CatalogId, CatalogItemId, CatalogVersionId, CheckId,
        ClientOperationId, CommandId, ComponentId, ComponentVersionId, ConsumerAttributeId,
        CustomerId, DeviceId, EdgeId, EntryId, EventId, FulfillmentModeId, LabelId, MediaId,
        MerchantId, OrderId, OrderItemId, PaymentId, PermissionId, RoleId, SupplyClaimId,
        SurfaceId, TransactionId, UserId, VariantDimensionId, VariantId,
    };
    pub use crate::primitives::label::{Label, LabelError, LabelValue, ResolvedLabel};
    pub use crate::primitives::media::{
        Media, MediaCollection, MediaDimensions, MediaError, MediaMimeType, MediaVariant,
        ResolvedMedia,
    };
    pub use crate::primitives::money::{
        CurrencyCode, Money, Rate, RationalMoney, RoundingStrategy,
    };
    pub use crate::primitives::schedule::{
        Schedule, ScheduleContext, ScheduleError, ScheduleLimit, ScheduleWindow, UtcTimeRange,
    };
    pub use crate::primitives::time::{EvaluationTime, TimeError, TimeZone, UtcTime};
    pub use crate::supply::{
        AvailableSupply, SupplyBucket, SupplyClaimState, SupplyConsume, SupplyError, SupplyKey,
        SupplyLedger, SupplyOperation, SupplyOperationKind, SupplyProvider, SupplyRequest,
        SupplyReserve, SupplyResolution, SupplyTarget, SupplyUnavailableReason, SupplyUnconsume,
        SupplyUnreserve, SupplyUnresolvedReason, SupplyView,
    };
    pub use crate::totals::{Total, TotalCategory, calculate_order_totals};
}
