use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultDataKey {
    ClosedVaults,
    VaultBalance(Address),
    RentDeposit(Address),
}

pub struct VaultGarbageCollector;

impl VaultGarbageCollector {
    pub fn collect_expired_vaults(env: &Env, trigger_address: &Address) -> u32 {
        trigger_address.require_auth();

        let closed_vaults: Vec<Address> = env
            .storage()
            .persistent()
            .get(&VaultDataKey::ClosedVaults)
            .unwrap_or_else(|| Vec::new(env));

        let mut purged_count: u32 = 0;
        let mut total_reclaimed_rent: i128 = 0;

        for vault in closed_vaults.iter() {
            let balance_key = VaultDataKey::VaultBalance(vault.clone());
            let balance: i128 = env
                .storage()
                .persistent()
                .get(&balance_key)
                .unwrap_or(0);

            if balance == 0 {
                if env.storage().persistent().has(&balance_key) {
                    env.storage().persistent().remove(&balance_key);
                    purged_count += 1;
                }

                let rent_key = VaultDataKey::RentDeposit(vault.clone());
                let rent_amount: i128 = env
                    .storage()
                    .persistent()
                    .get(&rent_key)
                    .unwrap_or(0);

                if rent_amount > 0 {
                    env.storage().persistent().remove(&rent_key);
                    total_reclaimed_rent += rent_amount;
                }
            }
        }

        if total_reclaimed_rent > 0 {
            let current_trigger_balance: i128 = env
                .storage()
                .persistent()
                .get(&VaultDataKey::VaultBalance(trigger_address.clone()))
                .unwrap_or(0);
            env.storage().persistent().set(
                &VaultDataKey::VaultBalance(trigger_address.clone()),
                &(current_trigger_balance + total_reclaimed_rent),
            );
        }

        purged_count
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_garbage_collector_storage_delta() {
        let env = Env::default();
        env.mock_all_auths();

        let trigger = Address::generate(&env);
        let closed_vault = Address::generate(&env);

        let mut closed_vaults = Vec::new(&env);
        closed_vaults.push_back(closed_vault.clone());

        env.storage()
            .persistent()
            .set(&VaultDataKey::ClosedVaults, &closed_vaults);
        env.storage()
            .persistent()
            .set(&VaultDataKey::VaultBalance(closed_vault.clone()), &0i128);
        env.storage()
            .persistent()
            .set(&VaultDataKey::RentDeposit(closed_vault.clone()), &500i128);

        assert!(env.storage().persistent().has(&VaultDataKey::VaultBalance(closed_vault.clone())));
        assert!(env.storage().persistent().has(&VaultDataKey::RentDeposit(closed_vault.clone())));

        let purged = VaultGarbageCollector::collect_expired_vaults(&env, &trigger);
        assert_eq!(purged, 1);

        assert!(!env.storage().persistent().has(&VaultDataKey::VaultBalance(closed_vault.clone())));
        assert!(!env.storage().persistent().has(&VaultDataKey::RentDeposit(closed_vault.clone())));

        let trigger_balance: i128 = env
            .storage()
            .persistent()
            .get(&VaultDataKey::VaultBalance(trigger.clone()))
            .unwrap();
        assert_eq!(trigger_balance, 500);
    }
}
