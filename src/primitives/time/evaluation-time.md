# Evaluation Time

Evaluation time is the explicit pair of a UTC instant and its intended local calendar interpretation.

Time-dependent domain logic receives this value instead of reading the system clock. The UTC side supports absolute limits; the calendar side supports local dates, days of week, times of day, and time-zone identity.

The type stores the supplied pair and does not perform time-zone conversion.
