use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::ModuleReport;

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "calendar",
        title: "Calendar",
        description: "Described behavior tests for local calendar primitives used by schedules and availability.",
        cases: vec![
            case(
                "logical dates validate and derive weekdays",
                "LogicalDate validates calendar days including leap years and deterministically derives DayOfWeek.",
                logical_dates_validate_and_derive_weekdays,
            ),
            case(
                "local time ranges are half open",
                "LocalTimeRange includes its start, excludes its end, and rejects ranges that cross midnight.",
                local_time_ranges_are_half_open,
            ),
            case(
                "logical date ranges are inclusive",
                "LogicalDateRange includes both endpoints and rejects reversed ranges.",
                logical_date_ranges_are_inclusive,
            ),
            case(
                "calendar moments preserve local interpretation",
                "CalendarMoment pairs local date and time with a TimeZone without reading the system clock.",
                calendar_moments_preserve_local_interpretation,
            ),
        ],
    }
}

fn logical_dates_validate_and_derive_weekdays() {
    assert!(LogicalDate::new(2024, 2, 29).is_ok());
    assert_eq!(
        LogicalDate::new(2023, 2, 29),
        Err(CalendarError::InvalidDay {
            year: 2023,
            month: 2,
            day: 29
        })
    );
    assert_eq!(
        LogicalDate::new(2026, 5, 22).unwrap().day_of_week(),
        DayOfWeek::Friday
    );
}

fn local_time_ranges_are_half_open() {
    let start = LocalTimeOfDay::from_hms(9, 0, 0).unwrap();
    let end = LocalTimeOfDay::from_hms(17, 0, 0).unwrap();
    let range = LocalTimeRange::new(start, end).unwrap();

    assert!(range.contains(start));
    assert!(!range.contains(end));
    assert_eq!(
        LocalTimeRange::new(end, start),
        Err(CalendarError::InvalidTimeRange {
            start_second: 61_200,
            end_second: 32_400
        })
    );
}

fn logical_date_ranges_are_inclusive() {
    let start = LogicalDate::new(2026, 5, 1).unwrap();
    let end = LogicalDate::new(2026, 5, 31).unwrap();
    let range = LogicalDateRange::new(start, end).unwrap();

    assert!(range.contains(start));
    assert!(range.contains(end));
    assert!(!range.contains(LogicalDate::new(2026, 6, 1).unwrap()));
    assert_eq!(
        LogicalDateRange::new(end, start),
        Err(CalendarError::InvalidDateRange {
            start: end,
            end: start
        })
    );
}

fn calendar_moments_preserve_local_interpretation() {
    let date = LogicalDate::new(2026, 5, 22).unwrap();
    let moment = CalendarMoment::new(
        date,
        LocalTimeOfDay::from_hms(12, 30, 0).unwrap(),
        TimeZone::parse("America/Los_Angeles").unwrap(),
    );

    assert_eq!(moment.date(), date);
    assert_eq!(moment.day_of_week(), DayOfWeek::Friday);
    assert_eq!(moment.time_of_day().hour(), 12);
    assert_eq!(moment.time_zone().name(), "America/Los_Angeles");
}
