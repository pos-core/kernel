# Catalog Item

A catalog item is the catalog definition of something that can be configured for ordering.

It owns its stable item ID and label, optional description and media, ordered variant dimensions, concrete variant matches, shared modifier definitions, and the policy used to price those modifiers. The catalog item itself does not own a price. Each variant match owns an explicit invariant price, and configuration uses the selected match's price directly.

Variant settings include `allow_free_variant`, which defaults to false. Unless the catalog item explicitly enables that setting, construction rejects any match whose explicit invariant price is zero. Negative match prices are always invalid.

Dimensions define both presentation order and selection-path order. A match contains one value from each dimension it traverses, cannot skip an earlier dimension, and may stop before later dimensions. Every authored match is a concrete selectable leaf. One match cannot be a strict subset of another because a path cannot be both complete and require another selection.

A catalog item may have no dimensions. It must then contain one empty match. That match represents the required concrete selection and supplies the simple item's explicit price without inventing an unnamed variant. An empty match is invalid when the item has dimensions.

The catalog item derives selection steps from its concrete matches rather than storing a separate path tree. Given a valid partial path, it returns the next ordered dimension and only the values that continue into a compatible concrete match. An exact concrete match has no next step.

The optional description is independent of the required item label. Catalog-item description and media remain independent of metadata authored on variant values or exact variant matches. The kernel does not inherit, merge, or choose between those scopes; clients decide presentation fallback or combination. Descriptions and media are not included in configuration or order snapshots.

With one match, that match is the effective default. With multiple matches, one may carry an explicit default marker. Otherwise configuration requires an explicit concrete variant selection.
