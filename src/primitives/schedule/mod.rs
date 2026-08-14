use std::fmt;

use crate::primitives::calendar::{CalendarMoment, DaysOfWeek, LocalTimeRange, LogicalDateRange};
use crate::primitives::time::{EvaluationTime, UtcTime};

#[doc = include_str!("schedule.md")]
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

#[doc = include_str!("schedule-context.md")]
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
