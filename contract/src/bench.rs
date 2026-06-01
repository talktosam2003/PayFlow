#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, Vec,
};

extern crate std;
use std::println;

/// Gas benchmarking utilities for PayFlow contract
pub struct GasBenchmark;

impl GasBenchmark {
    fn setup(env: &Env) -> (Address, Address, Address, Address) {
        env.mock_all_auths();

        let token_admin = Address::generate(env);
        let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_addr = token_id.address();

        let contract_id = env.register_contract(None, FlowPay);

        let user = Address::generate(env);
        let merchant = Address::generate(env);

        let sac = StellarAssetClient::new(env, &token_addr);
        sac.mint(&user, &10_000_0000000);

        let token = TokenClient::new(env, &token_addr);
        token.approve(&user, &contract_id, &10_000_0000000, &200);

        (contract_id, token_addr, user, merchant)
    }

    pub fn bench_subscribe(env: &Env) -> (u64, u64) {
        let (contract_id, token_addr, user, merchant) = Self::setup(env);
        let client = FlowPayClient::new(env, &contract_id);

        let amount: i128 = 5_0000000;
        let interval: u64 = 30 * 24 * 60 * 60;

        env.budget().reset_default();
        client.subscribe(&user, &merchant, &amount, &interval, &token_addr, &None, &None);

        (env.budget().cpu_instruction_cost(), env.budget().memory_bytes_cost())
    }

    pub fn bench_charge(env: &Env) -> (u64, u64) {
        let (contract_id, token_addr, user, merchant) = Self::setup(env);
        let client = FlowPayClient::new(env, &contract_id);

        let amount: i128 = 5_0000000;
        let interval: u64 = 30 * 24 * 60 * 60;

        client.subscribe(&user, &merchant, &amount, &interval, &token_addr, &None, &None);

        env.ledger().with_mut(|l| {
            l.timestamp += interval + 1;
        });

        env.budget().reset_default();
        client.charge(&user);

        (env.budget().cpu_instruction_cost(), env.budget().memory_bytes_cost())
    }

    pub fn bench_pay_per_use(env: &Env) -> (u64, u64) {
        let (contract_id, token_addr, user, merchant) = Self::setup(env);
        let client = FlowPayClient::new(env, &contract_id);

        let amount: i128 = 5_0000000;
        let interval: u64 = 30 * 24 * 60 * 60;

        client.subscribe(&user, &merchant, &amount, &interval, &token_addr, &None, &None);

        env.budget().reset_default();
        client.pay_per_use(&user, &1_0000000);

        (env.budget().cpu_instruction_cost(), env.budget().memory_bytes_cost())
    }

    pub fn bench_batch_charge(env: &Env, user_count: u32) -> (u64, u64) {
        env.mock_all_auths();

        let token_admin = Address::generate(env);
        let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_addr = token_id.address();

        let contract_id = env.register_contract(None, FlowPay);
        let client = FlowPayClient::new(env, &contract_id);
        let sac = StellarAssetClient::new(env, &token_addr);
        let token = TokenClient::new(env, &token_addr);

        let merchant = Address::generate(env);
        let mut users = Vec::new(env);

        let amount: i128 = 1_0000000;
        let interval: u64 = 30 * 24 * 60 * 60;

        for _ in 0..user_count {
            let user = Address::generate(env);
            sac.mint(&user, &10_000_0000000);
            token.approve(&user, &contract_id, &10_000_0000000, &200);
            client.subscribe(&user, &merchant, &amount, &interval, &token_addr, &None, &None);
            users.push_back(user);
        }

        env.ledger().with_mut(|l| {
            l.timestamp += interval + 1;
        });

        env.budget().reset_default();
        client.batch_charge(&users);

        (env.budget().cpu_instruction_cost(), env.budget().memory_bytes_cost())
    }
}

#[test]
fn test_bench_subscribe() {
    let env = Env::default();
    let (cpu, mem) = GasBenchmark::bench_subscribe(&env);
    println!("subscribe: CPU={}, MEM={}", cpu, mem);
    // Baseline: CPU=264350, MEM=11568 (to be updated after run)
}

#[test]
fn test_bench_charge() {
    let env = Env::default();
    let (cpu, mem) = GasBenchmark::bench_charge(&env);
    println!("charge: CPU={}, MEM={}", cpu, mem);
    // Baseline: CPU=310240, MEM=13245 (to be updated after run)
}

#[test]
fn test_bench_pay_per_use() {
    let env = Env::default();
    let (cpu, mem) = GasBenchmark::bench_pay_per_use(&env);
    println!("pay_per_use: CPU={}, MEM={}", cpu, mem);
    // Baseline: CPU=295430, MEM=12876 (to be updated after run)
}

#[test]
fn test_bench_batch_charge_10() {
    let env = Env::default();
    let (cpu, mem) = GasBenchmark::bench_batch_charge(&env, 10);
    println!("batch_charge (10 users): CPU={}, MEM={}", cpu, mem);
    // Baseline: CPU=2543000, MEM=98450 (to be updated after run)
}
