use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::primitives::calendar::CalendarMoment;

#[doc = include_str!("utc-time.md")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct UtcTime {
    unix_millis: i64,
}

impl UtcTime {
    pub const fn from_unix_millis(unix_millis: i64) -> Self {
        Self { unix_millis }
    }

    pub const fn unix_millis(self) -> i64 {
        self.unix_millis
    }

    pub fn from_system_time(system_time: SystemTime) -> Result<Self, TimeError> {
        match system_time.duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(Self::from_unix_millis(duration_to_positive_millis(
                duration,
            )?)),
            Err(error) => Ok(Self::from_unix_millis(duration_to_negative_millis(
                error.duration(),
            )?)),
        }
    }

    pub fn to_system_time(self) -> Result<SystemTime, TimeError> {
        if self.unix_millis >= 0 {
            UNIX_EPOCH
                .checked_add(Duration::from_millis(self.unix_millis as u64))
                .ok_or(TimeError::SystemTimeOverflow)
        } else {
            UNIX_EPOCH
                .checked_sub(Duration::from_millis(self.unix_millis.unsigned_abs()))
                .ok_or(TimeError::SystemTimeOverflow)
        }
    }

    pub fn checked_add_millis(self, millis: i64) -> Option<Self> {
        self.unix_millis
            .checked_add(millis)
            .map(Self::from_unix_millis)
    }

    pub fn checked_sub_millis(self, millis: i64) -> Option<Self> {
        self.unix_millis
            .checked_sub(millis)
            .map(Self::from_unix_millis)
    }
}

#[doc = include_str!("evaluation-time.md")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EvaluationTime {
    utc_time: UtcTime,
    calendar_moment: CalendarMoment,
}

impl EvaluationTime {
    pub fn new(utc_time: UtcTime, calendar_moment: CalendarMoment) -> Self {
        Self {
            utc_time,
            calendar_moment,
        }
    }

    pub fn utc_time(&self) -> UtcTime {
        self.utc_time
    }

    pub fn calendar_moment(&self) -> &CalendarMoment {
        &self.calendar_moment
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TimeZone {
    name: String,
}

impl TimeZone {
    pub fn utc() -> Self {
        Self {
            name: "UTC".to_owned(),
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, TimeError> {
        let name = value.as_ref().trim();

        validate_time_zone_name(name)?;

        Ok(Self {
            name: name.to_owned(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_utc(&self) -> bool {
        matches!(self.name.as_str(), "UTC" | "Etc/UTC")
    }
}

impl fmt::Display for TimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for TimeZone {
    type Err = TimeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for TimeZone {
    type Error = TimeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl AsRef<str> for TimeZone {
    fn as_ref(&self) -> &str {
        self.name()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TimeError {
    UnixMillisOverflow,
    SystemTimeOverflow,
    EmptyTimeZone,
    InvalidTimeZone(String),
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnixMillisOverflow => {
                f.write_str("UTC time exceeds supported Unix millisecond range")
            }
            Self::SystemTimeOverflow => f.write_str("UTC time cannot be represented as SystemTime"),
            Self::EmptyTimeZone => f.write_str("time zone name cannot be empty"),
            Self::InvalidTimeZone(value) => {
                write!(f, "invalid time zone name `{value}`")
            }
        }
    }
}

impl std::error::Error for TimeError {}

fn duration_to_positive_millis(duration: Duration) -> Result<i64, TimeError> {
    i64::try_from(duration.as_millis()).map_err(|_| TimeError::UnixMillisOverflow)
}

fn duration_to_negative_millis(duration: Duration) -> Result<i64, TimeError> {
    let millis = duration.as_millis();

    if millis == i64::MAX as u128 + 1 {
        return Ok(i64::MIN);
    }

    let millis = i64::try_from(millis).map_err(|_| TimeError::UnixMillisOverflow)?;

    millis.checked_neg().ok_or(TimeError::UnixMillisOverflow)
}

fn validate_time_zone_name(name: &str) -> Result<(), TimeError> {
    if name.is_empty() {
        return Err(TimeError::EmptyTimeZone);
    }

    if name.starts_with('/') || name.ends_with('/') || name.contains("//") {
        return Err(TimeError::InvalidTimeZone(name.to_owned()));
    }

    let valid = name.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-' | b'+' | b'.')
    });

    if valid {
        Ok(())
    } else {
        Err(TimeError::InvalidTimeZone(name.to_owned()))
    }
}
