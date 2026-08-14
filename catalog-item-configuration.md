# Catalog Item Configuration

This is the current design target for catalog items, variants, modifiers, fields, and modifier price calculation. The Rust implementation currently covers variants, shared modifiers, variant applicability, and modifier pricing. Fields remain design-only while the model is being worked out.

## Shape

`CatalogItem` is the sellable item family.

```text
CatalogItem
  Variants
    Variant selection path
  Fields
  Modifiers
    Prompt
      Choice
        Modifiers
```

The important decision is that variants do not each own separate modifier trees. A catalog item owns one root field set and one root modifier tree. The selected variant provides the context that determines which fields and modifiers are applicable and how their prices resolve.

## Vocabulary

- `CatalogItem`: the item family, such as pizza, coffee, burger, or catering platter.
- `Variant`: required base configuration for the item, such as size, temperature, crust, or other base choices.
- `ResolvedVariant`: a valid selected variant combination with one invariant base price.
- `Label`: stable display text identity with consumer-profile-specific values.
- `MediaCollection`: ordered catalog media metadata for a sellable thing or choice.
- `Media`: one media slot with a required default representation and optional consumer-profile-specific variants.
- `ConsumerAttribute`: generic atomic consumer trait, such as web, delivery, Spanish, prep, short, receipt, or kiosk.
- `ConsumerProfile`: compact set of consumer attributes used to resolve labels and, later, other context-specific behavior.
- `Field`: typed custom input on a catalog item. Fields are separate from modifiers.
- `Modifiers`: a container for prompts.
- `Prompt`: one selectable question.
- `Choice`: one selectable answer.
- `Rule`: validation or defaulting rule.
- `Effect`: a typed emitted fact consumed elsewhere.
- `ConfigurationSnapshot`: the single in/out format for selected item state. Input may be sparse; output is order-safe and includes resolved labels, prices, sources, effects, and totals.
- `Applicable`: usable for the selected variant and context.
- `Selected`: present in the current configuration.

The core should prefer `applicable` and `selected` for structural validity. `visible` is reserved for time-filtered catalog traversal: a scheduled choice outside its `EvaluationTime` is not part of the currently visible choice tree. A UI can still decide how to present invalid or stale selections, but the core owns the visible tree calculation.

## Catalog Item Validity

A catalog item configuration is valid when:

- exactly one valid variant combination is resolved
- a sole variant may be unlabeled, while every variant must have a label when multiple variants exist
- every applicable required field is valid
- every applicable required modifier prompt is valid
- every nested applicable modifier prompt is valid
- all selected choices pass their own rules
- all selected choices are compatible with the selected variant

Validity bubbles upward:

```text
Choice valid
  -> Prompt valid
  -> Modifiers valid
  -> CatalogItem valid
```

If no variant is selected, the catalog item configuration is invalid. Root modifiers and fields are not applicable until a variant is selected.

## Variants

A variant is a base-level required configuration. It decides the invariant price used by the item and by modifier pricing.

Examples:

- pizza size
- pizza crust
- coffee size
- coffee hot/cold
- catering package size

A variant selection does not need to have a default. A merchant may provide a default variant for convenience, but the invariant is that a valid catalog item configuration must resolve a variant.

A catalog item must define at least one priced variant. When an item has exactly one variant, that variant is the implicit configuration and may omit its label because there is no user-facing variant distinction to select. A sole variant may still have a label when the distinction is meaningful. Once an item has multiple variants, every variant must have a non-empty label. Expanding an unlabeled single-variant item into multiple variants therefore requires naming the existing variant as part of the same authoring change.

Variant choices often need relationships. Some combinations are invalid for business reasons unrelated to stock, such as a small pizza not supporting a certain crust or a cold drink not supporting a hot-drink add-on.

The current shape can support this as a variant path:

```text
Size
  Small
    Crust
      Regular
      Gluten Free
  Large
    Crust
      Regular
      Gluten Free
```

Each valid leaf or resolved combination has a flat invariant price. Parent or intermediate variant nodes may exist to organize the path and constrain child choices. Pricing may use parent relationships internally, but the result exposed to item pricing is one resolved invariant price.

## Variants, Fields, And Modifiers

Variants are the source of applicability and pricing context for fields and modifiers.

A selected variant can affect:

