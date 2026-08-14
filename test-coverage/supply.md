# Supply

Described behavior tests for generic fulfillability, provider resolution, bucketed supply, and reversible supply claims.

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 7
- Passed: 7
- Failed: 0

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| supply view resolves available unavailable and unresolved requests | A provider-backed supply view answers exact target and bucket requests with Available, Unavailable, or Unresolved. | Passed | 0 ms |
| supply buckets distinguish calculated supply | Supply buckets let one target represent separate calculated resources such as time windows or capacity classes. | Passed | 0 ms |
| supply shapes reject invalid keys quantities and duplicates | Supply keys, bucket dimensions, request quantities, and available supply rows validate their deterministic shape. | Passed | 0 ms |
| supply reserve and unreserve are reversible | Reserve creates a provisional claim and Unreserve reverses that exact claim without consuming supply. | Passed | 0 ms |
| supply consume and unconsume are reversible | Consume records final use and Unconsume reverses that exact consumed claim. | Passed | 0 ms |
| supply consume can commit a matching reservation | A reserved claim can be committed by consuming the same target, bucket, and quantity. | Passed | 0 ms |
| supply ledger rejects invalid transitions and mismatched consumes | The supply ledger rejects unknown claims, duplicate claims, mismatched consume requests, and impossible reversals. | Passed | 0 ms |
