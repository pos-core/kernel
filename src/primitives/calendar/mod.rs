use std::collections::BTreeSet;
use std::fmt;

use crate::primitives::time::TimeZone;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LogicalDate {
    year: i32,
    month: u8,
    day: u8,
}

impl LogicalDate {
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, CalendarError> {
        if !(1..=12).contains(&month) {
            return Err(CalendarError::InvalidMonth(month));
        }

        let max_day = days_in_month(year, month);

        if day == 0 || day > max_day {
            return Err(CalendarError::InvalidDay { year, month, day });
        }

        Ok(Self { year, month, day })
    }

    pub fn year(self) -> i32 {
        self.year
    }

    pub fn month(self) -> u8 {
        self.month
    }

    pub fn day(self) -> u8 {
        self.day
    }

    pub fn day_of_week(self) -> DayOfWeek {
        let days = days_from_civil(self.year, self.month, self.day);
        DayOfWeek::from_days_since_unix_epoch(days)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl DayOfWeek {
    pub fn monday_index(self) -> u8 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }

    pub fn from_monday_index(index: u8) -> Result<Self, CalendarError> {
        match index {
            0 => Ok(Self::Monday),
            1 => Ok(Self::Tuesday),
            2 => Ok(Self::Wednesday),
            3 => Ok(Self::Thursday),
            4 => Ok(Self::Friday),
            5 => Ok(Self::Saturday),
            6 => Ok(Self::Sunday),
            _ => Err(CalendarError::InvalidDayOfWeek(index)),
        }
    }

    fn from_days_since_unix_epoch(days: i64) -> Self {
        let index = (days + 3).rem_euclid(7) as u8;
        Self::from_monday_index(index).expect("rem_euclid(7) always returns 0..=6")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DaysOfWeek {
    days: BTreeSet<DayOfWeek>,
}

impl DaysOfWeek {
    pub fn new(days: impl IntoIterator<Item = DayOfWeek>) -> Result<Self, CalendarError> {
        let days: BTreeSet<_> = days.into_iter().collect();

        if days.is_empty() {
            return Err(CalendarError::EmptyDaysOfWeek);
        }

        Ok(Self { days })
    }

    pub fn all() -> Self {
        Self {
            days: [
                DayOfWeek::Monday,
                DayOfWeek::Tuesday,
                DayOfWeek::Wednesday,
                DayOfWeek::Thursday,
                DayOfWeek::Friday,
                DayOfWeek::Saturday,
                DayOfWeek::Sunday,
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn weekdays() -> Self {
        Self {
            days: [
                DayOfWeek::Monday,
                DayOfWeek::Tuesday,
                DayOfWeek::Wednesday,
                DayOfWeek::Thursday,
                DayOfWeek::Friday,
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn weekends() -> Self {
        Self {
            days: [DayOfWeek::Saturday, DayOfWeek::Sunday]
                .into_iter()
                .collect(),
        }
    }

    pub fn contains(&self, day: DayOfWeek) -> bool {
        self.days.contains(&day)
    }

    pub fn days(&self) -> &BTreeSet<DayOfWeek> {
        &self.days
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LocalTimeOfDay {
    second_of_day: u32,
}

impl LocalTimeOfDay {
    pub const MAX_SECOND_OF_DAY: u32 = 86_399;

    pub fn from_hms(hour: u8, minute: u8, second: u8) -> Result<Self, CalendarError> {
        if hour > 23 {
            return Err(CalendarError::InvalidHour(hour));
        }

        if minute > 59 {
            return Err(CalendarError::InvalidMinute(minute));
        }

        if second > 59 {
            return Err(CalendarError::InvalidSecond(second));
        }

        Ok(Self {
            second_of_day: u32::from(hour) * 3_600 + u32::from(minute) * 60 + u32::from(second),
        })
    }

    pub fn from_seconds_since_midnight(second_of_day: u32) -> Result<Self, CalendarError> {
        if second_of_day > Self::MAX_SECOND_OF_DAY {
            return Err(CalendarError::InvalidSecondOfDay(second_of_day));
        }

        Ok(Self { second_of_day })
    }

    pub fn second_of_day(self) -> u32 {
        self.second_of_day
    }

    pub fn hour(self) -> u8 {
        (self.second_of_day / 3_600) as u8
    }

    pub fn minute(self) -> u8 {
        (self.second_of_day % 3_600 / 60) as u8
    }

    pub fn second(self) -> u8 {
        (self.second_of_day % 60) as u8
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LocalTimeRange {
    start_second: u32,
    end_second: u32,
}

impl LocalTimeRange {
    pub const SECONDS_PER_DAY: u32 = 86_400;

    pub fn new(start: LocalTimeOfDay, end: LocalTimeOfDay) -> Result<Self, CalendarError> {
        Self::from_seconds(start.second_of_day(), end.second_of_day())
    }

    pub fn from_seconds(start_second: u32, end_second: u32) -> Result<Self, CalendarError> {
        if start_second >= Self::SECONDS_PER_DAY {
            return Err(CalendarError::InvalidRangeStartSecond(start_second));
        }

        if end_second > Self::SECONDS_PER_DAY {
            return Err(CalendarError::InvalidRangeEndSecond(end_second));
        }

        if start_second >= end_second {
            return Err(CalendarError::InvalidTimeRange {
                start_second,
                end_second,
            });
        }

        Ok(Self {
            start_second,
            end_second,
        })
    }

    pub fn all_day() -> Self {
        Self {
            start_second: 0,
            end_second: Self::SECONDS_PER_DAY,
        }
    }

    pub fn contains(self, time: LocalTimeOfDay) -> bool {
        self.start_second <= time.second_of_day() && time.second_of_day() < self.end_second
    }

    pub fn start_second(self) -> u32 {
        self.start_second
    }

    pub fn end_second(self) -> u32 {
        self.end_second
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LogicalDateRange {
    start: LogicalDate,
    end: LogicalDate,
}

impl LogicalDateRange {
    pub fn new(start: LogicalDate, end: LogicalDate) -> Result<Self, CalendarError> {
        if start > end {
            return Err(CalendarError::InvalidDateRange { start, end });
        }

        Ok(Self { start, end })
    }

    pub fn contains(self, date: LogicalDate) -> bool {
        self.start <= date && date <= self.end
    }

    pub fn start(self) -> LogicalDate {
        self.start
    }

    pub fn end(self) -> LogicalDate {
        self.end
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CalendarMoment {
    date: LogicalDate,
    day_of_week: DayOfWeek,
    time_of_day: LocalTimeOfDay,
    time_zone: TimeZone,
}

impl CalendarMoment {
    pub fn new(date: LogicalDate, time_of_day: LocalTimeOfDay, time_zone: TimeZone) -> Self {
        Self {
            date,
            day_of_week: date.day_of_week(),
            time_of_day,
            time_zone,
        }
    }

    pub fn date(&self) -> LogicalDate {
        self.date
    }

    pub fn day_of_week(&self) -> DayOfWeek {
        self.day_of_week
    }

    pub fn time_of_day(&self) -> LocalTimeOfDay {
        self.time_of_day
    }

    pub fn time_zone(&self) -> &TimeZone {
        &self.time_zone
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CalendarError {
    InvalidMonth(u8),
    InvalidDay {
        year: i32,
        month: u8,
        day: u8,
    },
    InvalidDayOfWeek(u8),
    EmptyDaysOfWeek,
    InvalidHour(u8),
    InvalidMinute(u8),
    InvalidSecond(u8),
    InvalidSecondOfDay(u32),
    InvalidRangeStartSecond(u32),
    InvalidRangeEndSecond(u32),
    InvalidTimeRange {
        start_second: u32,
        end_second: u32,
    },
    InvalidDateRange {
        start: LogicalDate,
        end: LogicalDate,
    },
}

impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMonth(month) => write!(f, "invalid calendar month `{month}`"),
            Self::InvalidDay { year, month, day } => {
                write!(f, "invalid logical date `{year:04}-{month:02}-{day:02}`")
            }
            Self::InvalidDayOfWeek(index) => {
                write!(f, "invalid day-of-week index `{index}`")
            }
            Self::EmptyDaysOfWeek => f.write_str("days-of-week set cannot be empty"),
            Self::InvalidHour(hour) => write!(f, "invalid local hour `{hour}`"),
            Self::InvalidMinute(minute) => write!(f, "invalid local minute `{minute}`"),
            Self::InvalidSecond(second) => write!(f, "invalid local second `{second}`"),
            Self::InvalidSecondOfDay(second) => {
                write!(f, "invalid second-of-day `{second}`")
            }
            Self::InvalidRangeStartSecond(second) => {
                write!(f, "invalid range start second `{second}`")
            }
            Self::InvalidRangeEndSecond(second) => {
                write!(f, "invalid range end second `{second}`")
            }
            Self::InvalidTimeRange {
                start_second,
                end_second,
            } => write!(
                f,
                "invalid local time range `{start_second}`..`{end_second}`"
            ),
            Self::InvalidDateRange { start, end } => write!(
                f,
                "invalid logical date range `{:04}-{:02}-{:02}`..`{:04}-{:02}-{:02}`",
                start.year(),
                start.month(),
                start.day(),
                end.year(),
                end.month(),
                end.day()
            ),
        }
    }
}

impl std::error::Error for CalendarError {}

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = i64::from(year) - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day = i64::from(day);
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::{
        CalendarError, CalendarMoment, DayOfWeek, DaysOfWeek, LocalTimeOfDay, LocalTimeRange,
        LogicalDate, LogicalDateRange,
    };
    use crate::primitives::time::TimeZone;

    #[test]
    fn logical_dates_validate_months_days_and_leap_years() {
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
            LogicalDate::new(2026, 13, 1),
            Err(CalendarError::InvalidMonth(13))
        );
    }

    #[test]
    fn logical_dates_derive_day_of_week() {
        let date = LogicalDate::new(2026, 5, 22).unwrap();

        assert_eq!(date.day_of_week(), DayOfWeek::Friday);
    }

    #[test]
    fn local_time_ranges_are_half_open_and_do_not_cross_midnight() {
        let nine = LocalTimeOfDay::from_hms(9, 0, 0).unwrap();
        let five = LocalTimeOfDay::from_hms(17, 0, 0).unwrap();
        let range = LocalTimeRange::new(nine, five).unwrap();

        assert!(range.contains(nine));
        assert!(!range.contains(five));
        assert_eq!(
            LocalTimeRange::new(five, nine),
            Err(CalendarError::InvalidTimeRange {
                start_second: 61_200,
                end_second: 32_400
            })
        );
    }

    #[test]
    fn date_ranges_are_inclusive() {
        let start = LogicalDate::new(2026, 5, 1).unwrap();
        let end = LogicalDate::new(2026, 5, 31).unwrap();
        let range = LogicalDateRange::new(start, end).unwrap();

        assert!(range.contains(start));
        assert!(range.contains(end));
        assert!(!range.contains(LogicalDate::new(2026, 6, 1).unwrap()));
    }

    #[test]
    fn calendar_moment_pairs_local_values_with_a_time_zone() {
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

    #[test]
    fn days_of_week_requires_at_least_one_day() {
        assert_eq!(DaysOfWeek::new([]), Err(CalendarError::EmptyDaysOfWeek));
        assert!(DaysOfWeek::weekdays().contains(DayOfWeek::Monday));
        assert!(!DaysOfWeek::weekdays().contains(DayOfWeek::Sunday));
    }
}
