# Schedule

A schedule answers whether supplied time context is included by a set of UTC and local-calendar constraints.

The UTC limit gates the whole schedule first. Matching exclusions then override availability. If local windows are defined, at least one must match the supplied local calendar moment; without windows, the schedule is included after the limit and exclusions pass.

Schedule evaluation is deterministic because callers provide the time context explicitly.
