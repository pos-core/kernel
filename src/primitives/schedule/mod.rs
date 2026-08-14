use std::fmt;

use crate::primitives::calendar::{CalendarMoment, DaysOfWeek, LocalTimeRange, LogicalDateRange};
use crate::primitives::time::{EvaluationTime, UtcTime};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Schedule {
    limit: ScheduleLimit,
    windows: Vec<ScheduleWindow>,
    exclusions: Vec<ScheduleWindow>,
}

impl Schedule {
    pub fn always() -> Self {
        Self {
            limit: ScheduleLimit::Always,
            windows: Vec::new(),
            exclusions: Vec::new(),
        }
    }

    pub fn never() -> Self {
        Self {
            limit: ScheduleLimit::Never,
            windows: Vec::new(),
            exclusions: Vec::new(),
        }
    }

    pub fn with_windows(windows: Vec<ScheduleWindow>) -> Result<Self, ScheduleError> {
        if windows.is_empty() {
            return Err(ScheduleError::EmptyWindows);
        }

        Ok(Self {
            limit: ScheduleLimit::Always,
            windows,
            exclusions: Vec::new(),
        })
    }

    pub fn with_limit(mut self, limit: ScheduleLimit) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_exclusion(mut self, exclusion: ScheduleWindow) -> Self {
        self.exclusions.push(exclusion);
        self
    }

    pub fn with_exclusions(mut self, exclusions: Vec<ScheduleWindow>) -> Self {
        self.exclusions = exclusions;
        self
    }

    pub fn is_scheduled(&self, context: &ScheduleContext) -> bool {
        if !self.limit.allows(context.utc_time()) {
            return false;
        }

        if self
            .exclusions
            .iter()
            .any(|exclusion| exclusion.matches(context.moment()))
        {
            return false;
        }

        if self.windows.is_empty() {
            return true;
        }

        self.windows
            .iter()
            .any(|window| window.matches(context.moment()))
    }

    pub fn is_scheduled_at(&self, evaluation_time: &EvaluationTime) -> bool {
        self.is_scheduled(&ScheduleContext::from_evaluation_time(evaluation_time))
    }

    pub fn includes(&self, context: &ScheduleContext) -> bool {
        self.is_scheduled(context)
    }

    pub fn limit(&self) -> &ScheduleLimit {
        &self.limit
    }

    pub fn windows(&self) -> &[ScheduleWindow] {
        &self.windows
    }

