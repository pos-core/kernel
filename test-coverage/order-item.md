# Order Item

Described behavior tests for catalog-backed order item snapshots and entry expansion.

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 4
- Passed: 4
- Failed: 0

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| order item snapshots configured catalog item | An order item preserves catalog item labels, variant labels, effects, modifier snapshot, unit prices, and total price. | Passed | 0 ms |
| order item expands to base and modifier entries | A catalog-backed order item expands into one base item entry and one entry for each priced modifier contribution. | Passed | 0 ms |
| order item rejects zero quantity and wrong modifier entry id count | Order item construction rejects zero quantity and entry expansion requires one modifier entry ID per priced contribution. | Passed | 0 ms |
| unconnected order item supports none ids down to modifiers | Manual order items can preserve labels, prompts, choices, and modifier price contributions without catalog IDs. | Passed | 0 ms |
