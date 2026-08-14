# Catalog Item

Described behavior tests for variants, shared item modifiers, variant applicability, and configured item pricing.

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 8
- Passed: 8
- Failed: 0

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| catalog item rejects empty titles and missing variants | Catalog item construction requires a non-empty title, non-empty variant titles, and at least one resolved variant. | Passed | 0 ms |
| catalog item rejects duplicate variant IDs and currency mismatches | A catalog item cannot define duplicate variants and all variant invariant prices must share one currency. | Passed | 0 ms |
| catalog item configures known variant and prices shared modifiers | Configuring a known variant returns invariant price, hydrated shared modifiers, modifier price contributions, and total price. | Passed | 0 ms |
| catalog item rejects unknown variant | Invalid variant combinations are modeled by absence; configuring a missing variant fails. | Passed | 0 ms |
| selected variant controls modifier choice applicability | A selected variant can make a shared modifier choice inapplicable without duplicating the modifier tree. | Passed | 0 ms |
| selected variant controls modifier prompt applicability | A selected variant can make a shared modifier prompt inapplicable. | Passed | 0 ms |
| item with no modifier prompts accepts empty selection and rejects unknown prompt selection | An item with no shared modifiers hydrates empty selections and rejects unexpected prompt selections. | Passed | 0 ms |
| resolved variant combinations are modeled by variant existence | The core sees only concrete valid variants; unsupported combinations are unknown variant IDs. | Passed | 0 ms |
