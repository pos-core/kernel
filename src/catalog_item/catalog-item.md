# Catalog Item

A catalog item is the catalog definition of something that can be configured for ordering.

It owns its stable item ID and label, an optional description label, an optional media collection, one or more variants, shared modifier definitions, and the policy used to price those modifiers. The catalog item itself does not own a price; each variant supplies the invariant price used during configuration and may carry its own optional media collection.

The description is independent of the required item label. Its absence means the item has no authored description. It is catalog presentation metadata and is not included in configuration or order snapshots.

Catalog-item media is optional: an item with no media owns an empty collection. Item media is independent of variant media, and both collections may preserve multiple authored definitions. The kernel exposes them separately; clients decide precedence, combination, and fallback. Media is not included in configuration or order snapshots by default.

A catalog item with one variant always treats that variant as its effective default, without requiring a marker. With multiple variants, one variant may mark itself as the optional explicit default. The catalog item accepts at most one marker; because it lives on the variant, removing that variant cannot leave a dangling default reference.

Configuring a catalog item means resolving an existing variant and hydrating its applicable modifier selections. An explicit variant wins. Without one, configuration uses the effective default: the sole variant or the explicitly marked default among multiple variants. Multiple unmarked variants require an explicit selection.
