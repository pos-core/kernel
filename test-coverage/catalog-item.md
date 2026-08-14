# Catalog Item

Described behavior tests for variants, optional descriptions and media, defaults, shared item modifiers, applicability, and configured item pricing.

## Definitions

- [Catalog item](../src/catalog_item/catalog-item.md)
- [Variant](../src/catalog_item/variant.md)
- [Media](../src/primitives/media/media.md)
- [Configured catalog item](../src/catalog_item/configured-catalog-item.md)

## Result

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 14
- Passed: 14
- Failed: 0

## Behaviors

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| catalog item rejects empty titles and missing variants | Catalog item construction requires a non-empty title, non-empty text for labeled variants, and at least one resolved variant. | Passed | 0 ms |
| sole variant may be unlabeled but multiple variants require labels | A catalog item may use one unlabeled priced variant for an implicit single configuration; once multiple variants exist, every variant requires a label. | Passed | 0 ms |
| catalog items and variants have optional descriptions | Catalog items and variants start without descriptions and can independently own optional label-backed descriptions. | Passed | 0 ms |
| explicit default variant is optional unique and owned by the variant | A multi-variant catalog item accepts zero or one explicit default marker and rejects multiple markers; after the marked variant is removed, a remaining sole variant becomes the effective default. | Passed | 0 ms |
| implicit configuration uses the default or sole variant | Configuration without a variant ID uses the effective default: an explicit marker for multiple variants or the sole variant; multiple unmarked variants require an explicit selection. | Passed | 0 ms |
| catalog item media is optional and preserves multiple definitions | A catalog item starts with an empty media collection and can preserve multiple ordered media definitions independently of its variants. | Passed | 0 ms |
| variant media is optional and preserves multiple definitions | A variant starts with an empty media collection and can preserve multiple ordered media definitions. | Passed | 0 ms |
| catalog item rejects duplicate variant IDs and currency mismatches | A catalog item cannot define duplicate variants and all variant invariant prices must share one currency. | Passed | 0 ms |
| catalog item configures known variant and prices shared modifiers | Configuring a known variant returns invariant price, hydrated shared modifiers, modifier price contributions, and total price. | Passed | 0 ms |
| catalog item rejects unknown variant | Invalid variant combinations are modeled by absence; configuring a missing variant fails. | Passed | 0 ms |
| selected variant controls modifier choice applicability | A selected variant can make a shared modifier choice inapplicable without duplicating the modifier tree. | Passed | 0 ms |
| selected variant controls modifier prompt applicability | A selected variant can make a shared modifier prompt inapplicable. | Passed | 0 ms |
| item with no modifier prompts accepts empty selection and rejects unknown prompt selection | An item with no shared modifiers hydrates empty selections and rejects unexpected prompt selections. | Passed | 0 ms |
| resolved variant combinations are modeled by variant existence | The core sees only concrete valid variants; unsupported combinations are unknown variant IDs. | Passed | 0 ms |
