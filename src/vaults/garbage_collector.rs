use soroban_sdk::{contracttype, Address, Env, Vec, token};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultStorageKey {
    ClosedVaults,
    VaultPosition(Address),
}

pub struct VaultGarbageCollector;

impl VaultGarbageCollector {
    /// Traverse closed vault list and erase zero-balance persistent storage keys.
    /// Transfer reclaimed rent deposit back to transaction trigger address.
    pub fn purge_expired_vaults(env: &Env, trigger_addr: &Address, native_token: &Address) -> u32 {
        trigger_addr.require_auth();

        let closed_vaults_key = VaultStorageKey::ClosedVaults;
        let closed_vaults: Vec<Address> = env
            .storage()
            .persistent()
            .get(&closed_vaults_key)
            .unwrap_or_else(|| Vec::new(env));

        let initial_storage_count = env.storage().persistent().keys().len();
        let mut purged_count = 0;

        for vault in closed_vaults.iter() {
            let pos_key = VaultStorageKey::VaultPosition(vault.clone());
            let balance: i128 = env.storage().persistent().get(&pos_key).unwrap_or(0);
            if balance == 0 {
                env.storage().persistent().remove(&pos_key);
                purged_count += 1;
            }
        }

        let final_storage_count = env.storage().persistent().keys().len();
        let delta = initial_storage_count - final_storage_count;

        if purged_count > 0 {
            let rent_reclaimed = (purged_count as i128) * 100;
            let token_client = token::Client::new(env, native_token);
            let contract_address = env.current_contract_address();
            token_client.transfer(&contract_address, trigger_addr, &rent_reclaimed);
        }

        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, token};

    #[test]
    fn test_purge_expired_vaults_storage_delta() {
        let env = Env::default();
        env.mock_all_auths();

        let trigger = Address::generate(&env);
        let vault1 = Address::generate(&env);
        let vault2 = Address::generate(&env);
        let native_token = env.register_stellar_asset_contract(trigger.clone());

        let closed_vaults_key = VaultStorageKey::ClosedVaults;
        let mut closed_vaults = Vec::new(&env);
        closed_vaults.push_back(vault1.clone());
        closed_vaults.push_back(vault2.clone());

        env.storage().persistent().set(&closed_vaults_key, &closed_vaults);

        env.storage().persistent().set(&VaultStorageKey::VaultPosition(vault1.clone()), &0i128);
        env.storage().persistent().set(&VaultStorageKey::VaultPosition(vault2.clone()), &500i128);

        let initial_keys_len = env.storage().persistent().keys().len();

        let delta = VaultGarbageCollector::purge_expired_vaults(&env, &trigger, &native_token);

        let final_keys_len = env.storage().persistent().keys().len();

        assert_eq!(initial_keys_len - final_keys_len, delta);
        assert!(delta > 0);
        assert!(!env.storage().persistent().has(&VaultStorageKey::VaultPosition(vault1)));
        assert!(env.storage().persistent().has(&VaultStorageKey::VaultPosition(vault2)));
    }
}
