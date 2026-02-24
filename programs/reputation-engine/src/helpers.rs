//! Pure helpers and validation logic for the reputation-engine program.

use anchor_lang::prelude::*;

use crate::errors::ReputationError;
use crate::state::{RealmReputationConfig, ReputationProfile};

pub(crate) fn require_authorized_updater(
    updater: &Signer,
    config: &Account<RealmReputationConfig>,
) -> Result<()> {
    let key = updater.key();
    require!(
        key == config.admin || key == config.oracle_authority,
        ReputationError::Unauthorized
    );
    Ok(())
}

pub(crate) fn apply_delta_u32(current: u32, delta: i32) -> Result<u32> {
    if delta >= 0 {
        current
            .checked_add(delta as u32)
            .ok_or(ReputationError::MathOverflow.into())
    } else {
        Ok(current.saturating_sub(delta.unsigned_abs()))
    }
}

pub(crate) fn recompute_multiplier(
    config: &Account<RealmReputationConfig>,
    profile: &mut Account<ReputationProfile>,
) -> Result<()> {
    let wp = weighted_points(profile, config)?;

    let mut bonus_bps = (wp / config.points_per_bonus_bps as u128) as u16;
    if bonus_bps > config.max_bonus_bps {
        bonus_bps = config.max_bonus_bps;
    }

    let penalty_bps_u32 = profile
        .penalties_score
        .checked_mul(config.penalty_unit_bps as u32)
        .ok_or(ReputationError::MathOverflow)?;
    let mut penalty_bps = penalty_bps_u32.min(config.max_penalty_bps as u32) as u16;
    if penalty_bps > config.max_penalty_bps {
        penalty_bps = config.max_penalty_bps;
    }

    let boosted = config
        .base_multiplier_bps
        .checked_add(bonus_bps)
        .ok_or(ReputationError::MathOverflow)?;
    let penalized = boosted.saturating_sub(penalty_bps);

    profile.multiplier_bps = penalized.clamp(config.min_multiplier_bps, config.max_multiplier_bps);
    Ok(())
}

pub(crate) fn weighted_points(
    profile: &ReputationProfile,
    config: &RealmReputationConfig,
) -> Result<u128> {
    let mut sum = 0u128;
    sum = sum
        .checked_add(profile.participation_score as u128 * config.participation_weight as u128)
        .ok_or(ReputationError::MathOverflow)?;
    sum = sum
        .checked_add(profile.proposal_creation_score as u128 * config.proposal_weight as u128)
        .ok_or(ReputationError::MathOverflow)?;
    sum = sum
        .checked_add(profile.staking_score as u128 * config.staking_weight as u128)
        .ok_or(ReputationError::MathOverflow)?;
    sum = sum
        .checked_add(profile.tenure_score as u128 * config.tenure_weight as u128)
        .ok_or(ReputationError::MathOverflow)?;
    sum = sum
        .checked_add(profile.delegation_trust_score as u128 * config.delegation_weight as u128)
        .ok_or(ReputationError::MathOverflow)?;
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_subtracts_safely() {
        assert_eq!(apply_delta_u32(10, -4).unwrap(), 6);
        assert_eq!(apply_delta_u32(3, -10).unwrap(), 0);
    }
}
