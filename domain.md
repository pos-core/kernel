# Domain Model

POS Core Kernel should have a small set of concepts with strict behavior. The model should be flexible because concepts compose, not because every feature gets a new entity.

For binding implementation rules, see [rules.md](rules.md). For implemented behavior, use the source-adjacent domain definitions and generated behavior reports linked from the project README. Those focused sources supersede exploratory direction in this file where they conflict.

## Goals

- Minimal vocabulary.
- Explicit invariants.
- Deterministic behavior.
- Distributed client operations.
- Storage-independent business logic.
- No concrete UI, payment processor, surface, order type, or fulfillment implementation in core.
- High testability through pure functions and immutable snapshots.
- Standardized kinds, categories, states, events, and allocation modes.

## Core Concepts

### Catalog

A catalog is a published menu or sellable offering for a brand. It is versioned so clients and orders can refer to a stable view of sellable items.

Draft changes happen outside published versions. Publishing creates a new immutable catalog version and may produce a resolved catalog snapshot for fast client consumption.

### Component

A component is the single catalog building block.

Components replace separate product, category, variant, modifier, and modifier group tables. Their behavior is determined by `kind` plus validated relationships in the catalog tree.

Suggested kinds:

- `catalog`
- `category`
- `item`
- `item_list`
- `modifier_list`
- `modifier`

Open question: whether `variant` deserves its own kind, or whether variants are modeled as child `item` components.

### Modifiers

Modifiers are the catalog-side configuration domain.

The settled shape is:

```text
Modifiers
  Prompt
    Choice
      Modifiers
```

`Modifiers` is a container for related prompts. It can have an optional title, such as "Customize your burger" or "Select your toppings."

`Prompt` is one selectable question with a title and optional description. It owns `min_select`, `max_select`, defaults, and the list of available choices.

`Choice` is one selectable answer with a title. A choice may have effects, collect authored text inputs, and reveal nested modifiers. Choice quantity represents count; a text input may be collected once for the choice selection or once per selected unit.

Effects remain separate from modifiers. A choice can emit effects, but modifiers do not need to know how price, stock, prep, tax, or reporting effects are consumed.

### Identity

Domain entities can have both stable IDs and versioned IDs.

A stable ID identifies the logical thing over time. For example, "large latte" can keep the same `component_id` across edits, publishes, archival, and restoration.

A versioned ID identifies the exact facts that existed at a point in catalog history. For example, "large latte in catalog version 12 with this name, price, tax class, and modifier structure."

This lets asynchronous clients keep working safely:

- new clients can discover the latest catalog version
- old clients can keep using the catalog version they already downloaded
- orders can snapshot the versioned facts they were priced from
- deleted components can disappear from new menus without breaking old orders or old clients
- integrations can map to stable IDs without pretending the facts behind those IDs never change

Suggested references:

- `ComponentId`: stable logical component identity.
- `CatalogVersionId`: immutable published catalog identity.
- `ComponentVersionId`: optional exact component revision identity.
- `ComponentRef`: `{ catalog_version_id, component_id }` when the exact catalog context is enough.
- `EdgeId`: stable or versioned relationship identity, depending on whether edge history needs to be tracked directly.

The domain should avoid raw UUIDs in public APIs. Typed IDs prevent accidentally passing one kind of identity where another is required.

Public IDs should use a standardized prefix plus a sortable unique identifier.

Suggested format:

```text
ORD-01HX7Y9M8N6ZQ4K3V2B1C0D9EA
ENT-01HX7Y9P2V7QK6A1R9J4M3B8CD
CVN-01HX7Y9Q9D4F5H6J7K8M9N0PQR
```

The prefix identifies the entity type. The suffix can be a ULID or UUIDv7-style sortable identifier.

Suggested prefixes:

