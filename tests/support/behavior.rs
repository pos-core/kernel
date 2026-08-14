use pos_core_kernel::prelude::*;

use crate::support::md_report::ReportCase;

#[derive(Clone, Copy)]
pub struct DescribedBehavior {
    name: &'static str,
    description: &'static str,
    assertions: fn(),
}

impl DescribedBehavior {
    pub const fn new(name: &'static str, description: &'static str, assertions: fn()) -> Self {
        Self {
            name,
            description,
            assertions,
        }
    }

    pub fn report_case(self) -> ReportCase {
        ReportCase {
            name: self.name,
            description: self.description,
            run: self.assertions,
        }
    }
}

pub fn usd(amount_minor: i64) -> Money {
    Money::new(amount_minor, CurrencyCode::parse("USD").unwrap())
}

pub fn catalog_item_id(suffix: &str) -> CatalogItemId {
    CatalogItemId::from_suffix(suffix).unwrap()
}

pub fn variant_id(suffix: &str) -> VariantId {
    VariantId::from_suffix(suffix).unwrap()
}

pub fn component_id(suffix: &str) -> ComponentId {
    ComponentId::from_suffix(suffix).unwrap()
}

pub fn consumer_attribute_id(suffix: &str) -> ConsumerAttributeId {
    ConsumerAttributeId::from_suffix(suffix).unwrap()
}

pub fn label_id(suffix: &str) -> LabelId {
    LabelId::from_suffix(suffix).unwrap()
}

pub fn media_id(suffix: &str) -> MediaId {
    MediaId::from_suffix(suffix).unwrap()
}

pub fn supply_claim_id(suffix: &str) -> SupplyClaimId {
    SupplyClaimId::from_suffix(suffix).unwrap()
}

pub fn supply_request(target: SupplyTarget, quantity: u32) -> SupplyRequest {
    SupplyRequest::new(target, quantity).unwrap()
}

pub fn supply_bucket(key: &str, value: &str) -> SupplyBucket {
    SupplyBucket::empty().with_dimension(key, value).unwrap()
}

pub fn schedule_context(
    unix_millis: i64,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> ScheduleContext {
    ScheduleContext::new(
        utc(unix_millis),
        CalendarMoment::new(
            LogicalDate::new(year, month, day).unwrap(),
            LocalTimeOfDay::from_hms(hour, minute, second).unwrap(),
            TimeZone::utc(),
        ),
    )
}

pub fn evaluation_time(
    unix_millis: i64,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> EvaluationTime {
    EvaluationTime::new(
        utc(unix_millis),
        CalendarMoment::new(
            LogicalDate::new(year, month, day).unwrap(),
            LocalTimeOfDay::from_hms(hour, minute, second).unwrap(),
            TimeZone::utc(),
        ),
    )
}

pub fn report_evaluation_time() -> EvaluationTime {
    EvaluationTime::new(
        UtcTime::from_unix_millis(1_779_452_977_000),
        CalendarMoment::new(
            LogicalDate::new(2026, 5, 22).unwrap(),
            LocalTimeOfDay::from_hms(12, 29, 37).unwrap(),
            TimeZone::utc(),
        ),
    )
}

pub fn utc(unix_millis: i64) -> UtcTime {
    UtcTime::from_unix_millis(unix_millis)
}

pub fn mime(value: &str) -> MediaMimeType {
    MediaMimeType::parse(value).unwrap()
}
