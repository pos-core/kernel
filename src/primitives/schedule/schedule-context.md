# Schedule Context

A schedule context pairs the UTC instant used by absolute schedule limits with the calendar moment used by local windows and exclusions.

It makes both interpretations explicit and prevents schedule evaluation from reading the system clock. Callers are responsible for supplying a mutually meaningful UTC instant and local calendar interpretation.

An evaluation time can be converted directly into schedule context.
