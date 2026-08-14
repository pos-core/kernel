# Schedule

Described behavior tests for schedule limits, local windows, exclusions, and explicit time context.

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 5
- Passed: 5
- Failed: 0

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| empty schedule is always scheduled | A schedule with no windows and an Always limit includes any explicit ScheduleContext. | Passed | 0 ms |
| schedule limits gate the whole schedule | ScheduleLimit applies to the UTC instant before local windows or exclusions are considered. | Passed | 0 ms |
| schedule windows require a matching local window | When windows exist, at least one local calendar window must match the supplied CalendarMoment. | Passed | 0 ms |
| schedule exclusions override windows | A matching exclusion makes the schedule unavailable even when the base schedule would include the moment. | Passed | 0 ms |
| schedule rejects invalid utc ranges | UTC time ranges are half open and must have a start strictly before their end. | Passed | 0 ms |
