use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "schedule",
        title: "Schedule",
        description: "Described behavior tests for schedule limits, local windows, exclusions, and explicit time context.",
        definitions: vec![
            DefinitionLink::new("Schedule", "../src/primitives/schedule/schedule.md"),
            DefinitionLink::new(
                "Schedule context",
                "../src/primitives/schedule/schedule-context.md",
            ),
        ],
        cases: vec![
            EMPTY_SCHEDULE_IS_ALWAYS_SCHEDULED.report_case(),
            SCHEDULE_LIMITS_GATE_THE_WHOLE_SCHEDULE.report_case(),
            SCHEDULE_WINDOWS_REQUIRE_A_MATCHING_LOCAL_WINDOW.report_case(),
            SCHEDULE_EXCLUSIONS_OVERRIDE_WINDOWS.report_case(),
            SCHEDULE_WINDOWS_COMBINE_DATES_DAYS_AND_TIMES.report_case(),
            SCHEDULE_REJECTS_INVALID_UTC_RANGES.report_case(),
        ],
    }
}

pub const EMPTY_SCHEDULE_IS_ALWAYS_SCHEDULED: DescribedBehavior = DescribedBehavior::new(
    "empty schedule is always scheduled",
    "A schedule with no windows and an Always limit includes any explicit ScheduleContext.",
    empty_schedule_is_always_scheduled,
);

#[test]
fn empty_schedule_is_always_scheduled() {
    let schedule = Schedule::always();

    assert!(schedule.is_scheduled(&schedule_context(100, 2026, 5, 22, 12, 0, 0)));
}

pub const SCHEDULE_LIMITS_GATE_THE_WHOLE_SCHEDULE: DescribedBehavior = DescribedBehavior::new(
    "schedule limits gate the whole schedule",
    "ScheduleLimit applies to the UTC instant before local windows or exclusions are considered.",
    schedule_limits_gate_the_whole_schedule,
);

#[test]
fn schedule_limits_gate_the_whole_schedule() {
    let until = Schedule::always().with_limit(ScheduleLimit::Until(utc(200)));
    let after = Schedule::always().with_limit(ScheduleLimit::After(utc(200)));
    let between =
        Schedule::always().with_limit(ScheduleLimit::between(utc(100), utc(200)).unwrap());

    assert!(until.is_scheduled(&schedule_context(199, 2026, 5, 22, 12, 0, 0)));
    assert!(!until.is_scheduled(&schedule_context(200, 2026, 5, 22, 12, 0, 0)));
    assert!(!after.is_scheduled(&schedule_context(199, 2026, 5, 22, 12, 0, 0)));
    assert!(after.is_scheduled(&schedule_context(200, 2026, 5, 22, 12, 0, 0)));
    assert!(between.is_scheduled(&schedule_context(100, 2026, 5, 22, 12, 0, 0)));
    assert!(!between.is_scheduled(&schedule_context(200, 2026, 5, 22, 12, 0, 0)));
    assert!(!Schedule::never().is_scheduled(&schedule_context(150, 2026, 5, 22, 12, 0, 0)));
}

pub const SCHEDULE_WINDOWS_REQUIRE_A_MATCHING_LOCAL_WINDOW: DescribedBehavior =
    DescribedBehavior::new(
        "schedule windows require a matching local window",
        "When windows exist, at least one local calendar window must match the supplied CalendarMoment.",
        schedule_windows_require_a_matching_local_window,
    );

#[test]
fn schedule_windows_require_a_matching_local_window() {
    let lunch = ScheduleWindow::new()
        .with_days_of_week(DaysOfWeek::weekdays())
        .with_time_range(LocalTimeRange::from_seconds(11 * 3_600, 14 * 3_600).unwrap());
    let schedule = Schedule::with_windows(vec![lunch]).unwrap();

    assert!(schedule.is_scheduled(&schedule_context(0, 2026, 5, 22, 12, 0, 0)));
    assert!(!schedule.is_scheduled(&schedule_context(0, 2026, 5, 22, 15, 0, 0)));
    assert!(!schedule.is_scheduled(&schedule_context(0, 2026, 5, 23, 12, 0, 0)));
}

pub const SCHEDULE_EXCLUSIONS_OVERRIDE_WINDOWS: DescribedBehavior = DescribedBehavior::new(
    "schedule exclusions override windows",
    "A matching exclusion makes the schedule unavailable even when the base schedule would include the moment.",
    schedule_exclusions_override_windows,
);

#[test]
fn schedule_exclusions_override_windows() {
    let holiday = ScheduleWindow::new().with_date_range(
        LogicalDateRange::new(
            LogicalDate::new(2026, 12, 25).unwrap(),
            LogicalDate::new(2026, 12, 25).unwrap(),
        )
        .unwrap(),
    );
    let schedule = Schedule::always().with_exclusion(holiday);

    assert!(!schedule.is_scheduled(&schedule_context(0, 2026, 12, 25, 12, 0, 0)));
    assert!(schedule.is_scheduled(&schedule_context(0, 2026, 12, 26, 12, 0, 0)));
}

pub const SCHEDULE_WINDOWS_COMBINE_DATES_DAYS_AND_TIMES: DescribedBehavior = DescribedBehavior::new(
    "schedule windows combine dates days and times",
    "A schedule window includes a moment only when its date range, day of week, and local time range all match.",
    schedule_windows_combine_dates_days_and_times,
);

#[test]
fn schedule_windows_combine_dates_days_and_times() {
    let may_friday_lunch = ScheduleWindow::new()
        .with_date_range(
            LogicalDateRange::new(
                LogicalDate::new(2026, 5, 1).unwrap(),
                LogicalDate::new(2026, 5, 31).unwrap(),
            )
            .unwrap(),
        )
        .with_days_of_week(DaysOfWeek::new([DayOfWeek::Friday]).unwrap())
        .with_time_range(LocalTimeRange::from_seconds(11 * 3_600, 14 * 3_600).unwrap());
    let schedule = Schedule::with_windows(vec![may_friday_lunch]).unwrap();

    assert!(schedule.is_scheduled(&schedule_context(0, 2026, 5, 22, 12, 0, 0)));
    assert!(!schedule.is_scheduled(&schedule_context(0, 2026, 5, 22, 15, 0, 0)));
    assert!(!schedule.is_scheduled(&schedule_context(0, 2026, 6, 5, 12, 0, 0)));
}

pub const SCHEDULE_REJECTS_INVALID_UTC_RANGES: DescribedBehavior = DescribedBehavior::new(
    "schedule rejects invalid utc ranges",
    "UTC time ranges are half open and must have a start strictly before their end.",
    schedule_rejects_invalid_utc_ranges,
);

#[test]
fn schedule_rejects_invalid_utc_ranges() {
    assert_eq!(
        UtcTimeRange::new(utc(200), utc(200)),
        Err(ScheduleError::InvalidUtcTimeRange {
            starts_at: utc(200),
            ends_at: utc(200)
        })
    );
}
