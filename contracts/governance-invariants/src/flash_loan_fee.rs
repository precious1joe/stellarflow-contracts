#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct FlashLoanFeeEngine;

#[contractimpl]
impl FlashLoanFeeEngine {
    pub fn calculate_flash_loan_fee(env: Env, caller: Address, f_base: i128, w_max: i128) -> i128 {
        let w_ve: i128 = env
            .storage()
            .persistent()
            .get(&(&env, caller))
            .unwrap_or(0);
        
        if w_max <= 0 {
            return f_base;
        }

        let ratio_numerator = w_ve.min(w_max);
        let discount_factor = (ratio_numerator * 1_000_000) / w_max;
        let max_discount = 500_000;
        let applied_discount = discount_factor.min(max_discount);

        let remaining_factor = 1_000_000 - applied_discount;
        (f_base * remaining_factor) / 1_000_000
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_flash_loan_fee_discount_invariants() {
        let env = Env::default();
        let caller = Address::generate(&env);
        let f_base = 1000;
        let w_max = 10000;

        let fee_zero = FlashLoanFeeEngine::calculate_flash_loan_fee(env.clone(), caller.clone(), f_base, w_max);
        assert_eq!(fee_zero, 1000);

        env.storage().persistent().set(&(&env, caller.clone()), &5000);
        let fee_half = FlashLoanFeeEngine::calculate_flash_loan_fee(env.clone(), caller.clone(), f_base, w_max);
        assert_eq!(fee_half, 750);

        env.storage().persistent().set(&(&env, caller.clone()), &10000);
        let fee_max = FlashLoanFeeEngine::calculate_flash_loan_fee(env.clone(), caller.clone(), f_base, w_max);
        assert_eq!(fee_max, 500);

        env.storage().persistent().set(&(&env, caller.clone()), &20000);
        let fee_over = FlashLoanFeeEngine::calculate_flash_loan_fee(env.clone(), caller, f_base, w_max);
        assert_eq!(fee_over, 500);
    }
}