- whether a field is applicable
- whether a modifier prompt is applicable
- whether a choice is applicable
- whether a nested modifier branch is applicable
- the invariant price used by choice price formulas
- future field or modifier price adjustments

The modifier tree itself remains shared. This prevents duplicating the same modifier structure under every variant.

Example:

```text
CatalogItem: Pizza
  Variants:
    Small Regular: $12.00
    Small Gluten Free: $14.00
    Large Regular: $18.00
    Large Gluten Free: $22.00
  Modifiers:
    Prompt: Meat toppings
      Choice: Pepperoni
        Prompt: Placement
          Choice: Left
          Choice: Right
          Choice: Whole
```

The placement prompt is defined once. The selected variant decides the invariant pizza price, and may also decide whether the toppings prompt is applicable.

## Fields

Fields are not modifiers. They are typed inputs attached to the catalog item.

Possible future examples:

- special instructions
- name on cake
- pickup note
- catering guest count
- allergy note

Special instructions are intentionally not finalized yet. They may become first-class, a field, or a separate order note concept. The key decision for now is that fields and modifiers both live on `CatalogItem` and both use selected variants as their applicability and pricing context.

## Modifiers

The modifier domain shape stays:

```text
Modifiers
  Prompt
    Choice
      Modifiers
```

`Modifiers` is a container for prompts.

`Prompt` owns:

- prompt ID
- label
- optional description label
- rules
- effects
- choices

`Choice` owns:

- choice ID
- label
- media collection
- optional schedule
- rules
- effects
- optional nested modifiers
- pricing definition

Choice schedules are authored availability. If a choice has a schedule, it can only be evaluated with an explicit `EvaluationTime`; if it has no schedule, time does not restrict it. A choice outside its schedule is not visible in time-filtered modifier traversal and cannot be selected. Temporary 86ing should be represented as schedule-shaped availability, while live stock remains a separate operational layer outside the catalog.

Effects remain separate from rules and pricing. A choice may emit effects, but modifiers do not need to know how stock, prep, tax, reporting, or order policies consume them.

## Rules

Rules describe validation and defaulting. They are not effects.

Current rule vocabulary:

- `Min(quantity)`: minimum selected quantity; default is `0`
- `Max(quantity)`: maximum selected quantity; default is unbounded
- `Default(quantity)`: default quantity for a choice; default is no default

There is no separate `Required` rule. A prompt with `Min(1)` and no default starts invalid until the user selects a choice. A prompt with `Min(1)` and one default choice starts valid.

Rules should be unique by rule kind for a given owner. For example, a prompt cannot have two `Min` rules, and a choice cannot have two `Default` rules.

Prompt rules validate the total selected quantity inside that prompt. Choice rules validate the quantity of that individual choice.

`Min(1)` plus `Max(1)` makes a single-select prompt:

```text
Prompt: Placement
  Rules: Min(1), Max(1)
  Choices:
    Left
    Right
    Whole
```

## Configuration Snapshot

The system should use one shape for configuration input and output.

Sparse input from a UI may contain only IDs, quantities, sources, and nested selections. Resolved output for an order should use the same tree shape but include every fact needed to render and explain the configured item without consulting the original catalog definition.

An order-safe configuration snapshot should include:

- catalog item ID and label
- catalog version or catalog view identity used to resolve it
- selected variant ID and optional label; the label may be absent only for a sole implicit variant
- variant invariant price
- prompt IDs and labels
- choice IDs and labels
- quantities
- selection source, such as explicit or default
- nested modifier snapshots
- choice price definitions used
- resolved price contributions
- applied factors and rounding strategy
- emitted effects
- total modifier price
- total configured item price

Media is catalog/view presentation metadata and is not included in configuration or order snapshots by default.

References are still useful for traceability and editing. They are not sufficient for representing an order. A receipt, refund, kitchen reprint, audit view, and report must be explainable from the order snapshot itself.

Changing catalog defaults, labels, prices, modifier rules, or deleting catalog entities must not reinterpret an existing order.

## Labels

Every catalog-authored human-facing string is represented by a label.

Catalog definitions should not store raw display strings directly on catalog items, variants, prompts, choices, fields, categories, buttons, receipt text, prep text, or customer-facing text.

