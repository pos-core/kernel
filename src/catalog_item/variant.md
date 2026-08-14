# Variant

A variant is one labeled value within a variant dimension, such as `Small`, `Thin`, or `Blue`.

It owns a stable variant ID, a required label, an optional description label, and optional media. Price, effects, modifier applicability, default selection, and exact-combination metadata do not belong to this value; they belong to authored variant matches because they may depend on a combination of values.

Descriptions and media are catalog presentation metadata. They are not included in configuration or order snapshots by default. The kernel does not apply fallback between variant media and catalog-item media; clients decide presentation precedence, combination, and fallback.

When a concrete selection contains values from multiple dimensions, their labels remain separate authored definitions and are resolved in dimension order. By default, a combined display label is derived from those resolved values. A variant match may optionally provide an exact label without erasing the component labels as structured facts.
