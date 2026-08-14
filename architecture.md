# Architecture

POS Core Kernel should start as a Rust library crate with a tight domain API and a distributable client library. Applications, databases, sync layers, and hardware integrations should sit outside the core.

## Principles

- The core library owns correctness.
- Client libraries can create deterministic local operations.
- Persistence is abstracted behind ports.
- Domain operations are deterministic.
- Published catalog data is immutable.
- Order totals are reproducible from snapshots.
- External systems adapt to the domain model, not the other way around.
- Standard vocabularies are treated as API contracts.
- The core contains no UI, concrete payment processors, concrete surfaces, concrete order types, or concrete fulfillment behavior.

## Suggested Layers

### Domain

Pure Rust types and functions.

Responsibilities:

- validate catalog composition
- publish catalog versions
- resolve catalog trees
- price items and orders
- enforce modifier constraints
- resolve supply requests through explicit providers
- track supply claim transitions
- snapshot order lines
- transition order states

The domain layer should not know about SQL, HTTP, terminals, printers, or payment processors.

The domain layer should also avoid concrete product behavior. It may define abstractions, IDs, contracts, command/event shapes, validation hooks, and standards, but not specific UI workflows, payment processors, surface implementations, order type implementations, or fulfillment implementations.

Concrete behavior belongs in adapters, plugins, policies, or application layers.

### Primitives

Reusable low-level building blocks live under `primitives`, not as separate root domains.

Current primitives include:

- `ids`: typed prefixed IDs and ID parsing
- `money`: integer money, rates, rational money, and rounding strategies
- `label`: stable label identity and consumer-profile resolution
- `media`: media collections, MIME metadata, dimensions, and consumer-profile variants
- `consumer`: consumer attributes and consumer profiles
- `time`: UTC instants and IANA/tzdb-style time zone names
- `calendar`: local logical dates, days of week, times of day, and date/time ranges
- `schedule`: deterministic temporal predicates over explicit UTC and calendar context

Future primitives should include timezone rule resolution when the project is ready to convert UTC instants into local calendar moments inside a controlled boundary.

No core primitive reads the current system clock. Domain operations should receive `EvaluationTime` as an argument so tests, replays, reports, and offline clients can supply exact moments. `EvaluationTime` pairs a `UtcTime` with the resolved local `CalendarMoment`; schedule evaluation consumes that explicit value.

### Ports

Traits that describe required persistence and integration behavior.

Examples:

- `CatalogRepository`
- `OrderRepository`
- `SupplyProvider`
- `IdGenerator`

Ports should be small and scenario-driven. Avoid building a generic database abstraction too early.

Providers are the boundary for outside facts. A provider can wrap a local snapshot, cache, database adapter, HTTP adapter, test fixture, or future async resolver, but the deterministic core should consume the provider result explicitly. The core should not hide global lookups behind domain methods.

Async resolution belongs outside deterministic domain calculation. For example, stock, delivery capacity, prep throttles, coupon limits, and marketplace state can be fetched asynchronously by an application layer, then supplied to core logic as provider inputs or resolved views.

### Client Library

A shipped client library is a core product surface.

The client library should let POS terminals, kiosks, phone order tools, web ordering flows, and integrations perform deterministic work locally:

- load published catalog views
- create prefixed IDs
- validate commands
- create orders
- add and edit entries
- split checks
- calculate totals
- append local operations
- replay local events
- prepare sync payloads

The client library should not require direct database access. It should operate from catalog views, local order state, actor context, and standardized command/event definitions.

Server APIs can then accept client operations, deduplicate them, validate them against server policy, and either accept, reject, or mark them for resolution.

The client and server should use the same deterministic builders for order entries and item configuration. A client-created item and a server-created item should produce the same entries, option mapping, prices, totals, and validation errors when given the same catalog view and command inputs.

The shared builder should be part of the core library, not duplicated in application code.

### Adapters

Concrete implementations of ports.

Examples:

