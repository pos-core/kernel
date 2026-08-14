# Consumer Profile

A consumer profile is a set of consumer-attribute IDs describing the audience for which a value is being resolved.

Labels and media variants use profiles as matching requirements. A candidate matches when the active profile contains every attribute it requires; a candidate with more required attributes is more specific.

The profile is set-like: attribute order is irrelevant and duplicate attributes are rejected.