- `BRD`: brand
- `MCH`: merchant
- `CAT`: catalog
- `CVN`: catalog version
- `CMP`: component
- `CVR`: component version
- `EDG`: catalog edge
- `SUR`: surface
- `FUL`: fulfillment mode
- `ORD`: order
- `ENT`: entry
- `CHK`: check
- `TXN`: transaction
- `PAY`: payment
- `ALC`: allocation
- `BAG`: bag
- `EVT`: event
- `CMD`: command
- `OPR`: client operation
- `USR`: user
- `ACT`: actor
- `ROL`: role
- `PER`: permission
- `CUS`: customer
- `DVC`: device
- `ATR`: consumer attribute
- `LBL`: label
- `MED`: media

IDs should be easy to identify in logs, receipts, webhooks, support tickets, and database rows.

### Consumer Attributes And Profiles

A consumer attribute is a generic, reusable trait that can be attached to surfaces, fulfillment modes, output targets, language preferences, or other catalog consumers.

Examples:

- `web`
- `delivery`
- `spanish`
- `prep`
- `short`
- `receipt`
- `kiosk`

A consumer profile is a compact set of consumer attributes. Labels resolve against consumer profiles, and the same profile concept can later drive visibility, applicability, pricing policy, media choice, or other context-specific catalog behavior.

Labels should not know directly about every first-class concept that contributed to the profile. For example, a web delivery Spanish customer view can resolve labels from:

```text
ConsumerProfile
  web
  delivery
  spanish
  customer
```

The profile keeps label resolution deterministic without turning labels into a rules engine over surfaces, fulfillment modes, locale, printers, and order types.

### Users And Actors

The domain should distinguish users from actors.

A user is a human or account identity. An actor is the identity that performed a command.

Actors can be:

- staff user
- customer user
- service account
- integration
- device
- system process

Commands and events should record actor identity. This gives audit history without forcing every action to come from the same kind of user.

Suggested actor context:

- `actor_id`
- `actor_kind`
- `user_id`
- `merchant_id`
- `brand_id`
- `device_id`
- `session_id`
- `permissions`
- `source_ip`
- `user_agent`
- `integration_id`

Access control should be policy-based and testable.

Policies should consider:

- actor kind
- role
- permission
- brand
- merchant
- surface
- fulfillment mode
- order state
- entry state
- command being attempted

Examples:

- a cashier can add items to an open POS order
- a manager can void paid items with a reason
- a kitchen device can mark lines made but cannot change prices
- a marketplace integration can create external unmapped entries
- a customer can pay their own check but cannot edit staff-only notes

Authorization should happen before command validation emits events. Rejected commands should be auditable without becoming domain events that changed business state.

### Catalog Tree

The catalog tree defines parent-child relationships between components.

The tree is where ordering, inheritance, and contextual overrides are resolved. A component can exist once and appear in multiple places if the tree permits it.

Tree edges may override:

- price
- tax class
- availability
- fulfillment restrictions
- surface restrictions

Tree edges must not point ambiguously across catalog versions. A published tree should resolve to the exact component data for that same published version.

### Surface

A surface is a consumer context for a catalog. It can be a device, channel, workflow, or integration contract.

Examples:

- `pos`
- `phone_order`
- `web`
- `kiosk`
- `app`
- `catering`
- `hub`

Surfaces let one published catalog produce filtered or adjusted views without creating separate catalogs for every channel.

A surface may affect:

- visibility
- ordering
- display names
- media selection
- availability
- fulfillment options
- price adjustments
- tax behavior if required by the business rules

Surface rules should be deterministic and versioned with the catalog. A client consuming the `kiosk` view of catalog version 12 should see the same facts every time that view is regenerated.

The core should define the surface abstraction and validation contracts, not concrete surface behavior. Specific `kiosk`, `web`, `pos`, `phone_order`, or `catering` implementations belong outside the core.

### Fulfillment Mode

A fulfillment mode describes how an order will be completed.

Examples:

- `dine_in`
- `take_out`
- `pickup`
- `curbside`
- `delivery`
- `catering_pickup`
- `catering_delivery`
- `ship`

Fulfillment mode is separate from surface.

For example:

- `web + pickup`
- `web + delivery`
- `pos + dine_in`
- `pos + take_out`
- `phone_order + catering_pickup`
- `kiosk + dine_in`

Fulfillment mode may affect:

