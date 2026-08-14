# Variant Match

A variant match is an unordered set of zero or more variant IDs with a required invariant price. It may also own an optional exact label, optional description label, and optional media collection.

The catalog item stores each match in dimension order after validating it. A match cannot contain two values from the same dimension. Match order supplied by an author therefore has no semantic effect.

A match is deepest when no other authored match is its strict superset. Deepest matches establish the concrete selections that exist in the catalog. This definition allows different authored paths to stop at different depths without requiring every dimension to participate.

Prices do not inherit between matches. Configuration uses only the selected deepest match's own explicit price. Repeating the same amount across several matches is intentional and visible rather than a hidden pricing rule. An explicit zero amount means free and is valid only when the catalog item's `allow_free_variant` setting is enabled.

Effects and modifier applicability also belong to the deepest match itself and do not inherit. A deepest match may carry the optional explicit default marker. A shallower match cannot be the default.

When an exact match label exists, configuration uses it as the display label. Otherwise the display label is derived by joining the selected variant labels in dimension order. Component labels remain separately available in either case.

Match description and media are independent of catalog-item and variant-value metadata. The kernel does not inherit or merge presentation metadata across those scopes, and descriptions and media do not enter configuration or order snapshots.
