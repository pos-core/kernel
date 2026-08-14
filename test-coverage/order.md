# Order

Described behavior tests for order events, replay, entry sources, and derived totals.

## Definitions

- [Order](../src/order/order.md)
- [Event envelope](../src/event/event-envelope.md)

## Result

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 1
- Passed: 1
- Failed: 0

## Behaviors

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| order events replay to the same state and totals | Applying order events incrementally and replaying their envelopes produce the same order state and reproducible amount-due total. | Passed | 0 ms |
