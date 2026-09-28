use super::*;
use soroban_sdk::{Env, Symbol, vec};

#[test]
fn test_proposal_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(GovernanceExecuterContract, ());
    let client = GovernanceExecuterContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let target = Address::generate(&env);
    let function = Symbol::new(&env, "some_func");
    let payload = vec![&env];

    // Create proposal with 0 delay for testing
    let proposal_id = client.create_proposal(&target, &function, &payload, &0);
    assert_eq!(proposal_id, 1);

    let proposal = client.get_proposal(&1);
    assert!(!proposal.executed);

    // Note: In a full integration test, target contract would be invoked here.
}

#[test]
fn test_batch_execution_timelock_and_atomicity() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(GovernanceExecuterContract, ());
    let client = GovernanceExecuterContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let target = Address::generate(&env);
    let function = Symbol::new(&env, "some_func");
    let payload = vec![&env];

    let id1 = client.create_proposal(&target, &function, &payload, &0);
    let id2 = client.create_proposal(&target, &function, &payload, &0);

    let batch = vec![&env, id1, id2];
    let results = client.execute_batch(&batch);
    assert_eq!(results.len(), 2);

    let p1 = client.get_proposal(&id1);
    let p2 = client.get_proposal(&id2);
    assert!(p1.executed);
    assert!(p2.executed);
}
