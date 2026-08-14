# Time

Described behavior tests for UTC time and time zone primitives.

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 5
- Passed: 5
- Failed: 0

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| utc time stores unix milliseconds | UtcTime is a UTC instant represented by Unix milliseconds and does not own an ID. | Passed | 0 ms |
| utc time converts to and from system time | UtcTime can convert to and from SystemTime on both sides of the Unix epoch. | Passed | 0 ms |
| utc time supports checked millisecond arithmetic | UtcTime exposes checked millisecond arithmetic so overflows are explicit. | Passed | 0 ms |
| evaluation time pairs utc and calendar time | EvaluationTime carries the explicit UTC instant and local calendar interpretation used by time-dependent domain logic. | Passed | 0 ms |
| time zone validates iana shaped names | TimeZone stores an IANA/tzdb-style zone name without attempting to resolve rule data. | Passed | 0 ms |
