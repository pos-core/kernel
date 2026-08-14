# Logical Date

A logical date is a calendar year, month, and day without a time of day, time zone, or UTC instant.

Construction validates the actual number of days in the month, including leap years. The date can derive its day of the week deterministically without consulting the system clock.

Use a logical date when the domain means a calendar day rather than a moment on the global timeline.
