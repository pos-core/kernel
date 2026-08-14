# Calendar

Described behavior tests for local calendar primitives used by schedules and availability.

## Definitions

- [Logical date](../src/primitives/calendar/logical-date.md)
- [Calendar moment](../src/primitives/calendar/calendar-moment.md)

## Result

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 5
- Passed: 5
- Failed: 0

## Behaviors

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| logical dates validate and derive weekdays | LogicalDate validates calendar days including leap years and deterministically derives DayOfWeek. | Passed | 0 ms |
| local time ranges are half open | LocalTimeRange includes its start, excludes its end, and rejects ranges that cross midnight. | Passed | 0 ms |
| logical date ranges are inclusive | LogicalDateRange includes both endpoints and rejects reversed ranges. | Passed | 0 ms |
| calendar moments preserve local interpretation | CalendarMoment pairs local date and time with a TimeZone without reading the system clock. | Passed | 0 ms |
| days of week require at least one day | DaysOfWeek rejects an empty set and provides deterministic weekday membership. | Passed | 0 ms |
