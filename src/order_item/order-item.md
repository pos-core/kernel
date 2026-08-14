# Order Item

An order item is an order-facing record of a configured or manually entered item.

It preserves its source, item and optional variant labels, quantity, invariant and modifier unit prices, total price, effects, and an order-owned modifier snapshot. Catalog-backed items retain the catalog version, item, and variant IDs; manual items do not require catalog IDs.

An order item can expand into one base order entry plus one entry for each priced modifier contribution.
