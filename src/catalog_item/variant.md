# Variant

A variant is a concrete, selectable form of a catalog item.

It owns a stable variant ID, an invariant price, effects, and the applicability rules that determine which shared modifiers are available for that form. Its ID represents a valid combination already resolved by the catalog rather than a set of option dimensions that the kernel must combine.

A catalog item with exactly one variant may leave its label absent because there is no user-facing distinction to select. When multiple variants exist, every variant must have a label.