- SQLite/Postgres catalog storage
- in-memory repositories for tests
- HTTP API
- CLI or desktop shell
- future payment or hardware integrations

Adapters and applications own concrete behavior:

- UI
- payment processors
- surface-specific presentation
- order type workflows
- fulfillment workflows
- hardware integrations
- merchant-specific policy

## Persistence Direction

The current relational sketch is useful, but it should follow the domain model after the invariants are stable.

Important storage decisions still open:

- Whether published versions physically copy rows or use immutable versioned rows.
- Whether tree edges reference row IDs, stable entity IDs plus versions, or both.
- Whether translations live in JSON columns or normalized translation tables.
- Whether media ordering is represented by a join table.
- How draft version `0` avoids uniqueness collisions with published versions.

## Asynchronous Consumption

Catalogs are published and consumed asynchronously. Clients may be online, offline, stale, or mid-order while a new catalog is published.

The core architecture should assume:

- multiple catalog versions can be valid at once
- clients may finish orders against an older downloaded catalog
- deleted components can still be referenced by old orders or old catalog versions
- the latest catalog is a discovery concern, not the only valid catalog
- order correctness depends on snapshots, not live catalog lookups

Stable IDs and versioned IDs make this manageable.

Stable IDs answer: "Is this the same logical thing over time?"

Versioned IDs answer: "Which exact facts were true when this was read, priced, or ordered?"

Persistence can represent that distinction in different ways, but the domain model should preserve it explicitly.

## Catalog Publishing

Publishing should be a single correctness boundary.

At publish time:

- validate the full catalog tree
- resolve inherited settings where appropriate
- verify there are no invalid modifier rules
- verify referenced media, tax classes, surfaces, and fulfillments exist
- create a new immutable catalog version
- generate a cacheable resolved catalog snapshot
- make the new version discoverable by clients

This gives the system one cache generation point and lets clients safely keep using older catalog versions.

The resolved snapshot can be treated as a publish artifact. It should be cheap to serve, deterministic to regenerate, and tied to one `CatalogVersionId`.

Snapshot generation can resolve:

- tree shape
- inherited availability
- inherited fulfillment and surface restrictions
- price and tax overrides
- price categories
- display names and fallback names
- media ordering
- deleted or hidden component filtering

The snapshot is not the source of truth. It is a reproducible read model generated from the validated catalog version.

## Catalog Views

Different consumers often need different versions of the same published catalog. These should be modeled as catalog views, not separate source catalogs.

A catalog view is identified by the published version plus a view context:

- `CatalogVersionId`
- `SurfaceId`
- optional `FulfillmentModeId`
- optional `MerchantId`
- optional locale
- resolved `ConsumerProfile`

Views can be generated eagerly at publish time or lazily on first request. Either way, generation must be deterministic and tied to the exact catalog version.

Good view outputs:

- `pos` view for staff workflows
- `phone_order` view for call center or counter entry
- `kiosk` view with stricter media and display constraints
- `web` view filtered for public online ordering
- `catering` view with packages, lead times, and quantity constraints
- fulfillment-specific views such as `web + delivery`, `web + pickup`, `pos + dine_in`, or `phone_order + catering_pickup`
- merchant-local view that overlays live stock or 86ing without mutating the catalog version

This keeps publishing as the cache point while still letting consumers get purpose-built catalogs.

Surface and fulfillment mode should be modeled as orthogonal inputs. Surface answers who is consuming the catalog. Fulfillment mode answers how the order will be completed. Both can contribute generic consumer attributes to the resolved `ConsumerProfile` used by labels and other context-specific catalog behavior.

The domain should validate allowed combinations instead of relying on consumers to infer them.

## Supply

Supply is generic limited fulfillability, not only stock.

Useful supply examples include:

- modifier choice inventory
- catalog item inventory
- daily item caps
- delivery slot capacity
- prep capacity
- marketplace availability
- calculated merchant policy limits

