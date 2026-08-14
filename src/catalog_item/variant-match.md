# Variant Match

A variant match is one concrete selectable variant path with a required invariant price. It may also own an optional exact label, optional description label, and optional media collection.

Authors may supply its variant IDs in any order, but the catalog item stores them in dimension order after validating them. A match cannot contain two values from the same dimension, skip an earlier dimension, or be empty when dimensions exist. It may stop before later dimensions.

Every match is a concrete leaf. One match cannot be a strict subset of another because that would make the shorter path both complete and incomplete. The set of concrete matches implicitly defines the valid path tree and the values available after any partial selection.

Prices do not inherit between matches. Configuration uses only the selected match's own explicit price. Repeating the same amount across several matches is intentional and visible rather than a hidden pricing rule. An explicit zero amount means free and is valid only when the catalog item's `allow_free_variant` setting is enabled.

Effects and modifier applicability also belong directly to the concrete match and do not inherit. Any one match may carry the optional explicit default marker.

When an exact match label exists, configuration uses it as the display label. Otherwise the display label is derived by joining the selected variant labels in dimension order. Component labels remain separately available in either case.

Match description and media are independent of catalog-item and variant-value metadata. The kernel does not inherit or merge presentation metadata across those scopes, and descriptions and media do not enter configuration or order snapshots.
