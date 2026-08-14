# Consumer Profile

A consumer profile is an ordered list of unique consumer-attribute IDs describing the audience for which a value is being resolved. Earlier attributes have higher precedence than later attributes.

Labels and media variants use profiles as matching requirements. A candidate matches when the active profile contains every attribute it requires; the order of those requirements does not affect matching. A candidate with more required attributes is more specific.

When equally specific candidates match, the active profile order breaks the tie. The candidate containing the earliest higher-precedence attribute that differs between them wins. Duplicate attributes are rejected.
