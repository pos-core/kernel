# Calendar Moment

A calendar moment is an explicit local interpretation: a logical date, derived day of week, local time of day, and time-zone name.

It does not calculate a UTC instant or read the system clock. Callers provide the local values and the time-zone identity whose interpretation they intend.

When domain logic needs both a UTC instant and this local interpretation, they are paired by an evaluation time or schedule context.