- visibility
- availability
- price adjustments
- tax behavior
- prep timing
- lead times
- minimum or maximum quantities
- required customer information
- required order metadata

Allowed surface and fulfillment mode combinations should be explicit. A catalog view should not silently invent behavior for an unsupported combination.

The core should define fulfillment mode as an input and policy axis, not implement concrete fulfillment workflows. Delivery dispatch, table service, catering lead-time behavior, shipping, and pickup logistics belong outside the core.

### Price Category

A price category is a logical pricing bucket for an item or price component.

It is distinct from:

- catalog category: where the item appears in the menu
- tax class: how tax is calculated
- accounting category: how revenue is reported

Price categories can help answer questions such as:

- Is this amount discountable?
- Is this amount part of the base item, a modifier, an upgrade, a deposit, or a fee?
- Which discounts can apply to it?
- Which totals bucket should include it?
- Does a surface or fulfillment mode adjust it?
- How should partial refunds allocate against it?

Suggested price categories:

- `base_item`
- `modifier`
- `upgrade`
- `included`
- `deposit`
- `package`
- `fee`
- `service_charge`

An item may have a base price category, and its child entries may have their own price categories. For example, a catering package can include base package pricing, optional upgrades, excluded service fees, and third-party-funded discounts without flattening all money into one item price.

Price category should be explicit on priced entries once an order is created. This makes totals, discounts, refunds, and settlement reproducible even if catalog rules change later.

### Catalog View

A catalog view is a resolved projection of a published catalog for a specific context.

Suggested context:

- `catalog_version_id`
- `surface_id`
- optional `fulfillment_mode_id`
- optional `merchant_id`
- optional locale

Catalog views are read models. They do not replace the underlying catalog version.

This gives the system one source catalog with many safe projections:

- POS can show operational names or staff-only items.
- Phone order entry can expose guided prompts and staff-facing search terms.
- Web can hide unavailable or in-store-only items.
- Kiosk can require images and short names.
- Catering can expose packages, lead times, and quantity rules that do not belong on the normal menu.
- Delivery can apply fulfillment-specific pricing or availability.
- Merchant-specific views can apply local stock and 86ing outside the catalog version.

### Order

An order is a transaction. It does not point live at catalog data for mutable business facts.

Order line items are snapshots. Names, prices, taxes, selected modifiers, and calculated totals are copied onto the order when the item is added or priced.

Orders should be built from generic, descriptive entries.

An order entry is a durable row of order meaning. It can represent a sold item, modifier, category/grouping row, discount, fee, service charge, tip, tax, tax adjustment, note, or operational grouping.

Suggested order entry fields:

- `entry_id`
- `parent_entry_id`
- `kind`
- `source_ref`
- `source_status`
- `description`
- `quantity`
- `unit_amount`
- `total_amount`
- `price_category`
- `tax_class_id`
- `accounting_category`
- `state`
- `metadata`

The entry is generic because many order facts share the same shape. It is descriptive because it snapshots the human-readable meaning at the time of the order.

For catalog-backed entries, `source_ref` should include the catalog version and component identity used to create the entry. For manual or operational entries, `source_ref` can identify the command, integration, or user action that created it.

Entries do not have to map to the catalog.

Suggested source statuses:

- `catalog_backed`: created from a known catalog version and component.
- `external_mapped`: created from an external source and mapped to a catalog component.
- `external_unmapped`: created from an external source with no catalog mapping.
- `manual`: created directly by a user or operator.
- `system`: created by domain logic, such as tax, allocation, or adjustment entries.

An external unmapped entry is snapshot-authoritative. Its description, amount, taxes, and metadata come from the external source or import command. The domain can validate arithmetic and state transitions, but it should not pretend the entry has catalog availability, modifier rules, or price rules.

If an external entry is mapped later, the mapping should be recorded as an event. Existing order facts should remain explainable from the original external snapshot.

Discounts should be typed by scope.

Examples:

- `line_discount`: applies to one order entry.
- `order_discount`: applies to the whole order or a subtotal group.
- `category_discount`: applies to entries in a logical category.
- `third_party_discount`: funded or controlled by a marketplace, platform, or integration.