The active order context is already scoped by merchant, brand, location, surface, fulfillment, actor, and other application concerns. Supply should not bake location into every target unless the application intentionally models a location-like dimension as a bucket.

Supply requests are made against a target and quantity. A target can be a catalog item, a modifier choice, or a custom supply key for resources that are not catalog-authored. Buckets further qualify a target with deterministic dimensions such as `time-window`, `capacity-class`, or `service-period`.

Supply providers answer requests as:

- available
- unavailable with a reason
- unresolved when the provider does not have enough information

Unresolved is important because a distributed POS can be offline, stale, or waiting for an external system. The core should not pretend unknown supply is the same as unavailable supply.

Supply mutations are expressed through claim operations:

- `Reserve`: provisional claim
- `Unreserve`: reversal of a provisional claim
- `Consume`: final use
- `Unconsume`: reversal of final use

Reserve/Unreserve and Consume/Unconsume are paired names on purpose. They are clear event verbs even if the grammar is plain. Reversals reference the same claim instead of manufacturing negative quantities.

The supply ledger is deterministic. It validates claim transitions and can calculate reserved and consumed quantities for a target and bucket. It does not own storage, networking, global stock counts, or async behavior.

## Correctness Model

Correctness should come from explicit state transitions, events, and pure calculations.

Examples:

- An order cannot move from `open` directly to `fulfilled`.
- A paid order cannot have catalog-driven line item price changes.
- A required modifier list cannot be satisfied by a deleted modifier.
- A catalog version cannot contain an edge to a component from another catalog version.

## Standards

The flexible parts of the system need standardized language.

The project should maintain versioned standards for:

- component kinds
- order entry kinds
- transaction entry kinds
- price categories
- accounting categories
- total categories
- allocation modes
- state facets
- command names
- event names
- reason codes
- surface IDs
- fulfillment mode IDs

These standards are part of the public contract. They should be documented, tested, and versioned like code.

Custom extensions should live in explicit namespaces or registries. A custom value can exist, but it should not accidentally inherit the behavior of a standard value unless that behavior is declared.

## ID Standard

Public IDs should use uppercase three-letter prefixes and a sortable unique suffix.

Format:

```text
ORD-01HX7Y9M8N6ZQ4K3V2B1C0D9EA
```

The prefix gives immediate human context. The suffix should be globally unique and roughly time-sortable, such as a ULID or UUIDv7-derived value.

Rust should still use typed wrappers around these IDs under `primitives::ids` so the compiler can distinguish `OrderId`, `EntryId`, `CatalogVersionId`, and other identifiers.

## Users, Actors, And Access

The system should model who performed an action separately from what business entity changed.

A user is a human or account identity. An actor is the command performer: staff user, customer, integration, device, service account, or system process.

Every command should carry actor context. Events should include enough actor attribution for audit and support.

Access control should be policy-based and deterministic. Policies should be testable functions over actor context, merchant/brand scope, surface, fulfillment mode, current state, and attempted command.

RBAC can be one policy input, but the architecture should not assume role checks are the entire authorization model. Order state, line state, merchant scope, and command type matter too.

## Event Driven Core

The system should be stable and event driven.

Commands express intent. Events record validated facts. State is derived by applying events in order.

For orders, this means actions can be deterministic:

- load current order state from events
- validate the command against that state
- use the relevant catalog view or snapshot inputs
- emit zero or more events
- apply those events to produce the next state

This supports auditability, retries, offline clients, and eventual consistency without making the domain unpredictable.

Events should be designed as durable business facts, not implementation logs. They should carry the snapshot data needed to explain and replay behavior even after catalogs, prices, or names change.

## Distributed Clients

Clients should be able to perform deterministic operations without immediately reaching the central database.

For example, a POS terminal, kiosk, phone order screen, or offline device can create an order locally using client-generated IDs and a downloaded catalog view. It can append local operations and sync them later.

The architecture should support:

