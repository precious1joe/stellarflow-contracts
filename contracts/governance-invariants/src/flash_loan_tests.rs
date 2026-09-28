use super::*;
use soroban_sdk::{Env, Address};

#[test]
fn test_flash_loan_fee_discounts() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, GovernanceInvariantsContract);
    let client = GovernanceInvariantsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);

    let w_max = 10_000_000_000_i128;
    let base_fee_bps = 30; // 0.30%

    // 1. Zero stake: no discount, fee should remain base_fee_bps (30)
    let fee_zero = client.calculate_flash_loan_fee(&user, &base_fee_bps, &w_max);
    assert_eq!(fee_zero, 30);

    // 2. Stake half of W_max: 25% discount (discount_bps = 2500)
    // W_ve = 5_000_000_000
    client.lock_tokens(&user, &5_000_000_000_i128);
    let fee_half = client.calculate_flash_loan_fee(&user, &base_fee_bps, &w_max);
    // 30 * (10000 - 2500) / 10000 = 30 * 7500 / 10000 = 22
    assert_eq!(fee_half, 22);

    // 3. Stake equal to or greater than W_max: max 50% discount (discount_bps = 5000)
    // Let's extend or lock more. Since user already locked, let's test another user with full W_max.
    let max_user = Address::generate(&env);
    client.lock_tokens(&max_user, &10_000_000_000_i128);
    let fee_max = client.calculate_flash_loan_fee(&max_user, &base_fee_bps, &w_max);
    // 30 * (10000 - 5000) / 10000 = 15
    assert_eq!(fee_max, 15);

    // Verify invariants across tier levels
    let fee_over = client.calculate_flash_loan_fee(&max_user, &base_fee_bps, &5_000_000_000_i128);
    // W_max is smaller than user weight, ratio capped at 0.5 -> 50% discount -> 15
    assert_eq!(fee_over, 15);
}
