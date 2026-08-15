# Order Item

Described behavior tests for order-owned catalog facts, modifier snapshots, and entry expansion.

## Definitions

- [Order item](../src/order_item/order-item.md)
- [Order-item modifier snapshot](../src/order_item/order-item-modifier-snapshot.md)
- [Choice inputs](../src/modifier/choice-inputs.md)
- [Order entry](../src/entry/order-entry.md)

## Result

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 5
- Passed: 5
- Failed: 0

## Behaviors

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| catalog-backed order item preserves configured catalog facts | A catalog-backed order item preserves item, exact match, and component variant labels, effects, entered choice inputs, its order-item modifier snapshot, unit prices, and total price. | Passed | 0 ms |
| empty variant match does not duplicate the item description | A catalog-backed order item preserves the empty concrete match and its price while rendering only the catalog item label when the item has no dimensions. | Passed | 0 ms |
| unconnected order item supports none ids down to modifiers | Manual order items can preserve labels, prompts, choices, entered inputs, and modifier price contributions without catalog IDs. | Passed | 0 ms |
| order item expands to base and modifier entries | A catalog-backed order item expands into one base item entry and one entry for each priced modifier contribution. | Passed | 0 ms |
| order item rejects zero quantity and wrong modifier entry id count | Order item construction rejects zero quantity and entry expansion requires one modifier entry ID per priced contribution. | Passed | 0 ms |