- client-generated prefixed IDs
- local append-only operation logs
- idempotency keys
- device IDs
- actor context
- client sequence numbers
- observed catalog version IDs
- deterministic event replay
- sync acceptance, rejection, and resolution states

The central system should treat synced operations as facts to validate, deduplicate, accept, reject, or mark for resolution. It should not require every order action to be born inside the central database.

Some operations can complete locally because they only depend on the downloaded catalog view and current local order state. Other operations need remote confirmation, such as payment authorization, inventory reservation, loyalty redemption, coupon redemption limits, marketplace acceptance, or final settlement.

This keeps the POS usable under unreliable network conditions while preserving correctness boundaries.

Client/server parity is a correctness requirement. The server should not maintain a separate item builder with different business behavior from the client library. If policy differs between client and server, the difference should be explicit and tested.

## Entries

Orders and transactions should use a shared entry-shaped model.

An entry is a generic, descriptive record with:

- stable entry ID
- kind
- optional parent entry ID
- source reference
- source status
- human-readable description snapshot
- amount or quantity fields
- state
- metadata for non-critical extension data

Order entries describe what was sold, adjusted, prepared, printed, bagged, or removed.

Transaction entries describe how money moved, was authorized, captured, refunded, voided, or allocated.

Entries can also carry reporting and settlement meaning, such as category totals, tax lines, tax liabilities, and tax paid by a third party.

The shared shape gives the system flexibility without losing correctness. The `kind` determines which validation rules apply, and the description snapshot keeps receipts, audits, and offline clients understandable even after catalog changes.

Critical financial meaning should be represented by typed transaction entry kinds and allocation events, not loose metadata.

Discount entries should preserve original prices instead of rewriting them. An item-level discount should be a separate entry or allocation tied to the discounted item entry, with explicit rules for modifier inclusion, price category eligibility, quantity scope, tax treatment, stacking, funding party, and refunds.

Entries may be catalog-backed, externally mapped, externally unmapped, manual, or system-generated.

Unmapped external entries are valid order facts. They should carry enough source data to be auditable and replayable, but they should not inherit catalog behavior by accident. Mapping an external item to a catalog component later should be an explicit event, not a silent reinterpretation.

## Price Categories

Price categories are part of the pricing model, not the storage model.

They give priced entries stable logical meaning:

- base item amount
- modifier amount
- upgrade amount
- included amount
- package amount
- deposit
- fee
- service charge

Price categories can drive discount eligibility, refund allocation, totals grouping, and settlement behavior. They should be resolved into order entries so later calculations do not need to re-interpret mutable catalog rules.

This is especially useful when one item has multiple priced parts, such as a catering package with upgrades, included options, required fees, and third-party-funded discounts.

Examples of important typed entry kinds:

- `category`
- `item`
- `modifier`
- `line_discount`
- `order_discount`
- `category_discount`
- `third_party_discount`
- `fee`
- `service_charge`
- `tax`
- `tax_adjustment`
- `tax_paid_by_third_party`
- `payment`
- `refund`
- `allocation`

## Totals

Totals should be calculated from entries, not maintained as unrelated mutable fields.

The totals engine should be generic: it reduces entries into named buckets. The buckets should be strongly defined logical categories.

Examples:

- `gross_sales`
- `net_sales`
- `discounts`
- `fees`
- `service_charges`
- `tax`
- `tax_paid_by_third_party`
- `payments`
- `refunds`
- `amount_due`
- `merchant_payout`

This allows the same order to support multiple total views:

- customer receipt
- cashier closeout
- merchant payout
- marketplace reconciliation
- tax report

Each total should be traceable back to the source entries that produced it. That traceability is what keeps flexible totals from becoming untestable accounting magic.

## Reporting

Reporting should be built from standardized projections, not ad hoc queries over mutable operational state.

Useful report inputs:

- order entries
- transaction entries
- check allocations
- payment allocations
- totals
- events
- state facets
- catalog version references
- surfaces
- fulfillment modes

Reports should group by standardized dimensions such as entry kind, price category, accounting category, tax class, source status, surface, fulfillment mode, merchant, and responsible party.