The discount kind should determine allocation and validation rules. For example, a whole order discount may need to allocate across eligible item entries for tax, refund, and reporting purposes.

Item discounts need explicit rules.

An item discount should define:

- whether it applies to the base item only
- whether it applies to selected modifiers
- which price categories are eligible
- whether it applies to one unit or all units on the line
- whether it is a fixed amount or percentage
- whether it applies before or after tax
- whether it can stack with other discounts
- whether it is merchant-funded, platform-funded, or third-party-funded
- how it allocates when an item is partially refunded

Avoid hiding item discounts by mutating the item price. The original item price and the discount entry should both remain explainable.

Entry hierarchy replaces special-case modifier storage:

- category or grouping entry
- item entry
- child modifier entry
- child discount entry
- child fee entry
- child tax entry
- child note entry

The hierarchy should be validatable. For example, a modifier entry should not exist without a sellable parent item unless that behavior is explicitly allowed.

Order entry creation should go through shared deterministic builders.

The same builder should be usable by clients and servers. Given the same catalog view, actor context, surface, fulfillment mode, and command input, it should produce the same entries, option mappings, prices, totals, and validation errors.

This prevents offline/local client orders from drifting away from server behavior.

### Transaction

A transaction records financial movement related to an order.

Transactions should also be built from generic, descriptive entries.

A transaction entry can represent:

- payment authorization
- payment capture
- cash tender
- gift card tender
- refund
- void
- tax collected
- tax liability
- tax paid by third party
- tip
- service charge
- fee
- change due
- payment allocation to order entries

Suggested transaction entry fields:

- `entry_id`
- `parent_entry_id`
- `kind`
- `source_ref`
- `description`
- `amount`
- `currency`
- `tender_type`
- `status`
- `responsible_party`
- `allocated_order_entry_ids`
- `metadata`

Transactions and orders should be related by allocation, not by overwriting order prices. For example, a single payment can allocate money across multiple order entries, and a partial refund can target specific paid entries.

This keeps money movement auditable while allowing order entries to preserve what was sold.

Tax responsibility should be explicit.

For example, a marketplace order may include tax on the customer receipt while the marketplace is responsible for collecting or remitting that tax. That should be represented as a transaction or settlement fact such as `tax_paid_by_third_party`, not hidden as a discount or removed from the order total.

Suggested responsible parties:

- `customer`
- `merchant`
- `third_party`
- `platform`
- `marketplace`

This lets receipts, payouts, settlement reports, and tax reports explain the same order from different angles without changing the underlying order entries.

### Totals

Totals should be generic calculations over entries with solid logical categories.

A total is a named amount derived from order entries, transaction entries, or both.

Suggested total fields:

- `total_id`
- `category`
- `amount`
- `currency`
- `source_entry_ids`
- `responsible_party`
- `description`

Suggested logical categories:

- `gross_sales`
- `net_sales`
- `item_sales`
- `discounts`
- `fees`
- `service_charges`
- `tips`
- `tax`
- `tax_paid_by_third_party`
- `payments`
- `refunds`
- `amount_due`
- `merchant_payout`

Totals should be reproducible from entries. They should not become independent mutable facts unless a later event records an explicit adjustment.

Different consumers can request different total views:

- customer receipt totals
- cashier settlement totals
- merchant payout totals
- tax reporting totals
- third-party marketplace reconciliation totals

The same entries should be able to explain all of those views. The view changes the grouping and labels, not the underlying facts.

### Reporting

Reporting should be a projection over standardized facts.

Reports should derive from:

- order entries
- transaction entries
- allocations
- totals
- events
- state facets
- catalog version references
- surfaces
- fulfillment modes
- merchant and brand IDs

Reports should not reinterpret mutable catalog data without a versioned reference. If a historical report groups sales by catalog category, price category, tax class, or accounting category, it should use the categories snapshotted or referenced at the time of the order.

Important reporting dimensions:

