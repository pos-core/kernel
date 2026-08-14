# Money

Money is a currency code paired with an integer amount in that currency's minor units.

Arithmetic is checked for overflow and rejects incompatible currencies. Floating-point values are not used.

Rates are integer numerator-and-denominator ratios. Rate multiplication can remain rational until an explicit named rounding strategy materializes a new Money value.
