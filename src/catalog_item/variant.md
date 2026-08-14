# Variant

A variant is a concrete, selectable form of a catalog item.

It owns a stable variant ID, an optional description label, an invariant price, effects, an optional media collection, and the applicability rules that determine which shared modifiers are available for that form. Its ID represents a valid combination already resolved by the catalog rather than a set of option dimensions that the kernel must combine.

The description is independent of the variant label and may be absent whether the variant itself is labeled or is the sole implicit unlabeled variant. It is catalog presentation metadata and is not included in configuration or order snapshots.

Variant media is optional: a variant with no media owns an empty collection. The collection may contain multiple media definitions and preserves their authored order. Media describes the catalog presentation and does not become part of configuration or order snapshots by default.

The kernel does not apply fallback between variant media and any broader catalog-item media. Choosing one collection, combining collections, or falling back between them is client presentation behavior.

A variant may mark itself as the catalog item's explicit default. The marker is optional, and the containing catalog item validates that no other variant is also marked default. A sole variant is always the catalog item's effective default even when it is not explicitly marked.

A catalog item with exactly one variant may leave its label absent because there is no user-facing distinction to select. When multiple variants exist, every variant must have a label.