- time
- brand
- merchant
- surface
- fulfillment mode
- catalog version
- component ID
- entry kind
- price category
- accounting category
- tax class
- responsible party
- payment/tender type
- source status

Reports should be reproducible. If a report changes because reporting logic changed, that should be a versioned report definition change, not accidental data drift.

### Accounting Ledger

Accounting should be append-only.

Orders, transactions, payments, refunds, tax responsibility, third-party funding, payouts, and adjustments should produce ledger entries. Existing ledger entries should not be edited to change financial history.

Corrections should be represented by new entries:

- reversal
- refund
- adjustment
- reclassification
- payout correction
- tax correction

Suggested ledger entry fields:

- `ledger_entry_id`
- `source_event_id`
- `source_entry_id`
- `kind`
- `amount`
- `currency`
- `accounting_category`
- `responsible_party`
- `merchant_id`
- `occurred_at`
- `recorded_at`
- `reversal_of`
- `metadata`

Ledger entries should be traceable back to the order, transaction, event, entry, allocation, or external source that produced them.

Reports can read directly from entries for operational views, but financial reporting should prefer ledger projections when correctness, auditability, and reconciliation matter.

### Coupons

Coupons are standardized pricing rules that produce discount entries.

A coupon should not mutate item prices directly. Applying a coupon should emit events and create one or more discount entries or discount allocations.

Suggested coupon fields:

- `coupon_id`
- `code`
- `kind`
- `scope`
- `funding_party`
- `eligibility_rules`
- `discount_rule`
- `stacking_policy`
- `redemption_limits`
- `validity_window`
- `surface_ids`
- `fulfillment_mode_ids`
- `merchant_ids`
- `metadata`

Coupon scopes:

- item
- category
- price category
- whole order
- fulfillment mode
- surface
- customer/account

Coupon rules should be deterministic against an order snapshot and catalog view. The same coupon applied to the same order state should produce the same discount entries.

Coupon application should record:

- coupon identity
- rule version
- eligible entries
- excluded entries
- created discount entries
- allocation method
- funding party
- reason if rejected

This makes coupons reportable, refundable, auditable, and explainable on receipts.

### Checks

A check is a payable grouping of part of an order.

The order remains the complete source of truth. Checks describe which portions of the order are to be paid together or have already been paid together.

Checks can be grouped by:

- whole order
- seat
- guest
- item
- item quantity
- fixed amount
- percentage
- remaining balance
- manual adjustment

Suggested check fields:

- `check_id`
- `order_id`
- `label`
- `status`
- `allocations`

Suggested check allocation fields:

- `allocation_id`
- `check_id`
- `order_entry_id`
- `mode`
- `quantity`
- `amount`
- `percentage`
- `priority`

Allocation modes:

- `entire_entry`: the full entry belongs to the check.
- `quantity`: a quantity portion of the entry belongs to the check.
- `amount`: a fixed amount of the entry belongs to the check.
- `percentage`: a percentage of the entry belongs to the check.
- `remainder`: whatever remains after higher-priority allocations belongs to the check.
- `derived`: generated from allocation rules, such as tax following taxable items.

Checks should not copy prices as their source of truth. Check totals should be derived from order entries plus check allocations.

Discounts, tax, fees, service charges, and tips need explicit allocation rules. For example:

- tax can follow taxable entries
- whole order discounts can allocate proportionally across eligible entries
- service charges can allocate by subtotal, guest count, or explicit amount
- third-party-paid tax can appear in tax reporting without increasing customer amount due

Payments should allocate to checks, order entries, or both. Paid state should be derived from payment allocations, refunds, and check totals.

### Entry Transformations

Order entries may need to be split, merged, and edited after creation.

These operations should preserve provenance. The system should be able to explain what the entry used to be, what it became, and which options or child entries carried forward.

#### Split

An entry can be split multiple ways depending on the workflow.

Examples:

- split by check
- split by seat
- split by quantity
- split by amount
- split by preparation station
- split by bag or handoff group
- split by refund target

Splitting should usually be represented as allocations over a stable entry, not destructive duplication.

If a true entry split is needed, the new entries should record:

