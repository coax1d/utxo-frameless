use node_template_runtime::GenesisConfig as FramelessGenesisConfig;
use sc_service::ChainType;
use sp_core::{Pair, Public, sr25519, H256, ByteArray};
use hex_literal::hex;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<FramelessGenesisConfig>;

// /// Generate a crypto pair from seed.
// pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
// 	TPublic::Pair::from_string(&format!("//{}", seed), None)
// 		.expect("static values are valid; qed")
// 		.public()
// }

// type AccountPublic = <Signature as Verify>::Signer;

// /// Generate an account ID from seed.
// pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
// where
// 	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
// {
// 	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
// }

// /// Generate an Aura authority key.
// pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
// 	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
// }

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		"Development",
		"dev",
		sc_service::ChainType::Development,
		|| testnet_genesis(
			vec![
				// Alice
				hex!("79eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999"),
			],
		),
		vec![],
		None,
		None,
		None,
		None,
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		sc_service::ChainType::Local,
		|| testnet_genesis(
			vec![
				// Alice
				hex!("79eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999"),
			],
		),
		vec![],
		None,
		None,
		None,
		None,
		None,
	))
}

fn testnet_genesis(endowed_utxos: Vec<[u8; 32]>) -> FramelessGenesisConfig {
	FramelessGenesisConfig {
		  genesis_utxos: endowed_utxos
			.iter()
			.map(|x|
				node_template_runtime::utxo::TransactionOutput {
					value: 100 as node_template_runtime::utxo::Value,
					pubkey: H256::from_slice(x),
				}
			)
			.collect()
	}
}
