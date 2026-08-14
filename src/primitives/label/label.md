# Label

A label is an authored text definition with a required default value and optional consumer-profile-specific values.

Translations use those same profile-specific values. Language or locale is represented by a consumer attribute, so it may combine with other attributes to define text for a particular consumer, such as Spanish on a KDS.

Resolution chooses the most-specific value whose required profile is satisfied, or the default when none match. When equally specific values match, the active consumer profile's attribute order supplies precedence; label definition order remains irrelevant.

A label may omit its ID for manually entered text. Absence of a label is represented by `Option<Label>`, not by an empty label.
