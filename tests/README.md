# Behavior Test Layout

Every domain behavior test has one canonical definition under `tests/behavior_reports/`.

Each behavior-report module also declares links to the domain definitions a reader needs for that report. The definition Markdown lives beside the owning production code and is included in the owning type's Rustdoc; tests link to it but do not duplicate it.

Each behavior consists of three colocated parts:

1. A `DescribedBehavior` constant containing the report name, plain-language description, and assertion function.
2. The corresponding `#[test]` assertion function immediately below the constant.
3. One reference to the constant in that module's `report()` case list.

Example:

```rust
pub const CONCRETE_MATCHES_CANNOT_OVERLAP: DescribedBehavior = DescribedBehavior::new(
    "concrete matches cannot overlap as prefixes",
    "Every match is a priced concrete leaf...",
    concrete_matches_cannot_overlap,
);

#[test]
fn concrete_matches_cannot_overlap() {
    // Assertions define the behavior contract.
}
```

This same assertion function runs as an ordinary Rust test and as a Markdown report case. Do not duplicate an assertion body in a production module, another integration test, or report-only wrapper.

Keep each behavior-report module in this order:

1. Imports.
2. `report()` with cases in reading order.
3. Described behavior constants and their test functions in the same order.
4. Non-test fixture builders and helper functions.

The module's `report()` lists definitions before cases:

```rust
ModuleReport {
    slug: "modifier",
    title: "Modifier",
    description: "...",
    definitions: vec![DefinitionLink::new(
        "Configuration snapshot",
        "../src/modifier/configuration-snapshot.md",
    )],
    cases: vec![CONFIGURATION_SNAPSHOT_PRESERVES_PRICE_FACTS.report_case()],
}
```

Definition links are relative to the generated file under `test-coverage/`. Add a definition link when a report uses a domain term whose meaning or boundary is not obvious from ordinary language.

Behavior names and descriptions must use the precise defined term. For example, say `configuration snapshot`, `order-item modifier snapshot`, or `effective selections`; do not use bare `snapshot` when more than one snapshot concept exists.

`tests/support/` is only for shared report infrastructure and data-building helpers. It must not own domain behavior assertions or their descriptions.

To add a new behavior module:

1. Create `tests/behavior_reports/<module>.rs`.
2. Export it from `tests/behavior_reports/mod.rs`.
3. Add its `report()` to `tests/behavior_report.rs`.

To add a new domain definition:

1. Create a lowercase kebab-case Markdown file beside the owning Rust source.
2. Attach it to the owning type with `#[doc = include_str!("definition-name.md")]`.
3. Add a `DefinitionLink` to every behavior report that relies on the term.

Run `cargo test`. The ordinary test harness executes every described behavior, and `writes_behavior_markdown_reports` regenerates `test-coverage/`. Files under `test-coverage/` are generated outputs and should not be edited manually.