    pub fn exclusions(&self) -> &[ScheduleWindow] {
        &self.exclusions
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::always()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ScheduleLimit {
    Always,
    Never,
    Until(UtcTime),
    After(UtcTime),
    Between(UtcTimeRange),
}

impl ScheduleLimit {
    pub fn between(starts_at: UtcTime, ends_at: UtcTime) -> Result<Self, ScheduleError> {
        Ok(Self::Between(UtcTimeRange::new(starts_at, ends_at)?))
    }

    pub fn allows(&self, utc_time: UtcTime) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Until(ends_at) => utc_time < *ends_at,
            Self::After(starts_at) => *starts_at <= utc_time,
            Self::Between(range) => range.contains(utc_time),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UtcTimeRange {
    starts_at: UtcTime,
    ends_at: UtcTime,
}

impl UtcTimeRange {
    pub fn new(starts_at: UtcTime, ends_at: UtcTime) -> Result<Self, ScheduleError> {
        if starts_at >= ends_at {
            return Err(ScheduleError::InvalidUtcTimeRange { starts_at, ends_at });
        }

        Ok(Self { starts_at, ends_at })
    }

    pub fn contains(self, utc_time: UtcTime) -> bool {
        self.starts_at <= utc_time && utc_time < self.ends_at
    }

    pub fn starts_at(self) -> UtcTime {
        self.starts_at
    }

    pub fn ends_at(self) -> UtcTime {
        self.ends_at
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ScheduleWindow {
    date_range: Option<LogicalDateRange>,
    days_of_week: Option<DaysOfWeek>,
    time_range: Option<LocalTimeRange>,
}

impl ScheduleWindow {
    pub fn always() -> Self {
        Self::default()
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_date_range(mut self, date_range: LogicalDateRange) -> Self {
        self.date_range = Some(date_range);
        self
    }

    pub fn with_days_of_week(mut self, days_of_week: DaysOfWeek) -> Self {
        self.days_of_week = Some(days_of_week);
        self
    }

    pub fn with_time_range(mut self, time_range: LocalTimeRange) -> Self {
        self.time_range = Some(time_range);
        self
    }

    pub fn matches(&self, moment: &CalendarMoment) -> bool {
        if let Some(date_range) = self.date_range
            && !date_range.contains(moment.date())
        {
            return false;
        }

        if let Some(days_of_week) = &self.days_of_week
            && !days_of_week.contains(moment.day_of_week())
        {
            return false;
        }

        if let Some(time_range) = self.time_range
            && !time_range.contains(moment.time_of_day())
        {
            return false;
        }

        true
    }

    pub fn date_range(&self) -> Option<LogicalDateRange> {
        self.date_range
    }

    pub fn days_of_week(&self) -> Option<&DaysOfWeek> {
        self.days_of_week.as_ref()
    }

    pub fn time_range(&self) -> Option<LocalTimeRange> {
        self.time_range
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScheduleContext {
    utc_time: UtcTime,
    moment: CalendarMoment,
}

impl ScheduleContext {
    pub fn new(utc_time: UtcTime, moment: CalendarMoment) -> Self {
        Self { utc_time, moment }
    }

    pub fn from_evaluation_time(evaluation_time: &EvaluationTime) -> Self {
        Self {
            utc_time: evaluation_time.utc_time(),
            moment: evaluation_time.calendar_moment().clone(),
        }
    }

    pub fn utc_time(&self) -> UtcTime {
        self.utc_time
    }

    pub fn moment(&self) -> &CalendarMoment {
        &self.moment
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ScheduleError {
    EmptyWindows,
    InvalidUtcTimeRange {
        starts_at: UtcTime,
        ends_at: UtcTime,
    },
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyWindows => f.write_str("windowed schedule requires at least one window"),
            Self::InvalidUtcTimeRange { starts_at, ends_at } => write!(
                f,
                "invalid UTC time range `{}`..`{}`",
                starts_at.unix_millis(),
                ends_at.unix_millis()
            ),
        }
    }
}

impl std::error::Error for ScheduleError {}

#[cfg(test)]
mod tests {
    use super::{
        Schedule, ScheduleContext, ScheduleError, ScheduleLimit, ScheduleWindow, UtcTimeRange,
    };
    use crate::primitives::calendar::{
        CalendarMoment, DayOfWeek, DaysOfWeek, LocalTimeOfDay, LocalTimeRange, LogicalDate,
        LogicalDateRange,
    };
    use crate::primitives::time::{TimeZone, UtcTime};

    #[test]
    fn empty_schedule_is_always_scheduled() {
        let schedule = Schedule::always();

        assert!(schedule.includes(&context(100, 2026, 5, 22, 12, 0, 0)));
    }

    #[test]
    fn schedule_limits_gate_the_whole_schedule() {
        let until = Schedule::always().with_limit(ScheduleLimit::Until(utc(200)));
        let after = Schedule::always().with_limit(ScheduleLimit::After(utc(200)));
        let between =
            Schedule::always().with_limit(ScheduleLimit::between(utc(100), utc(200)).unwrap());

        assert!(until.includes(&context(199, 2026, 5, 22, 12, 0, 0)));
        assert!(!until.includes(&context(200, 2026, 5, 22, 12, 0, 0)));
        assert!(!after.includes(&context(199, 2026, 5, 22, 12, 0, 0)));
        assert!(after.includes(&context(200, 2026, 5, 22, 12, 0, 0)));
        assert!(between.includes(&context(100, 2026, 5, 22, 12, 0, 0)));
        assert!(!between.includes(&context(200, 2026, 5, 22, 12, 0, 0)));
        assert!(!Schedule::never().includes(&context(150, 2026, 5, 22, 12, 0, 0)));
    }

    #[test]
    fn schedule_windows_require_any_matching_window_when_present() {
        let breakfast = ScheduleWindow::new()
            .with_days_of_week(DaysOfWeek::weekdays())
            .with_time_range(
                LocalTimeRange::from_seconds(6 * 3_600, 10 * 3_600 + 30 * 60).unwrap(),
            );
        let schedule = Schedule::with_windows(vec![breakfast]).unwrap();

        assert!(schedule.includes(&context(0, 2026, 5, 22, 9, 30, 0)));
        assert!(!schedule.includes(&context(0, 2026, 5, 22, 11, 0, 0)));
        assert!(!schedule.includes(&context(0, 2026, 5, 23, 9, 30, 0)));
    }

    #[test]
    fn schedule_exclusions_override_windows_and_always() {
        let holiday = ScheduleWindow::new().with_date_range(
            LogicalDateRange::new(
                LogicalDate::new(2026, 12, 25).unwrap(),
                LogicalDate::new(2026, 12, 25).unwrap(),
            )
            .unwrap(),
        );
        let schedule = Schedule::always().with_exclusion(holiday);

        assert!(!schedule.includes(&context(0, 2026, 12, 25, 12, 0, 0)));
        assert!(schedule.includes(&context(0, 2026, 12, 26, 12, 0, 0)));
    }

    #[test]
    fn schedule_windows_can_combine_dates_days_and_times() {
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

        assert!(schedule.includes(&context(0, 2026, 5, 22, 12, 0, 0)));
        assert!(!schedule.includes(&context(0, 2026, 5, 22, 15, 0, 0)));
        assert!(!schedule.includes(&context(0, 2026, 6, 5, 12, 0, 0)));
    }

    #[test]
    fn schedule_rejects_empty_windowed_schedules_and_invalid_utc_ranges() {
        assert_eq!(
            Schedule::with_windows(Vec::new()),
            Err(ScheduleError::EmptyWindows)
        );
        assert_eq!(
            UtcTimeRange::new(utc(200), utc(200)),
            Err(ScheduleError::InvalidUtcTimeRange {
                starts_at: utc(200),
                ends_at: utc(200)
            })
        );
    }

    fn context(
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

    fn utc(unix_millis: i64) -> UtcTime {
        UtcTime::from_unix_millis(unix_millis)
    }
}
