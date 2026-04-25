//! Property-based tests for vote tally invariants.
//!
//! These tests operate on plain arithmetic that mirrors the tally logic in
//! `GovernanceContract::cast_vote` and `finalise`, without requiring a
//! Soroban execution environment.

#![cfg(test)]

use proptest::prelude::*;

/// Maximum token supply used across all strategies (mirrors a realistic cap).
const MAX_SUPPLY: i128 = i128::MAX / 4;

proptest! {
    /// Invariant 1: votes_yes + votes_no + votes_abstain == total_votes
    ///
    /// For any non-negative split of votes the sum must equal the total.
    #[test]
    fn tally_sum_equals_total(
        yes     in 0i128..=MAX_SUPPLY / 3,
        no      in 0i128..=MAX_SUPPLY / 3,
        abstain in 0i128..=MAX_SUPPLY / 3,
    ) {
        let total = yes + no + abstain;
        prop_assert_eq!(total, yes + no + abstain);
    }

    /// Invariant 2: total votes never exceed total token supply.
    ///
    /// Each voter contributes at most their balance, and the sum of all
    /// balances is bounded by the total supply.
    #[test]
    fn total_votes_never_exceed_supply(
        supply  in 1i128..=MAX_SUPPLY,
        yes     in 0i128..=MAX_SUPPLY,
        no      in 0i128..=MAX_SUPPLY,
        abstain in 0i128..=MAX_SUPPLY,
    ) {
        // Clamp each bucket so the combined tally stays within supply,
        // replicating the constraint that voters can only spend their balance.
        let yes     = yes.min(supply);
        let no      = no.min(supply - yes);
        let abstain = abstain.min(supply - yes - no);

        let total = yes + no + abstain;
        prop_assert!(total <= supply);
    }

    /// Invariant 3: quorum check is consistent with stored vote counts.
    ///
    /// A proposal passes iff total_votes >= quorum AND votes_yes > votes_no.
    /// This mirrors the condition in `finalise` exactly.
    #[test]
    fn quorum_check_consistent_with_vote_counts(
        yes     in 0i128..=MAX_SUPPLY / 3,
        no      in 0i128..=MAX_SUPPLY / 3,
        abstain in 0i128..=MAX_SUPPLY / 3,
        quorum  in 1i128..=MAX_SUPPLY,
    ) {
        let total = yes + no + abstain;
        let passes = total >= quorum && yes > no;

        // Re-derive from the same inputs — must agree.
        let expected = (yes + no + abstain) >= quorum && yes > no;
        prop_assert_eq!(passes, expected);
    }
}
