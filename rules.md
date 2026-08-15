# High Level Rules

This file is the hard operating agreement for POS Core Kernel. It should stay stricter and shorter than the exploratory domain notes.

## Core Boundary

- The core is a Rust domain library focused on correctness.
- The core contains no UI, storage engine, concrete payment processor, concrete surface, concrete order type, or concrete fulfillment workflow.
- Concrete behavior belongs in adapters, applications, plugins, or merchant policy layers.
- Persistence shape must follow the domain model, not define it.
- The same deterministic builders must be usable by client and server code.

## Identity

- Public IDs are typed wrappers, not raw strings.
- Public IDs use a three-letter uppercase prefix plus a sortable unique suffix.
- Each ID type owns its prefix.
- Stable IDs identify logical things over time.
- Versioned IDs identify exact facts from a published version.
- Deleting or hiding a draft entity must not break old catalog versions, orders, reports, or audit records.

## Catalog Publishing

- Draft catalogs may change freely.
- Published catalog versions are immutable.
- Publishing is the validation and cache boundary.
- A published catalog version may produce one or more deterministic catalog views.
- Surfaces and fulfillment modes are context axes for catalog views, not hard-coded core behavior.
- Live stock and local 86ing are outside the immutable catalog version.

## Labels

- Every catalog-authored human-facing string is represented by a label.
- Catalog definitions should not store raw display strings directly on items, variants, prompts, choices, fields, categories, buttons, receipt text, prep text, or customer-facing text.
- Labels may have stable IDs so catalog-authored strings can support translations, consumer-specific variants, and historical snapshots.
- Custom/manual order labels use the same `Label` primitive with no ID.
- A catalog item owns ordered variant dimensions; dimension order controls selection presentation and combined-label order but does not require every concrete selection to use every dimension.
- Every variant value belongs to one dimension and has a required label.
- Authored variant matches are concrete selectable paths. Authors may supply their variant IDs in any order, but the catalog stores each path in dimension order and rejects two values from the same dimension.
- Dimension order defines selection order and dependency. A match must contain one value from every earlier dimension it traverses, cannot skip a dimension, and may stop before later dimensions.
- One concrete match cannot be a strict subset of another. A path cannot be both a complete selection and a prefix requiring another selection.
- Concrete matches implicitly define the valid path tree. Given a partial path, the kernel exposes the next ordered dimension and only the values that continue into a compatible concrete match.
- Every variant match owns a required explicit invariant price. Prices do not inherit between matches, and configuration uses only the selected match's own price.
- The catalog-item variant setting `allow_free_variant` defaults to false. A zero-priced match is invalid unless the setting is explicitly enabled.
- A catalog item with no dimensions uses one empty match so a simple item has a required configuration and price without an unnamed variant value. Empty matches are invalid when dimensions exist.
- Catalog items and variant values may independently own optional label-backed descriptions and media.
- Variant matches may independently own an optional exact label, optional label-backed description, and optional media.
- An exact match label overrides the derived combined display label without erasing the selected component labels.
- Description and media never inherit or merge across catalog-item, variant-value, and variant-match scopes inside the kernel.
- Catalog-item, variant-value, and match descriptions and media are catalog presentation metadata and are excluded from configuration and order snapshots.
- A sole match is always its catalog item's effective default without requiring a marker.
- With multiple matches, one match may carry the optional explicit default marker. A catalog item may contain at most one marker, and removing that match removes the explicit default with it.
- Catalog-item, variant-value, and variant-match media are independent and optional, each represented by an ordered `MediaCollection` allowing zero or more media definitions.
- Media fallback and precedence across catalog definitions are client presentation concerns; the kernel exposes collections without combining them.
- Structural strings such as typed IDs, standard kinds, currency codes, and internal enum names are not labels.
- Labels resolve against a `ConsumerProfile`, which is an ordered list of unique generic `ConsumerAttribute` IDs.
- Surfaces, fulfillment modes, locale preferences, printers, receipts, prep displays, and other consumers may contribute attributes to a profile.
- Label resolution chooses the most specific matching value, then uses active consumer-profile order to break equal-specificity ties.
- Earlier consumer-profile attributes have higher precedence; label definition order does not affect resolution.
- Translations are label value variants selected by language or locale attributes in the active consumer profile.
- Order snapshots preserve the resolved label values used at order time and may also preserve label IDs for traceability.
- Changing a label definition or translation must not reinterpret an existing order.

## Distributed Operations

- Clients can create IDs and local operations before reaching a central database.
- Client operations are append-only, idempotent, and replayable.
- Commands validate intent.
- Events record accepted facts.
- Replaying the same events in the same order must produce the same state.
- Invalid commands do not emit business-state-changing events.

## Money

- No floats anywhere in core domain logic.
- Money is stored as integer minor units.
- All money arithmetic goes through the `primitives::money` module.
- All money arithmetic is checked for overflow and currency mismatch.
- Percentages, factors, and rates are represented as integer ratios, never floats.
- Fractional-money results use explicit named rounding strategies from the `primitives::money` module.
- Modifier prices cannot be negative.
- Discounts, coupons, refunds, and adjustments are explicit entries or allocations, not hidden negative modifier prices.

Suggested rounding strategy names:

