# Variant Dimension

A variant dimension is an ordered catalog grouping for related variant values, such as `Size` or `Crust`.

It owns a stable dimension ID, a required label, and one or more variants. The dimension's position in the catalog item's dimension vector controls selection order, selection dependency, and combined-label order.

A concrete match may stop before reaching a later dimension, but it cannot skip an earlier dimension and then select from a later one. Concrete matches implicitly define which values can follow each partial path, so dimensions do not own a separate hierarchy or dependency tree.
