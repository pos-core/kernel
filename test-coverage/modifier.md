# Modifier

Described behavior tests for prompts, choices, choice inputs, rules, hydration, dehydration, and pricing.

## Definitions

- [Modifier definitions](../src/modifier/modifiers.md)
- [Choice inputs](../src/modifier/choice-inputs.md)
- [Selections](../src/modifier/selections.md)
- [Configuration](../src/modifier/configuration.md)
- [Configuration snapshot](../src/modifier/configuration-snapshot.md)
- [Modifier pricing](../src/modifier/modifier-pricing.md)

## Result

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 37
- Passed: 37
- Failed: 0

## Behaviors

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| defaults dehydrate into effective selections | Default choices hydrate into configuration and dehydrate into effective selections with their default source preserved. | Passed | 0 ms |
| configuration snapshot preserves default selections labels and price facts | A configuration snapshot includes default selections, prompt and choice labels, price definitions, price contributions, and the modifier total. | Passed | 0 ms |
| defaults must satisfy prompt min and max rules | Choice defaults are validated against the containing prompt selection count rules. | Passed | 0 ms |
| explicit selections replace defaults for a prompt | An explicit selection suppresses default choices on the same prompt instead of merging with them. | Passed | 0 ms |
| prompt min and max rules validate selection counts | Prompt Min and Max rules validate the summed selected counts across choices. | Passed | 0 ms |
| max zero prompt allows no selection and rejects any selection | A prompt with Max(0) is valid when empty and invalid when any choice is selected. | Passed | 0 ms |
| optional prompt without selection hydrates empty configuration | An optional prompt with no selected choices still appears in hydrated configuration with an empty choice list. | Passed | 0 ms |
| scheduled choice requires evaluation time when selected | A selected choice with a schedule cannot be hydrated unless the caller supplies explicit EvaluationTime. | Passed | 0 ms |
| scheduled choice hydrates only inside its schedule | A scheduled choice accepts selections inside its own schedule and rejects selections outside it. | Passed | 0 ms |
| scheduled choice is not visible outside its schedule | A scheduled choice is filtered from visible modifier traversal when EvaluationTime falls outside its schedule. | Passed | 0 ms |
| scheduled default choice uses supplied evaluation time | A scheduled default choice requires EvaluationTime and only defaults when its schedule includes that time. | Passed | 0 ms |
| choice rules apply to selected choice quantity | Choice-level Min and Max rules validate the selected count on that choice. | Passed | 0 ms |
| choice inputs collect once or once per selected unit and round trip | A selected choice can collect one text value for the whole selection and ordered text values for individual units, preserving them through hydration, effective selections, and configuration snapshots. | Passed | 0 ms |
| choice inputs reject invalid definitions and values | Choice input authoring requires labels, coherent length bounds, and unique IDs; hydration enforces known inputs, once-versus-per-unit occurrence, required values, and character-length limits. | Passed | 0 ms |
| choice default rules are unique nonzero and within choice bounds | A choice default must be unique, greater than zero, and valid against the choice's own Min and Max rules. | Passed | 0 ms |
| duplicate choice selections are rejected even with different nested selections | A prompt cannot select the same choice ID twice; distinct configuration must be modeled below the selected choice. | Passed | 0 ms |
| prompt rejects zero duplicate and unknown choice selections | Prompt validation rejects zero counts, repeated choice IDs, and choice IDs that do not exist under the prompt. | Passed | 0 ms |
| prompt rejects selection count overflow | Prompt validation uses checked arithmetic while summing selected counts. | Passed | 0 ms |
| duplicate min or max rules are rejected | Each rule kind can appear at most once in a prompt or choice rule set. | Passed | 0 ms |
| definitions reject empty titles invalid prompt rules and empty required prompts | Definition construction validates prompt and choice titles, rejects prompt defaults, and rejects required prompts with no choices. | Passed | 0 ms |
| definitions reject invalid min max constraints and duplicate choice definitions | Definition construction rejects Min greater than Max and duplicate choice IDs inside a prompt definition. | Passed | 0 ms |
| configuration dehydrates nested choices and round trips | Explicit nested choices survive hydrate, dehydrate, and rehydrate without changing configuration. | Passed | 0 ms |
| three level nested selection dehydrates and round trips | Deeply nested explicit selections remain stable across hydrate and dehydrate. | Passed | 0 ms |
| nested required prompts must be satisfied when parent choice is selected | Selecting a choice with required nested modifiers validates the nested required prompt immediately. | Passed | 0 ms |
| nested defaults dehydrate into effective selections | Nested defaults under an explicit parent choice are included in dehydrated effective selections with their default source preserved. | Passed | 0 ms |
| same modifiers can hold repeated prompt IDs as ordered instances | Repeated prompt IDs in one modifier container hydrate by occurrence and dehydrate back as ordered prompt instances. | Passed | 0 ms |
| same prompt and choice IDs can be reused in different branches | Prompt and choice IDs can recur in separate branches because nested structure scopes selection lookup. | Passed | 0 ms |
| strict hydrate rejects unknown prompt selections and extra prompt occurrences | Strict hydration rejects prompt selections that are not present at the current modifier level, including too many repeated occurrences. | Passed | 0 ms |
| strict hydrate rejects nested selections for terminal choices | Choices without nested modifiers reject unexpected nested selection payloads. | Passed | 0 ms |
| titled modifiers expose title without affecting hydration | A modifier title is descriptive metadata and does not affect selection hydration. | Passed | 0 ms |
| modifiers can walk nested definition tree | The definition walker visits modifiers, prompts, and choices in nested order. | Passed | 0 ms |
| prompt validation returns effects and nested modifier definitions | Validated choice selections carry the choice effects and nested modifier definition needed by consumers. | Passed | 0 ms |
| flat modifier price is multiplied by selected factor | A priced choice emits one contribution and selected child factor choices multiply that contribution upward. | Passed | 0 ms |
| invariant rate price adds to flat amount without floats | A choice can add a flat amount plus an integer rate of the selected variant invariant price. | Passed | 0 ms |
| nested factors multiply up to the nearest priced ancestor | Factor choices can stack through nested modifiers and are consumed by the nearest priced ancestor branch. | Passed | 0 ms |
| defaults are free unless pricing policy says otherwise | Default selected priced choices contribute zero by default, and the pricing policy can opt into charging them. | Passed | 0 ms |
| unconsumed root factor is invalid | A factor choice with no priced ancestor is rejected instead of silently disappearing. | Passed | 0 ms |
