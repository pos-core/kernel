# Label

A label is an authored text definition with a required default value and optional consumer-profile-specific values.

Resolution chooses the single most-specific value whose required profile is satisfied, or the default when none match. Equal-specificity matches are rejected as ambiguous instead of depending on definition order.

A label may omit its ID for manually entered text. Absence of a label is represented by `Option<Label>`, not by an empty label.
