use node_template_runtime::GenesisConfig as FramelessGenesisConfig;
use sc_service::ChainType;
use sp_core::{Pair, Public, sr25519, H256, ByteArray};

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
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || FramelessGenesisConfig { ..Default::default()},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))

	// Ok(ChainSpec::from_genesis(
	// 	"Development",
	// 	"dev",
	// 	sc_service::ChainType::Development,
	// 	|| testnet_genesis(
	// 		get_account_id_from_seed::<sr25519::Public>("Alice"),
	// 		vec![
	// 			get_account_id_from_seed::<sr25519::Public>("Alice"),
	// 			get_account_id_from_seed::<sr25519::Public>("Bob"),
	// 			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
	// 			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
	// 		],
	// 		// Genesis set of pubkeys that own UTXOs
	// 		vec![
	// 			get_from_seed::<sr25519::Public>("Alice"),
	// 			get_from_seed::<sr25519::Public>("Bob"),
	// 		],
	// 		true,
	// 	),
	// 	vec![],
	// 	None,
	// 	None,
	// 	None,
	// 	None,
	// 	None,
	// ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || FramelessGenesisConfig,
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

// fn testnet_genesis(
// 	// wasm_binary: &[u8],
// 	root_key: AccountId,
// 	endowed_accounts: Vec<AccountId>,
// 	endowed_utxos: Vec<sr25519::Public>,
// 	_enable_println: bool
// ) -> FramelessGenesisConfig {
// 	// This prints upon creation of the genesis block
// 	println!("============ HELPER INPUTS FOR THE UI DEMO ============");
// 	println!("OUTPOINT (Alice's UTXO Hash): 0x76584168d10a20084082ed80ec71e2a783abbb8dd6eb9d4893b089228498e9ff\n");
// 	println!("SIGSCRIPT (Alice Signature on a transaction where she spends 50 utxo on Bob): 0x6ceab99702c60b111c12c2867679c5555c00dcd4d6ab40efa01e3a65083bfb6c6f5c1ed3356d7141ec61894153b8ba7fb413bf1e990ed99ff6dee5da1b24fd83\n");
// 	println!("PUBKEY (Bob's public key hash): 0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48\n");
// 	println!("NEW UTXO HASH in UTXOStore onchain: 0xdbc75ab8ee9b83dcbcea4695f9c42754d94e92c3c397d63b1bc627c2a2ef94e6\n");

// 	FramelessGenesisConfig {
// 		  genesis_utxos: endowed_utxos
// 			.iter()
// 			.map(|x|
// 				node_template_runtime::utxo::TransactionOutput {
// 					value: 100 as utxo::Value,
// 					pubkey: H256::from_slice(x.as_slice()),
// 				}
// 			)
// 			.collect()
// 	}
// }