A label needs stable identity because the same catalog object may need different strings for different consumers. Labels also let order snapshots preserve the exact resolved text while retaining traceability back to the catalog label identity.

Current label shape:

```text
Label
  label_id
  default
  values
    required ConsumerProfile -> value
    required ConsumerProfile -> value
```

Resolution:

- a label value matches when all of its required consumer attributes are present in the active `ConsumerProfile`
- the most specific matching value wins
- equally specific matching values are invalid label data
- if no value matches, the label default is used

Important consumer attributes may represent:

- customer receipts
- kitchen or prep displays
- printers
- POS buttons
- kiosk/web/app display
- fulfillment modes such as delivery or pickup
- locale or language preferences
- reporting and audit views

Snapshots should preserve the resolved label values used at order time. They may also preserve the label ID for traceability, but rendering an old order must not require resolving the label from the current catalog.

The current Rust code stores label-backed fields for catalog items, labeled variants, prompts, choices, and prompt descriptions. A sole implicit variant may have no label; this is represented as label absence, never as an empty `Label`. Some getter names still expose `title` and `description` strings as compatibility/readability helpers over the default or resolved label value.

## Media

Media is a primitive collection of catalog-authored media metadata. It does not own paths, URLs, storage keys, blobs, CDN behavior, or rendering rules.

Current media shape:

```text
MediaCollection
  Media
    media_id
    mime_type
    label?
    dimensions?
    variants
      required ConsumerProfile
      media_id
      mime_type
      label?
      dimensions?
```

Rules:

- every `Media` has a default `media_id` and MIME type
- `MediaCollection` preserves definition order
- duplicate default media IDs are invalid inside one collection
- media variants require a non-empty `ConsumerProfile`
- media resolution chooses the most specific matching variant and falls back to default
- equally specific matching variants are invalid
- media labels resolve with the same active `ConsumerProfile`
- dimensions are optional, but width and height must be nonzero when present
- choices can own a media collection
- media does not survive into configuration or order snapshots unless a future workflow explicitly needs that

## Selection Identity

Choice IDs must be unique inside a prompt.

Duplicate prompt IDs may be valid when they are separate ordered prompt occurrences. Because of that, configuration snapshots should not be keyed only by prompt ID. They need ordered occurrence or path information.

Conceptual sparse snapshot shape:

```json
{
  "prompts": [
    {
      "prompt_id": "CMP-PLACEMENT",
      "occurrence": 0,
      "choices": [
        {
          "choice_id": "CMP-LEFT",
          "quantity": 1,
          "modifiers": {
            "prompts": []
          }
        }
      ]
    }
  ]
}
```

The resolved form stores the effective selection snapshot, including defaults. Hydration can still apply defaults when no selection payload is provided, but once an order is created its modifier snapshot should preserve what was selected at that time.

This allows partial rehydration after a catalog changes. If an old selection references a choice ID that still exists in the same structural position, it can be carried forward. If the choice no longer exists or moved into an incompatible prompt, hydration should report an unresolved selection instead of pretending the old order still matches the new catalog.

A definition hash can be useful for caching or quick mismatch detection, but correctness should rely on typed IDs, ordered paths, and explicit rehydration results.

## Defaults

Defaults are included configuration, not charged selections, when `defaults_are_free` is enabled.

Rules:

- default choices can satisfy prompt minimums
- configuration snapshots include default selections so orders are stable
- removing a default does not create negative price
- default selected priced choices contribute `0` when defaults are free
- nested default factor choices may still shape the price of an explicit priced ancestor

Example:

```text
Prompt: Toppings
  Choice: Pepperoni $2.00, Default(1)
```

With `defaults_are_free`, the default pepperoni contributes `$0.00`. If the customer removes pepperoni, the order does not receive a negative `$2.00` credit. Discounts and coupons are separate concepts.

## Pricing Goals

Modifier pricing should be:

- deterministic
- explainable
- non-negative
- testable as output
- independent of UI
- based on integer money and integer rates
- rounded only through named money strategies

Every price calculation should produce a traceable contribution:

- source choice path
- selected quantity
- selection source, such as default or explicit
- invariant price used
- flat amount used
- invariant rate used
- factors applied
- rounding strategy used
- resolved amount
- price category

## Price Terms

`Invariant price` is the resolved variant base price.

`Flat amount` is a non-negative money amount on a priced choice.

