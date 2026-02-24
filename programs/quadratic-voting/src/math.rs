//! Pure math and validation helpers for quadratic cost and vote weight.

use anchor_lang::prelude::*;

use crate::errors::QuadraticVotingError;
use crate::state::QuadraticBallot;

pub const MAX_MULTIPLIER_BPS: u16 = 20_000;
pub const MIN_MULTIPLIER_BPS: u16 = 5_000;

/// Ensures multiplier is within the ballot's configured reputation bounds.
pub fn require_multiplier_bounds(ballot: &QuadraticBallot, multiplier_bps: u16) -> Result<()> {
    require!(
        multiplier_bps >= ballot.min_reputation_bps
            && multiplier_bps <= ballot.max_reputation_bps,
        QuadraticVotingError::ReputationMultiplierOutOfBounds
    );
    Ok(())
}

/// Quadratic cost for increasing votes from `previous_votes` to `new_votes`: cost = new² - old².
pub fn quadratic_increment_cost(previous_votes: u32, new_votes: u32) -> Result<u64> {
    require!(new_votes >= previous_votes, QuadraticVotingError::MathOverflow);
    let before = square_u64(previous_votes);
    let after = square_u64(new_votes);
    after
        .checked_sub(before)
        .ok_or(QuadraticVotingError::MathOverflow.into())
}

pub fn square_u64(value: u32) -> u64 {
    let v = value as u64;
    v.saturating_mul(v)
}

/// Vote weight scaled by reputation multiplier (basis points). Result is in scaled units.
pub fn scaled_vote_weight(votes: u32, multiplier_bps: u16) -> Result<u128> {
    let weighted = (votes as u128)
        .checked_mul(multiplier_bps as u128)
        .ok_or(QuadraticVotingError::MathOverflow)?;
    require!(
        multiplier_bps <= MAX_MULTIPLIER_BPS && multiplier_bps >= MIN_MULTIPLIER_BPS,
        QuadraticVotingError::ReputationMultiplierOutOfBounds
    );
    Ok(weighted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadratic_cost_delta_is_correct() {
        let delta = quadratic_increment_cost(5, 10).unwrap();
        assert_eq!(delta, 75);
    }

    #[test]
    fn scaled_weight_uses_bps_precision() {
        let weighted = scaled_vote_weight(10, 15_000).unwrap();
        assert_eq!(weighted, 150_000);
    }
}
