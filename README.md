# POS Core Kernel

A correctness-focused Rust domain library for building POS systems that behave consistently across servers and distributed clients.

The project is in early development. The goal is to keep the core model small, composable, storage-independent, and highly testable before committing to database schema or application shape.

POS Core Kernel should ship with a client library. POS terminals, kiosks, phone order tools, web ordering, and integrations should be able to create deterministic local operations from a downloaded catalog view, then sync those operations later.

## Documentation

- [Domain direction](domain.md): long-range domain vocabulary, goals, and open design direction.
- [Project rules](rules.md): binding boundaries and invariants for the current implementation.
- [Testing guide](tests/README.md): the canonical behavior-test layout and documentation workflow.
- [Behavior index](test-coverage/index.md): generated, readable reports of the behavior currently covered by tests.

Implemented concepts are defined in Markdown beside the Rust code that owns them and linked from the behavior reports. Those definitions and reports are the source of truth for current behavior.

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

## Documentation Model

Important domain terms have standalone Markdown definitions beside the Rust code that owns them. The owning type includes the same file as its Rust documentation:

```rust
#[doc = include_str!("configuration-snapshot.md")]
pub struct ConfigurationSnapshot {
    // ...
}
```

This keeps each Markdown page, generated Rustdoc, and behavior-report definition link anchored to one source of truth. Definitions explain what a concept means and where its boundary lies; behavior tests define the rules it must satisfy.

`cargo test` runs each described behavior as an ordinary Rust test and regenerates the Markdown reports under `test-coverage/`. Behavior descriptions and their assertion functions stay together under `tests/behavior_reports/`.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
