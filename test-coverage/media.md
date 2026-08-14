# Media

Described behavior tests for media collections, MIME metadata, dimensions, and consumer-profile variants.

## Definitions

- [Media](../src/primitives/media/media.md)

## Result

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 5
- Passed: 5
- Failed: 0

## Behaviors

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| media collection rejects duplicate defaults | A MediaCollection preserves definition order but rejects duplicate default Media IDs. | Passed | 0 ms |
| media resolves most specific consumer profile variant | Media always has a default and may resolve to the most specific matching consumer-profile variant. | Passed | 0 ms |
| media falls back to default | Media resolves to its default representation when no consumer-profile variant matches. | Passed | 0 ms |
| media rejects ambiguous equal specificity variants | Media resolution rejects equally specific matching variants instead of relying on definition order. | Passed | 0 ms |
| media validates mime types and dimensions | Media MIME types are normalized and dimensions must be nonzero when provided. | Passed | 0 ms |