- `CentRoundUp`
- `CentRoundDown`
- `NearestUpIncrement(5)`
- `NearestUpIncrement(10)`
- `NearestUpIncrement(25)`
- `NearestUpEnding(49)`
- `NearestUpEnding(99)`

## Time And Calendar

- Timed occurrences are stored as `primitives::time::UtcTime`.
- `UtcTime` is a value object, not an entity, and does not own an ID.
- No code in the core reads the current system clock.
- Domain operations must receive time as an input; they must not call `now`, `SystemTime::now`, or `Instant::now` directly.
- `EvaluationTime` is the explicit input for time-dependent domain behavior.
- `EvaluationTime` pairs the UTC instant with the resolved local `CalendarMoment` used for business calendar checks.
- `TimeZone` stores an IANA/tzdb-style zone name such as `America/Los_Angeles`.
- Time zone display names and translations belong to CLDR-backed presentation or labels, not raw timezone logic.
- Local business concepts belong in `primitives::calendar`.
- `calendar` owns logical dates, days of week, hours of day, and business-date values.
- `schedule` owns deterministic temporal predicates made from UTC limits, local calendar windows, and local exclusions.
- Empty schedules are always scheduled; `Never` and schedule limits are explicit.
- A scheduled choice may only be evaluated when an `EvaluationTime` is supplied.
- A choice outside its schedule is not visible and cannot be selected at that `EvaluationTime`.
- Temporary 86ing is schedule-shaped availability. Live stock is a separate operational layer and is not owned by catalog schedules.
- Core logic should not pass raw local date strings around as business rules.
- If a workflow needs both real time and business time, store the UTC occurrence and the calendar interpretation separately.

## Providers And Supply

- Providers are explicit inputs to deterministic domain behavior.
- Providers may wrap storage, caches, snapshots, HTTP services, test fixtures, or future async adapters.
- Async fetching happens outside the deterministic core. Core logic consumes resolved provider outputs or provider views.
- Domain methods must not hide global provider lookups.
- Supply means generic limited fulfillability, not only stock.
- Supply can model inventory, daily caps, delivery slots, prep capacity, external availability, calculated policy limits, and similar constrained resources.
- Supply is already scoped by the active order/application context. Do not bake location into the core supply target by default.
- Supply targets can be catalog items, modifier choices, or custom keys.
- Supply buckets qualify targets with deterministic dimensions such as time window, capacity class, or service period.
- Unknown supply is `Unresolved`, not the same as unavailable.
- `Reserve` and `Unreserve` are a reversible provisional claim pair.
- `Consume` and `Unconsume` are a reversible final-use claim pair.
- Supply reversals reference the original claim instead of creating negative quantities.
- Supply ledgers validate claim transitions and remain storage-free.

## Entries And Totals

- Orders and transactions use generic, descriptive entries.
- Entries snapshot human-readable and financial meaning when they are created.
- An order must contain all state needed to represent itself.
- An order may keep references to catalog, customer, actor, payment, or integration entities for traceability, but it must not require those references to render, price, refund, audit, or report what was ordered.
- Catalog-backed order lines snapshot selected variants, modifier selections, entered choice-input values, labels, prices, price contributions, taxes, effects, and totals needed to explain the order.
- Changing catalog labels, defaults, modifier rules, prices, or deleted entities must not reinterpret an existing order.
- Totals are derived from entries and allocations.
- Financial accounting is append-only.
- Corrections create new entries; they do not rewrite old facts.
- Reporting reads standardized, versioned facts rather than mutable catalog state.

## Standards

- Flexible concepts need standardized logical categories.
- Standard values are API contracts and should be documented, tested, and versioned.
- Extension points must be explicit.
- Unknown custom values must not silently inherit built-in behavior.

Important standard vocabularies include:

- component and catalog item kinds
- order entry kinds
- transaction entry kinds
- price categories
- accounting categories
- total categories
- allocation modes
- state facet names and values
- command names
- event names
- reason codes
- surface IDs
- fulfillment mode IDs

## Rust Organization

- Low-level shared building blocks live under `primitives`, including typed IDs, money, labels, media, consumer profiles, time, calendar, and schedule types.
- Entity modules use communicative three-letter prefixes in file names when an entity owns a public ID.
- Public fields use `snake_case`.
- Reserved words are avoided; prefer names like `kind`, `position`, and `sort_order`.
- Domain modules should expose small, testable APIs before persistence or transport abstractions.

## Domain Documentation

- Important domain terms must be defined where their owning code lives.
- A standalone definition uses a lowercase kebab-case Markdown file beside the Rust source that owns the concept.
- The owning Rust type includes that file with `#[doc = include_str!("definition-name.md")]` so the Markdown file and Rustdoc share one source of truth.
- Definitions state what a concept means, what facts it contains, and its boundary from adjacent concepts.
- Behavior reports link to the definitions required to understand their test names and descriptions.
- Test descriptions use the precise domain term, such as `configuration snapshot` or `order-item modifier snapshot`, instead of an ambiguous shorthand such as `snapshot`.

## Tests

- Tests should read like documentation.
- Every behavior test should have a concise description.
- A reader should not need to inspect an assertion body to identify the domain concepts named by a behavior.
- Module test reports are written to `test-coverage/`.
- Markdown reports are preferred for now because they are readable in git.
- Test coverage should focus first on invariants, deterministic configuration snapshots, pricing, events, and replay.
