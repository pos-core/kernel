# Catalog Item

A catalog item is the catalog definition of something that can be configured for ordering.

It owns its stable item ID and label, optional description and media, ordered variant dimensions, authored variant matches, shared modifier definitions, and the policy used to price those modifiers. The catalog item itself does not own a price. Each variant match owns an explicit invariant price, and configuration uses the selected deepest match's price directly.

Variant settings include `allow_free_variant`, which defaults to false. Unless the catalog item explicitly enables that setting, construction rejects any deepest match whose explicit invariant price is zero. Negative match prices are always invalid.

Dimensions define authored presentation and selection order, not a required hierarchy. A concrete selection does not have to contain a value from every dimension. A match is deepest when no other authored match contains all of its variants plus at least one more. Only deepest matches represent concrete selectable forms of the item.

A catalog item may have no dimensions. It must then contain one empty deepest match. That match represents the required concrete selection and supplies the simple item's explicit price without inventing an unnamed variant.

The optional description is independent of the required item label. Catalog-item description and media remain independent of metadata authored on variant values or exact variant matches. The kernel does not inherit, merge, or choose between those scopes; clients decide presentation fallback or combination. Descriptions and media are not included in configuration or order snapshots.

With one deepest match, that match is the effective default. With multiple deepest matches, one may carry an explicit default marker. Otherwise configuration requires an explicit concrete variant selection.
