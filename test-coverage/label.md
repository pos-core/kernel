# Label

Described behavior tests for consumer profiles and label value resolution.

## Definitions

- [Label](../src/primitives/label/label.md)
- [Consumer profile](../src/primitives/consumer/consumer-profile.md)

## Result

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 7
- Passed: 7
- Failed: 0

## Behaviors

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| consumer profile preserves attribute precedence while matching requirements | A consumer profile preserves authored attribute order for precedence while satisfying requirements by attribute membership. | Passed | 0 ms |
| consumer profile rejects duplicate attributes | ConsumerProfile owns the uniqueness contract and rejects duplicate ConsumerAttribute IDs at construction time. | Passed | 0 ms |
| consumer profile rejects duplicate attributes added later | ConsumerProfile preserves attribute uniqueness when a caller adds attributes after construction. | Passed | 0 ms |
| label resolves most specific consumer profile value | A label chooses the matching value with the largest required consumer profile and preserves the label ID. | Passed | 0 ms |
| label falls back to default | A label uses its default value when no profile-specific value matches the active consumer profile. | Passed | 0 ms |
| label can exist without id | A manual or custom label can have no label ID while preserving the same default text and resolution behavior. | Passed | 0 ms |
| label uses consumer profile order for equal specificity | When equally specific label values match, the value containing the earliest differing attribute in the active consumer profile wins. | Passed | 0 ms |
