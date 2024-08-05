use std::str::FromStr as _;

use edr_eth::{spec::HardforkActivations, SpecId, B256};
use edr_provider::{
    hardhat_rpc_types::ForkConfig, test_utils::create_test_config_with_fork, time::CurrentTime,
    MethodInvocation, NoopLogger, Provider, ProviderRequest,
};
use edr_test_utils::env::get_alchemy_url;
use tokio::runtime;

// https://github.com/NomicFoundation/edr/issues/570
#[tokio::test(flavor = "multi_thread")]
async fn issue_570() -> anyhow::Result<()> {
    let logger = Box::new(NoopLogger);
    let subscriber = Box::new(|_event| {});

    let mut config = create_test_config_with_fork(Some(ForkConfig {
        json_rpc_url: get_alchemy_url().replace("eth-mainnet", "base-sepolia"),
        block_number: Some(13_560_400),
        http_headers: None,
    }));

    let chain_id = 84532;

    config
        .chains
        .insert(chain_id, HardforkActivations::with_spec_id(SpecId::CANCUN));

    // The default chain id set by Hardhat
    config.chain_id = chain_id;

    let provider = Provider::new(
        runtime::Handle::current(),
        logger,
        subscriber,
        config,
        CurrentTime,
    )?;

    let transaction_hash =
        B256::from_str("0xe565eb3bfd815efcc82bed1eef580117f9dc3d6896db42500572c8e789c5edd4")?;

    let result = provider.handle_request(ProviderRequest::Single(
        MethodInvocation::DebugTraceTransaction(transaction_hash, None),
    ))?;

    assert!(!result.traces.is_empty());

    Ok(())
}
