//! Pure math helpers for the realms-adapter program.

use solana_program::program_error::ProgramError;

use crate::error::RealmsAdapterError;

pub(crate) fn compute_effective_weight(
    qv_component: u64,
    reputation_multiplier_bps: u16,
) -> Result<u64, ProgramError> {
    let scaled = (qv_component as u128)
        .checked_mul(reputation_multiplier_bps as u128)
        .ok_or(ProgramError::from(RealmsAdapterError::MathOverflow))?;
    let weight = scaled / 10_000u128;
    u64::try_from(weight).map_err(|_| RealmsAdapterError::MathOverflow.into())
}

pub(crate) fn integer_sqrt_u64(value: u64) -> u64 {
    if value < 2 {
        return value;
    }

    let mut x0 = value;
    let mut x1 = (x0 + value / x0) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x0 + value / x0) / 2;
    }
    x0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_rounds_down() {
        assert_eq!(integer_sqrt_u64(99), 9);
        assert_eq!(integer_sqrt_u64(100), 10);
        assert_eq!(integer_sqrt_u64(101), 10);
    }

    #[test]
    fn weight_uses_bps_multiplier() {
        let weight = compute_effective_weight(10, 15_000).unwrap();
        assert_eq!(weight, 15);
    }
}
