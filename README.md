# POS Core Kernel

A flexible, correctness-focused distributed POS domain library.

The project is currently in the design stage. The goal is to keep the core model small, composable, distributed, and highly testable before committing to database schema or application shape.

POS Core Kernel should ship with a client library. POS terminals, kiosks, phone order tools, web ordering, and integrations should be able to create deterministic local operations from a downloaded catalog view, then sync those operations later.

## Current Docs

- [domain.md](domain.md): core concepts, invariants, and test strategy.
- [architecture.md](architecture.md): layering, publishing boundary, and first implementation milestone.
- [rules.md](rules.md): hard project rules for core boundaries, money, time, identity, and tests.
- [catalog-item-configuration.md](catalog-item-configuration.md): current variant, field, modifier, and pricing design target.
- [spec.md](spec.md): earlier product/domain sketch.
- [notes.md](notes.md): raw relational schema and versioning notes.

## Current Direction

POS Core Kernel is a Rust library crate with pure domain logic and a client-safe operation model. Persistence, APIs, sync, hardware, and payments should be adapters around the core rather than part of the core model.

The Cargo package is named `pos-core-kernel` and is imported in Rust code as
`pos_core_kernel`.

## Rust Layout

The crate uses prefix-oriented module names so domain entities and their public IDs stay visually connected.

Examples:

- `ids/ord_order_id.rs`: `ORD-...`
- `ids/ent_entry_id.rs`: `ENT-...`
- `ids/itm_catalog_item_id.rs`: `ITM-...`
- `ids/var_variant_id.rs`: `VAR-...`
- `catalog_item/itm_catalog_item.rs`: variant dimensions, matches, and configured catalog items
- `order/ord_order.rs`: order aggregate and order events
- `entry/ent_entry.rs`: generic descriptive order entries
- `modifier/mod_definition.rs`: `Modifiers -> Prompt -> Choice -> Modifiers`
- `event/evt_event.rs`: event envelope
- `totals/ttl_total.rs`: derived totals

The core should remain free of UI, storage, concrete payment processors, concrete surfaces, concrete order types, and concrete fulfillment behavior.

## Domain Definitions

Important domain terms have standalone Markdown definitions beside the Rust code that owns them. The owning type includes the same file as its Rust documentation:

```rust
#[doc = include_str!("configuration-snapshot.md")]
pub struct ConfigurationSnapshot {
    // ...
}
```

This keeps the Markdown page, generated Rustdoc, and behavior-report definition links anchored to one source of truth. Definitions explain what a concept means, what it contains, and how it differs from adjacent concepts; behavior tests define the rules it must satisfy.

## Test Reports

`cargo test` runs each described behavior as an ordinary Rust test and writes the same behaviors to Markdown under `test-coverage/`. Each module report links the domain definitions needed to read its behavior descriptions. The canonical test layout is documented in [`tests/README.md`](tests/README.md).

- `test-coverage/index.md`: all module reports
- `tests/behavior_reports/`: colocated behavior descriptions and assertion functions

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