- original entry ID
- split reason
- split mode
- quantity or amount moved
- source child entries
- state facets that carried forward

#### Merge

Entries may be merged for display, kitchen handling, check simplification, or operational convenience.

Merging should not erase original entry identity.

Safe merge behavior should require compatible:

- catalog source
- selected options
- price category
- tax class
- discount rules
- prep state
- payment allocation state

If compatibility is not exact, merging should be a view or grouping entry rather than a destructive consolidation.

Merged entries or merge views should retain `source_entry_ids` so receipts, refunds, kitchen events, and audit trails can still point back to the original facts.

#### Edit

Editing an item should be modeled as a deterministic transformation, not silent mutation.

Examples:

- change quantity
- add an option
- remove an option
- replace one option with another
- change size
- apply or remove an item discount
- move the item to another seat or check

Edits should preserve option mapping.

When an item with child option entries is edited, the command should produce a mapping from previous child entries to new child entries:

- `kept`: same option carried forward
- `added`: new option entry
- `removed`: old option entry removed or voided
- `replaced`: old option mapped to a new option
- `changed`: same logical option with changed quantity, price, or state

This mapping matters for:

- kitchen make/void diffs
- reprints
- refunds
- payment allocation preservation
- bag and prep state carry-forward
- audit history
- customer-facing explanations

State carry-forward should be explicit. For example, payment allocation may carry forward proportionally, while printed or made state may reset or require a remake event depending on the edit.

Order behavior should be command and event driven.

A command expresses intent, such as:

- `open_order`
- `add_item`
- `remove_item`
- `change_quantity`
- `apply_discount`
- `submit_order`
- `mark_paid`
- `void_order`
- `mark_fulfilled`

The domain validates the command against the current order state and emits deterministic events.

Events describe what happened, such as:

- `order_opened`
- `item_added`
- `quantity_changed`
- `discount_applied`
- `order_submitted`
- `payment_recorded`
- `order_voided`
- `order_fulfilled`

The current order state should be reproducible by applying its event stream in order. This makes order actions testable, auditable, and safe to replay.

### Order Line State

Order lines need operational state beyond the catalog snapshot.

Examples:

- printed
- made
- removed or voided
- paid amount
- payment allocation
- bag assignment
- prep station assignment
- handoff status

This should be modeled as state facets on stable order line IDs.

A facet is one independent dimension of state. For example, preparation, printing, payment allocation, and bag assignment are separate facets because they change for different reasons and may be controlled by different workflows.

Suggested facets:

- `lifecycle`: active, removed, voided, fulfilled
- `print`: not printed, printed, reprint requested
- `prep`: not started, in progress, made, remade
- `payment`: unpaid, partially paid, paid, refunded
- `bag`: unassigned or assigned to a bag ID
- `handoff`: waiting, ready, handed off

Facet state should be derived from events:

- `line_printed`
- `line_made`
- `line_voided`
- `line_payment_allocated`
- `line_assigned_to_bag`
- `line_handed_off`

The mechanism can be generic, but important facets should still have typed values and explicit transition rules. Avoid using untyped arbitrary state for facts that affect money, fulfillment, auditability, or customer promises.

## Invariants

These rules should be enforced by the domain layer and covered by tests.

### Standardization

Standardization is a core correctness tool.

The system should define controlled vocabularies for:

- component kinds
- entry kinds
- price categories
- accounting categories
- tax responsibility categories
- total categories
- surface IDs
- fulfillment mode IDs
- allocation modes
- state facet names and values
- command names
- event names
- reason codes

Extensions should be possible, but extension points must be explicit. Unknown or custom values should not silently behave like built-in values.

Each standard value should define:

- name
- meaning
- allowed relationships
- validation rules
- effect on totals
- effect on receipts and reporting
- whether it can be extended or customized

This makes the generic model safe. Entries, allocations, totals, and facets can be flexible because their logical categories are standardized and testable.

### Naming

- Public field names use `snake_case`.
- Reserved words are avoided. Use `kind`, `sort_order`, and `position` instead of `type` or `order`.
- Money is stored as integer minor units.
- Percent rates should not be floats unless there is a deliberate rounding policy.

