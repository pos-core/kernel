# Selections

Selections are the structured selection payload supplied to modifier hydration.

They identify prompt occurrences and selected choices, including quantities, selection source, and optional nested selections. Input may be sparse: omitted choices can still be supplied by defaults in the modifier definitions.

Dehydrating a configuration produces effective selections that include the choices actually applied, including defaults and nested selections. Selections do not contain resolved labels or calculated price contributions.
