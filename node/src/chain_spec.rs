use calcx_runtime::WASM_BINARY;
use sc_service::ChainType;

pub type ChainSpec = sc_service::GenericChainSpec;

pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.unwrap_or(&[]),
        None,
    )
    .with_name("Development")
    .with_id("dev")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(serde_json::json!({}))
    .build())
}
