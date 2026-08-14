# Order Entry

An order entry is an atomic priced line used by an order and its derived totals.

It records a stable entry ID, kind, source, description, quantity, unit amount, calculated total amount, and optional pricing or accounting categories. The source preserves whether the line came from catalog data, an external system, or manual entry.

An order item expands into entries so base-item and modifier amounts remain independently attributable.
