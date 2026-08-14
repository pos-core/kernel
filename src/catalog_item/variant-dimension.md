# Variant Dimension

A variant dimension is an ordered catalog grouping for related variant values, such as `Size` or `Crust`.

It owns a stable dimension ID, a required label, and one or more variants. The dimension's position in the catalog item's dimension vector controls selection presentation and combined-label order. Position does not require every concrete match to use the dimension and does not itself create parent-child hierarchy between dimensions.
