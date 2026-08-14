# Media

Media defines one default media representation and optional alternatives selected by consumer profile.

Each representation identifies the media and MIME type and may carry a label and dimensions. Resolution chooses the most-specific matching variant or falls back to the default. When equally specific variants match, the active consumer profile's attribute order supplies precedence.

A media collection preserves definition order and prevents duplicate default media IDs.
