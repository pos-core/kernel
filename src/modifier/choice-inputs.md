# Choice Inputs

Choice inputs are authored text settings attached to a modifier choice. They collect text that configures the selected choice without representing another choice or changing its quantity.

Each setting has a component ID, label, required flag, optional minimum and maximum character lengths, and a `repeat_per_quantity` flag. Length is counted in Unicode scalar values.

When `repeat_per_quantity` is false, the input is collected at most once for the whole choice selection. When it is true, each supplied value identifies a one-based unit number within the selected quantity. A required repeated input must have exactly one value for every selected unit; an optional repeated input may have values for any subset of those units.

For example, a cupcake choice selected with quantity three can require a name for units one, two, and three. A special-request input on the same choice can be collected once regardless of quantity.

Hydration validates input identity, occurrence, required values, and length constraints. The resulting configuration, effective selections, configuration snapshot, and order-item modifier snapshot preserve the entered text and the authored input label.
