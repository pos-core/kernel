# Money

Described behavior tests for checked minor-unit arithmetic, integer rates, and named rounding strategies.

## Definitions

- [Money](../src/primitives/money/money.md)

## Result

- Generated: 2026-05-22 12:29:37 UTC
- Total cases: 3
- Passed: 3
- Failed: 0

## Behaviors

| Test | Description | Status | Time |
| --- | --- | --- | ---: |
| money uses checked minor units | Money stores integer minor units and performs checked arithmetic without floating-point values. | Passed | 0 ms |
| money multiplies by integer rates with named rounding | Rate multiplication keeps rational arithmetic exact until an explicit named rounding strategy materializes minor units. | Passed | 0 ms |
| money rounds up to named increment and ending targets | Named rounding can move an amount upward to a required minor-unit increment or price ending. | Passed | 0 ms |