`Invariant rate` is a non-negative integer ratio applied to the invariant price.

`Factor` is a non-negative integer ratio selected by a child choice and applied upward to the nearest priced ancestor branch.

`None` means no amount, rate, or factor. A factor of `0%` is allowed only if we deliberately want a selected branch to become free; it is not the default.

Old naming map:

- flat price -> flat amount
- variant percent price -> invariant rate
- parent percent price -> factor

## Choice Price Definition

Conceptual shape:

```text
ChoicePrice
  flat_amount: Money = 0
  invariant_rate: Rate = 0
  factor: Option<Rate> = None
```

The base contribution for a priced choice is:

```text
flat_amount + (invariant_price * invariant_rate)
```

Selected descendant factors multiply the nearest priced ancestor branch:

```text
resolved_branch_price =
  (flat_amount + invariant_component)
  * selected_factor_1
  * selected_factor_2
  * ...
```

The result cannot be negative because all inputs are non-negative.

If a descendant choice has its own flat amount or invariant rate, it starts its own priced branch. Factors below that descendant apply to the descendant branch first.

## Factor Examples

Pizza variant:

```text
Large Regular Pizza: $18.00 invariant price
```

Pepperoni placement:

```text
Choice: Pepperoni
  Flat amount: $2.00
  Prompt: Placement
    Choice: Left, Factor 50%
    Choice: Right, Factor 50%
    Choice: Whole, Factor 100%
```

Resolved prices:

```text
Pepperoni + Whole = $2.00 * 100% = $2.00
Pepperoni + Left  = $2.00 * 50%  = $1.00
Pepperoni + Right = $2.00 * 50%  = $1.00
```

Nested factor:

```text
Pepperoni: $2.00
  Left: 50%
    Some nested factor: 50%
```

Resolved price:

```text
$2.00 * 50% * 50% = $0.50
```

Invariant rate example:

```text
Large Regular Pizza: $18.00 invariant price
Choice: Premium topping
  Flat amount: $1.00
  Invariant rate: 10%
```

Resolved before factors:

```text
$1.00 + ($18.00 * 10%) = $2.80
```

## Rounding

All percentages and factors are exact integer ratios until money must be materialized.

The default rule should be:

- calculate one branch as rational minor units
- apply all selected factors
- round once at the branch boundary
- sum rounded branch contributions

Rounding is always performed by the `primitives::money` module with an explicit named strategy.

Suggested strategy families:

- cent up
- cent down
- nearest upward increment, such as `5`, `10`, or `25` cents
- nearest upward price ending, such as `49` or `99` cents

The rounding strategy should be part of the catalog item pricing policy, not embedded in one modifier choice.

## Pricing Policy

Conceptual shape:

```text
CatalogItemPricingPolicy
  defaults_are_free: bool
  modifier_rounding: RoundingStrategy
```

`defaults_are_free` controls whether default selected priced choices contribute zero.

`modifier_rounding` controls fractional minor units from invariant rates and factors.

The exact Rust shape can change, but the policy must be explicit so tests can prove every pricing path.

## Effects And Pricing

Effects still matter, but the price calculation should not require every consumer to interpret arbitrary effect payloads.

The current direction is:

- price definitions produce deterministic price contributions
- effects emit separate typed facts for other domains
- order building consumes price contributions and effects
- taxes, coupons, reporting, prep time, and stock can later consume effects without changing modifier hydration

This keeps modifiers testable before the full order engine exists.

## Future Tests

The eventual implementation should cover:

- no variant selected is invalid
- variant selected with no default is valid only after explicit selection
- variant paths reject incompatible combinations
- variant path resolves one invariant price
- root modifiers are inapplicable before variant selection
- selected variant controls modifier applicability
- selected variant controls field applicability
- prompt `Min` and `Max` rules
- choice `Min`, `Max`, and `Default` rules
- duplicate rule kind rejection
- duplicate choice rejection inside a prompt
- duplicate prompt occurrence hydration
- configuration snapshots include defaults
- partial rehydration reports unresolved old selections
- defaults are free
- removing defaults does not produce negative price
- flat choice price
- invariant rate choice price
- factor applied to nearest priced ancestor
- nested factors multiply upward
- factor does not emit standalone money
- rounding strategy is explicit
- no float-based price path exists
