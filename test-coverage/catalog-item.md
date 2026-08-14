# Catalog Item

Described behavior tests for ordered variant dimensions, deepest matches, explicit base pricing, presentation metadata, defaults, and configured item pricing.

## Definitions

- [Catalog item](../src/catalog_item/catalog-item.md)
- [Variant dimension](../src/catalog_item/variant-dimension.md)
- [Variant](../src/catalog_item/variant.md)
- [Variant match](../src/catalog_item/variant-match.md)
- [Media](../src/primitives/media/media.md)
- [Configured catalog item](../src/catalog_item/configured-catalog-item.md)

## Result

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 15
- Passed: 15
- Failed: 0

## Behaviors

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| catalog item rejects invalid basic variant authoring | Catalog item, dimension, and variant labels require text; dimensions require values; and every catalog item requires at least one variant match. | Passed | 0 ms |
| empty match models an item with no dimensions | A catalog item with no dimensions still has one required concrete selection: an empty deepest match that resolves its price without inventing an unnamed variant. | Passed | 0 ms |
| dimension order controls selection and combined label order | Variant IDs inside a match are unordered, but the catalog stores and resolves them in authored dimension order so a combined label is deterministic. | Passed | 0 ms |
| deepest matches can stop at different depths | Deepest is relative to authored supersets rather than the number of dimensions, so unrelated Crust and Size selections can each be concrete one-value matches. | Passed | 0 ms |
| sparse matches define only authored combinations | Dimensions do not imply a Cartesian product; pizza size and crust values form concrete selections only where a deepest match is authored. | Passed | 0 ms |
| deepest match uses only its own explicit price | Every match owns a required price, and configuration uses the selected deepest match's price without inheriting from a shallower match. | Passed | 0 ms |
| free variants require explicit catalog item permission | The allow_free_variant setting defaults to false, so a zero-priced deepest match is rejected unless the catalog item explicitly enables free variants. | Passed | 0 ms |
| matches reject invalid or duplicate variant sets | Variant IDs are unique across dimensions; a match cannot use two values from one dimension, reference an unknown value, or duplicate another unordered match. | Passed | 0 ms |
| match prices require one currency | All declared variant-match prices on one catalog item use the same currency. | Passed | 0 ms |
| explicit default match is optional unique and concrete | A multi-selection item accepts at most one default marker, and only a deepest match can carry it so removing the match cannot leave a dangling reference. | Passed | 0 ms |
| implicit configuration uses the default or sole deepest match | Configuration without explicit variant IDs uses the marked default or the sole deepest match; multiple unmarked deepest matches require a selection. | Passed | 0 ms |
| catalog items variants and matches have independent optional metadata | Catalog items and variant values independently own labels, optional descriptions, and media; matches own an optional exact label, description, and media without inheriting between scopes. | Passed | 0 ms |
| configured match returns effects and prices shared modifiers | Configuring a deepest match returns its effects, resolved invariant price, hydrated shared modifiers, modifier contributions, and total price. | Passed | 0 ms |
| deepest match controls modifier applicability | Different deepest matches can restrict shared modifier choices or prompts without duplicating the modifier tree. | Passed | 0 ms |
| item with no modifier prompts accepts only empty selection | An item with no shared modifiers hydrates empty selections and rejects unexpected prompt selections. | Passed | 0 ms |