### Versioning

- Published catalog versions are immutable.
- Drafts may change freely.
- Soft deletion marks a draft or versioned row as deleted.
- Deletion of a stable entity must not invalidate versioned references that already exist.
- Pruning must never break an order, audit log, or client pinned to an old catalog version.
- A tree edge in one catalog version must not resolve to component data from another version.

### Events

- Commands validate intent.
- Events record facts.
- Event application must be deterministic.
- Replaying the same events must produce the same state.
- Events should contain enough snapshot data to remain meaningful after catalog changes.
- Invalid commands should not emit events.
- Event IDs and idempotency keys should make retries safe.
- Order line events should target stable `OrderLineId` values.
- Monetary line state should be represented as payment allocations, not only a paid/unpaid boolean.
- Operational state should use independent facets when workflows can change independently.
- Order and transaction entries should be append-friendly and descriptive enough to explain receipts, audits, and replays.

### Distributed Client Operations

Client operations can be distributed.

A client should be able to create an order, add entries, split checks, and record local operational state before those facts reach a central database.

This requires client-generated IDs and an append-only local operation log.

Suggested client operation fields:

- `operation_id`
- `device_id`
- `actor_id`
- `order_id`
- `idempotency_key`
- `client_sequence`
- `observed_catalog_version_id`
- `surface_id`
- `fulfillment_mode_id`
- `command`
- `events`
- `occurred_at`
- `synced_at`
- `sync_status`

Suggested sync statuses:

- `local`
- `queued`
- `submitted`
- `accepted`
- `rejected`
- `needs_resolution`

The domain should separate local deterministic work from remote confirmation.

Local deterministic work:

- create order IDs
- add catalog-backed entries from a downloaded catalog view
- add external unmapped entries
- calculate local totals
- split checks
- assign bags
- mark printed or made

Remote confirmation may still be required for:

- payment authorization
- inventory reservation
- loyalty redemption
- coupon redemption limits
- marketplace acceptance
- final settlement

Events created offline should include enough context to be replayed and audited later. Sync should be idempotent so retrying the same operation does not duplicate order entries, payments, or state changes.

### Composition

Allowed relationships should be deliberately small.

- `catalog` may contain `category`, `item_list`, or `item`.
- `category` may contain `category`, `item_list`, or `item`.
- `item_list` may contain `item`.
- `item` may contain `modifier_list`.
- `modifier_list` may contain `modifier`.
- `modifier` may contain nothing unless nested modifiers are explicitly supported later.

The domain layer must reject:

- cycles
- invalid parent-child kind pairs
- duplicate sibling positions where uniqueness is required
- modifier defaults that are not children of the modifier list
- `min_select` greater than `max_select`
- required selections with no valid choices

### Pricing

Pricing should be deterministic and explainable.

Suggested precedence:

1. Explicit line override.
2. Tree edge override.
3. Component base price.
4. Inherited parent rule.

Open question: whether inherited parent price rules should exist at all. They add power, but they can make correctness harder to reason about.

### Availability

Availability is resolved from catalog rules plus live location state.

Catalog availability answers whether something is generally sellable in a context. Live stock answers whether a specific merchant or location can fulfill it right now.

Live stock and 86ing should stay outside published catalog versions.

### Tax

Tax class can be attached to a component or overridden by a tree edge.

Tax calculation should be a pure, testable operation over a priced order snapshot and a tax policy input.

## Test Strategy

The project should bias toward pure domain tests before persistence tests.

Core test families:

- composition validation
- catalog version immutability
- tree resolution
- surface and fulfillment filtering
- price precedence
- price category resolution
- item discount allocation
- coupon eligibility and allocation
- reporting projections
- modifier min/max/default rules
- order snapshot stability after catalog edits
- order command validation
- event replay determinism
- order line state facet transitions
- line-level payment allocation invariants
- stock behavior isolated from catalog publishing
- serialization compatibility for public DTOs

Property tests are a good fit for tree validation, cycle detection, price resolution, and order total invariants.