Historical reports should use versioned or snapshotted facts. A renamed category or edited tax class should not silently rewrite last month's reporting.

Report definitions should be versioned when their logic changes.

## Append-Only Ledger

Financial accounting should be append-only.

The system should append ledger entries for sales, discounts, taxes, payments, refunds, marketplace responsibility, third-party funding, payout adjustments, and corrections.

Existing ledger entries should not be mutated to rewrite history. Corrections should be new entries that reference the original facts.

Ledger entries should include source references back to events, order entries, transaction entries, allocations, and external systems. This gives reports and reconciliation a stable financial truth while preserving the operational order history.

## Coupons

Coupons should be rule-driven generators of discount entries and allocation events.

They should not rewrite item prices directly.

Coupon evaluation should be deterministic over:

- order state
- order entries
- catalog view
- surface
- fulfillment mode
- merchant/customer context
- coupon rule version

Coupon application should produce explicit facts:

- coupon applied or rejected
- eligible entries
- excluded entries
- discount entries
- allocation rules
- funding party
- redemption record

This makes coupons compatible with receipts, refunds, reporting, marketplace funding, and tax handling.

## Checks

Checks are payable groupings over an order.

They should be modeled as allocations over order entries, not as separate mini-orders with copied prices.

Supported grouping modes should include:

- by whole order
- by seat or guest
- by item
- by item quantity
- by amount
- by percentage
- by remaining balance
- by manual adjustment

The same order entry can be split across checks by quantity, amount, or percentage. Derived entries such as tax, whole-order discounts, service charges, and third-party tax responsibility should follow explicit allocation rules.

Payment state should be calculated:

- check total
- minus payment allocations
- plus or minus refunds and adjustments
- equals amount due

This keeps split checks, partial payments, refunds, and settlement reports tied back to the same order facts.

## Entry Transformations

Order entries should support split, merge, and edit operations without losing history.

The architecture should prefer provenance-preserving transformations:

- split by allocation when possible
- merge by view or grouping when exact consolidation would lose meaning
- edit by emitting replacement or change events with option mapping

Every transformation should answer:

- which entry or entries were the source
- which entry or entries are the result
- what quantity, amount, or percentage moved
- which child option entries were kept, added, removed, replaced, or changed
- which state facets carried forward
- which state facets reset

This is especially important for option-heavy items. If a burger with modifiers is edited after being printed or partially paid, the system needs to know which option entries are still the same facts and which ones are new facts.

Merge should be conservative. If entries have different options, discounts, tax treatment, prep state, or payment allocation, merging should usually be a display grouping rather than destructive consolidation.

Edit operations should produce deterministic kitchen, payment, refund, and receipt consequences from the same option mapping.

## State Facets

Some state changes are not whole-order state changes. They belong to individual order lines or operational groupings.

Examples:

- whether a line was printed
- whether a line was made
- whether a line was removed or voided
- how much of a line has been paid
- which bag contains a line
- whether a line has been handed off

The architecture should support generic state facets: independent dimensions of state derived from events and attached to stable IDs such as `OrderLineId`, `BagId`, or `PrintJobId`.

This keeps workflows composable. Payment allocation, prep status, print status, and bag assignment can change independently while still reducing to deterministic order state.

The generic layer should provide:

- stable target IDs
- event ordering
- idempotency
- replay
- typed facet values for important business facts
- validation hooks for facet-specific transition rules

The domain should avoid treating critical state as arbitrary metadata. If a state affects money, fulfillment, customer communication, or audit history, it should have typed events and explicit rules.

## Recommended First Implementation Milestone

Build the core crate without a real database.

Milestone scope:

- domain structs
- catalog tree validator
- modifier validator
- price resolver
- order line snapshotting
- order command handlers
- order event application
- order line state facets
- in-memory repository for tests
- focused unit and property tests

This keeps the first implementation centered on correctness instead of schema churn.
