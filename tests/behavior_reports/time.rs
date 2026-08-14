use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "time",
        title: "Time",
        description: "Described behavior tests for UTC time and time zone primitives.",
        definitions: vec![
            DefinitionLink::new("UTC time", "../src/primitives/time/utc-time.md"),
            DefinitionLink::new(
                "Evaluation time",
                "../src/primitives/time/evaluation-time.md",
            ),
        ],
        cases: vec![
            UTC_TIME_STORES_UNIX_MILLISECONDS.report_case(),
            UTC_TIME_CONVERTS_TO_AND_FROM_SYSTEM_TIME.report_case(),
            UTC_TIME_SUPPORTS_CHECKED_MILLISECOND_ARITHMETIC.report_case(),
            EVALUATION_TIME_PAIRS_UTC_AND_CALENDAR_TIME.report_case(),
            TIME_ZONE_VALIDATES_IANA_SHAPED_NAMES.report_case(),
        ],
    }
}

pub const UTC_TIME_STORES_UNIX_MILLISECONDS: DescribedBehavior = DescribedBehavior::new(
    "utc time stores unix milliseconds",
    "UtcTime is a UTC instant represented by Unix milliseconds and does not own an ID.",
    utc_time_stores_unix_milliseconds,
);

#[test]
fn utc_time_stores_unix_milliseconds() {
    let time = UtcTime::from_unix_millis(1_700_000_000_123);

    assert_eq!(time.unix_millis(), 1_700_000_000_123);
}

pub const UTC_TIME_CONVERTS_TO_AND_FROM_SYSTEM_TIME: DescribedBehavior = DescribedBehavior::new(
    "utc time converts to and from system time",
    "UtcTime can convert to and from SystemTime on both sides of the Unix epoch.",
    utc_time_converts_to_and_from_system_time,
);

#[test]
fn utc_time_converts_to_and_from_system_time() {
    let after_epoch = std::time::UNIX_EPOCH + std::time::Duration::from_millis(1234);
    let before_epoch = std::time::UNIX_EPOCH - std::time::Duration::from_millis(1234);

    assert_eq!(
        UtcTime::from_system_time(after_epoch).unwrap(),
        UtcTime::from_unix_millis(1234)
    );
    assert_eq!(
        UtcTime::from_system_time(before_epoch).unwrap(),
        UtcTime::from_unix_millis(-1234)
    );
    assert_eq!(
        UtcTime::from_unix_millis(1234).to_system_time().unwrap(),
        after_epoch
    );
    assert_eq!(
        UtcTime::from_unix_millis(-1234).to_system_time().unwrap(),
        before_epoch
    );
}

pub const UTC_TIME_SUPPORTS_CHECKED_MILLISECOND_ARITHMETIC: DescribedBehavior =
    DescribedBehavior::new(
        "utc time supports checked millisecond arithmetic",
        "UtcTime exposes checked millisecond arithmetic so overflows are explicit.",
        utc_time_supports_checked_millisecond_arithmetic,
    );

#[test]
fn utc_time_supports_checked_millisecond_arithmetic() {
    let time = UtcTime::from_unix_millis(10);

    assert_eq!(
        time.checked_add_millis(5),
        Some(UtcTime::from_unix_millis(15))
    );
    assert_eq!(
        time.checked_sub_millis(15),
        Some(UtcTime::from_unix_millis(-5))
    );
    assert!(
        UtcTime::from_unix_millis(i64::MAX)
            .checked_add_millis(1)
            .is_none()
    );
}

pub const EVALUATION_TIME_PAIRS_UTC_AND_CALENDAR_TIME: DescribedBehavior = DescribedBehavior::new(
    "evaluation time pairs utc and calendar time",
    "EvaluationTime carries the explicit UTC instant and local calendar interpretation used by time-dependent domain logic.",
    evaluation_time_pairs_utc_and_calendar_time,
);

#[test]
fn evaluation_time_pairs_utc_and_calendar_time() {
    let utc_time = UtcTime::from_unix_millis(1_779_452_977_000);
    let calendar_moment = CalendarMoment::new(
        LogicalDate::new(2026, 5, 22).unwrap(),
        LocalTimeOfDay::from_hms(12, 30, 0).unwrap(),
        TimeZone::parse("America/Los_Angeles").unwrap(),
    );
    let evaluation_time = EvaluationTime::new(utc_time, calendar_moment.clone());

    assert_eq!(evaluation_time.utc_time(), utc_time);
    assert_eq!(evaluation_time.calendar_moment(), &calendar_moment);
}

pub const TIME_ZONE_VALIDATES_IANA_SHAPED_NAMES: DescribedBehavior = DescribedBehavior::new(
    "time zone validates iana shaped names",
    "TimeZone stores an IANA/tzdb-style zone name without attempting to resolve rule data.",
    time_zone_validates_iana_shaped_names,
);

#[test]
fn time_zone_validates_iana_shaped_names() {
    let pacific = TimeZone::parse(" America/Los_Angeles ").unwrap();
    let utc = TimeZone::utc();

    assert_eq!(pacific.name(), "America/Los_Angeles");
    assert_eq!(utc.name(), "UTC");
    assert!(utc.is_utc());
    assert_eq!(
        TimeZone::parse("America//Los_Angeles"),
        Err(TimeError::InvalidTimeZone(
            "America//Los_Angeles".to_owned()
        ))
    );
    assert_eq!(TimeZone::parse(" "), Err(TimeError::EmptyTimeZone));
}
