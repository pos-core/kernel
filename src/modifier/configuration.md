# Configuration

A configuration is the validated, resolved state produced by hydrating modifier definitions with selections and evaluation context.

It records the prompts and choices that apply, resolved labels, quantities, explicit-or-default selection sources, effects, and nested configurations. It can calculate modifier pricing or dehydrate back into effective selections.

Hydration moves from authored definitions plus selection input to this resolved state. Dehydration extracts the effective selection structure from that state.
