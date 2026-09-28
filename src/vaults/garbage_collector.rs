use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec, Symbol};

#[contract]
pub struct VaultGarbageCollector;

#[contractimpl]
impl VaultGarbageCollector {
    pub fn purge_expired_vaults(env: Env, trigger: Address, closed_vaults: Vec<Address>, token: Address, rent_amount: i128) {
        trigger.require_auth();

        let mut storage_delta: i64 = 0;

        for vault in closed_vaults.iter() {
            let balance_key = (Symbol::new(&env, "VaultBalance"), vault.clone());
            if env.storage().persistent().has(&balance_key) {
                let balance: i128 = env.storage().persistent().get(&balance_key).unwrap_or(0);
                if balance == 0 {
                    env.storage().persistent().remove(&balance_key);
                    storage_delta += 1;
                }
            }
        }

        if storage_delta > 0 && rent_amount > 0 {
            let token_client = token::Client::new(&env, &token);
            token_client.transfer(&env.current_contract_address(), &trigger, &rent_amount);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_purge_expired_vaults_delta() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(VaultGarbageCollector, ());
        let client = VaultGarbageCollectorClient::new(&env, &contract_id);

        let token_admin = Address::generate(&env);
        let token_id = env.register(
            soroban_sdk::token::Token,
            soroban_sdk::token::TokenArgs::new(&token_admin),
        );
        let token_client = soroban_sdk::token::Client::new(&env, &token_id);

        let trigger = Address::generate(&env);
        let vault1 = Address::generate(&env);
        let vault2 = Address::generate(&env);

        token_client.mint(&contract_id, &100);

        let balance_key1 = (Symbol::new(&env, "VaultBalance"), vault1.clone());
        let balance_key2 = (Symbol::new(&env, "VaultBalance"), vault2.clone());

        env.storage().persistent().set(&balance_key1, &0i128);
        env.storage().persistent().set(&balance_key2, &100i128);

        assert!(env.storage().persistent().has(&balance_key1));
        assert!(env.storage().persistent().has(&balance_key2));

        let mut closed_vaults = Vec::new(&env);
        closed_vaults.push_back(vault1.clone());
        closed_vaults.push_back(vault2.clone());

        client.purge_expired_vaults(&trigger, &closed_vaults, &token_id, &50);

        assert!(!env.storage().persistent().has(&balance_key1));
        assert!(env.storage().persistent().has(&balance_key2));
        assert_eq!(token_client.balance(&trigger), 50);
    }
}
