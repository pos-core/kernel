use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, MoneyError> {
        let value = value.as_ref();
        let valid = value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase());

        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(MoneyError::InvalidCurrency(value.to_owned()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Money {
    amount_minor: i64,
    currency: CurrencyCode,
}

impl Money {
    pub fn new(amount_minor: i64, currency: CurrencyCode) -> Self {
        Self {
            amount_minor,
            currency,
        }
    }

    pub fn zero(currency: CurrencyCode) -> Self {
        Self::new(0, currency)
    }

    pub fn amount_minor(&self) -> i64 {
        self.amount_minor
    }

    pub fn currency(&self) -> &CurrencyCode {
        &self.currency
    }

    pub fn checked_add(&self, other: &Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;

        let amount_minor = self
            .amount_minor
            .checked_add(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;

        Ok(Self::new(amount_minor, self.currency.clone()))
    }

    pub fn checked_sub(&self, other: &Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;

        let amount_minor = self
            .amount_minor
            .checked_sub(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;

        Ok(Self::new(amount_minor, self.currency.clone()))
    }

    pub fn checked_mul_quantity(&self, quantity: u32) -> Result<Self, MoneyError> {
        let amount_minor = self
            .amount_minor
            .checked_mul(i64::from(quantity))
            .ok_or(MoneyError::Overflow)?;

        Ok(Self::new(amount_minor, self.currency.clone()))
    }

    pub fn checked_mul_rate(
        &self,
        rate: Rate,
        rounding: RoundingStrategy,
    ) -> Result<Self, MoneyError> {
        RationalMoney::from_money(self.clone())?
            .checked_mul_rate(rate)?
            .round(rounding)
    }

    fn ensure_same_currency(&self, other: &Self) -> Result<(), MoneyError> {
        if self.currency == other.currency {
            Ok(())
        } else {
            Err(MoneyError::CurrencyMismatch {
                left: self.currency.clone(),
                right: other.currency.clone(),
            })
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Rate {
    numerator: u32,
    denominator: u32,
}

impl Rate {
    pub fn new(numerator: u32, denominator: u32) -> Result<Self, MoneyError> {
        if denominator == 0 {
            return Err(MoneyError::InvalidRateDenominator);
        }

        Ok(Self {
            numerator,
            denominator,
        })
    }

    pub fn zero() -> Self {
        Self {
            numerator: 0,
            denominator: 1,
        }
    }

    pub fn one() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }

    pub fn percent(percent: u32) -> Self {
        Self {
            numerator: percent,
            denominator: 100,
        }
    }

    pub fn basis_points(basis_points: u32) -> Self {
        Self {
            numerator: basis_points,
            denominator: 10_000,
        }
    }

    pub fn numerator(self) -> u32 {
        self.numerator
    }

    pub fn denominator(self) -> u32 {
        self.denominator
    }

    pub fn is_zero(self) -> bool {
        self.numerator == 0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RoundingStrategy {
    CentRoundUp,
    CentRoundDown,
    NearestUpIncrement(u32),
    NearestUpEnding(u32),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RationalMoney {
    numerator_minor: i128,
    denominator: i128,
    currency: CurrencyCode,
}

impl RationalMoney {
    pub fn zero(currency: CurrencyCode) -> Self {
        Self {
            numerator_minor: 0,
            denominator: 1,
            currency,
        }
    }

    pub fn from_money(money: Money) -> Result<Self, MoneyError> {
        Ok(Self {
            numerator_minor: i128::from(money.amount_minor),
            denominator: 1,
            currency: money.currency,
        })
    }

    pub fn from_money_rate(money: &Money, rate: Rate) -> Result<Self, MoneyError> {
        Self::from_money(money.clone())?.checked_mul_rate(rate)
    }

    pub fn currency(&self) -> &CurrencyCode {
        &self.currency
    }

    pub fn checked_add(&self, other: &Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;

        let left = self
            .numerator_minor
            .checked_mul(other.denominator)
            .ok_or(MoneyError::Overflow)?;
        let right = other
            .numerator_minor
            .checked_mul(self.denominator)
            .ok_or(MoneyError::Overflow)?;
        let numerator_minor = left.checked_add(right).ok_or(MoneyError::Overflow)?;
        let denominator = self
            .denominator
            .checked_mul(other.denominator)
            .ok_or(MoneyError::Overflow)?;

        Self::new_checked(numerator_minor, denominator, self.currency.clone())
    }

    pub fn checked_mul_rate(&self, rate: Rate) -> Result<Self, MoneyError> {
        let numerator_minor = self
            .numerator_minor
            .checked_mul(i128::from(rate.numerator()))
            .ok_or(MoneyError::Overflow)?;
        let denominator = self
            .denominator
            .checked_mul(i128::from(rate.denominator()))
            .ok_or(MoneyError::Overflow)?;

        Self::new_checked(numerator_minor, denominator, self.currency.clone())
    }

    pub fn checked_mul_quantity(&self, quantity: u32) -> Result<Self, MoneyError> {
        let numerator_minor = self
            .numerator_minor
            .checked_mul(i128::from(quantity))
            .ok_or(MoneyError::Overflow)?;

        Self::new_checked(numerator_minor, self.denominator, self.currency.clone())
    }

    pub fn round(&self, strategy: RoundingStrategy) -> Result<Money, MoneyError> {
        if self.denominator <= 0 {
            return Err(MoneyError::InvalidRateDenominator);
        }

        if self.numerator_minor < 0 {
            return Err(MoneyError::NegativeRationalAmount);
        }

        let amount_minor = match strategy {
            RoundingStrategy::CentRoundDown => self.numerator_minor / self.denominator,
            RoundingStrategy::CentRoundUp => ceil_div(self.numerator_minor, self.denominator)?,
            RoundingStrategy::NearestUpIncrement(increment) => {
                if increment == 0 {
                    return Err(MoneyError::InvalidRoundingIncrement(increment));
                }

                let increment = i128::from(increment);
                let denominator = self
                    .denominator
                    .checked_mul(increment)
                    .ok_or(MoneyError::Overflow)?;

                ceil_div(self.numerator_minor, denominator)?
                    .checked_mul(increment)
                    .ok_or(MoneyError::Overflow)?
            }
            RoundingStrategy::NearestUpEnding(ending) => {
                if ending > 99 {
                    return Err(MoneyError::InvalidRoundingEnding(ending));
                }

                let whole_minor = ceil_div(self.numerator_minor, self.denominator)?;
                let dollar_floor = whole_minor - whole_minor.rem_euclid(100);
                let candidate = dollar_floor
                    .checked_add(i128::from(ending))
                    .ok_or(MoneyError::Overflow)?;

                if candidate >= whole_minor {
                    candidate
                } else {
                    candidate.checked_add(100).ok_or(MoneyError::Overflow)?
                }
            }
        };

        let amount_minor = i64::try_from(amount_minor).map_err(|_| MoneyError::Overflow)?;

        Ok(Money::new(amount_minor, self.currency.clone()))
    }

    fn new_checked(
        numerator_minor: i128,
        denominator: i128,
        currency: CurrencyCode,
    ) -> Result<Self, MoneyError> {
        if denominator == 0 {
            return Err(MoneyError::InvalidRateDenominator);
        }

        Ok(Self {
            numerator_minor,
            denominator,
            currency,
        })
    }

    fn ensure_same_currency(&self, other: &Self) -> Result<(), MoneyError> {
        if self.currency == other.currency {
            Ok(())
        } else {
            Err(MoneyError::CurrencyMismatch {
                left: self.currency.clone(),
                right: other.currency.clone(),
            })
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MoneyError {
    InvalidCurrency(String),
    InvalidRateDenominator,
    InvalidRoundingIncrement(u32),
    InvalidRoundingEnding(u32),
    CurrencyMismatch {
        left: CurrencyCode,
        right: CurrencyCode,
    },
    NegativeRationalAmount,
    Overflow,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCurrency(value) => {
                write!(
                    f,
                    "invalid currency `{value}`; expected three uppercase ASCII letters"
                )
            }
            Self::InvalidRateDenominator => {
                f.write_str("rate denominator must be greater than zero")
            }
            Self::InvalidRoundingIncrement(increment) => write!(
                f,
                "rounding increment `{increment}` is invalid; expected a positive minor-unit increment"
            ),
            Self::InvalidRoundingEnding(ending) => write!(
                f,
                "rounding ending `{ending}` is invalid; expected a value from 0 to 99"
            ),
            Self::CurrencyMismatch { left, right } => {
                write!(f, "currency mismatch `{left}` != `{right}`")
            }
            Self::NegativeRationalAmount => {
                f.write_str("cannot round a negative rational money amount")
            }
            Self::Overflow => f.write_str("money arithmetic overflow"),
        }
    }
}

impl std::error::Error for MoneyError {}

fn ceil_div(numerator: i128, denominator: i128) -> Result<i128, MoneyError> {
    if denominator <= 0 {
        return Err(MoneyError::InvalidRateDenominator);
    }

    let adjusted = numerator
        .checked_add(denominator - 1)
        .ok_or(MoneyError::Overflow)?;

    Ok(adjusted / denominator)
}

#[cfg(test)]
mod tests {
    use super::{CurrencyCode, Money, Rate, RoundingStrategy};

    #[test]
    fn money_uses_checked_minor_units() {
        let usd = CurrencyCode::parse("USD").unwrap();
        let one = Money::new(100, usd.clone());
        let two = Money::new(250, usd);

        assert_eq!(one.checked_add(&two).unwrap().amount_minor(), 350);
    }

    #[test]
    fn money_multiplies_by_integer_rates_with_named_rounding() {
        let usd = CurrencyCode::parse("USD").unwrap();
        let amount = Money::new(199, usd);

        assert_eq!(
            amount
                .checked_mul_rate(Rate::percent(50), RoundingStrategy::CentRoundDown)
                .unwrap()
                .amount_minor(),
            99
        );
        assert_eq!(
            amount
                .checked_mul_rate(Rate::percent(50), RoundingStrategy::CentRoundUp)
                .unwrap()
                .amount_minor(),
            100
        );
    }

    #[test]
    fn money_rounds_up_to_named_increment_and_ending_targets() {
        let usd = CurrencyCode::parse("USD").unwrap();
        let amount = Money::new(201, usd);

        assert_eq!(
            amount
                .checked_mul_rate(Rate::one(), RoundingStrategy::NearestUpIncrement(25))
                .unwrap()
                .amount_minor(),
            225
        );
        assert_eq!(
            amount
                .checked_mul_rate(Rate::one(), RoundingStrategy::NearestUpEnding(99))
                .unwrap()
                .amount_minor(),
            299
        );
    }
}
