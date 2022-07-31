#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod utxo;

use parity_scale_codec::{Decode, Encode};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use log::info;

use sp_api::impl_runtime_apis;
use sp_block_builder::runtime_decl_for_BlockBuilder::BlockBuilder;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, Extrinsic, Hash},
	transaction_validity::{TransactionSource, TransactionValidity, ValidTransaction},
	ApplyExtrinsicResult, BoundToRuntimeAppPublic,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_storage::well_known_keys;

#[cfg(any(feature = "std", test))]
use sp_runtime::{BuildStorage, Storage};

use sp_core::{OpaqueMetadata, H256};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/*
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
	"jsonrpc":"2.0",
	"id":1,
	"method":"author_submitExtrinsic",
	"params": ["0x"]
}'

curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
	"jsonrpc":"2.0",
	"id":1,
	"method":"state_getStorage",
	"params": ["0x626F6F6C65616E"]
}'
*/

/// An index to a block.
pub type BlockNumber = u32;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datas-tructures.
pub mod opaque {
	use sp_runtime::OpaqueExtrinsic;

	use super::*;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, BasicExtrinsic>;

	// This part is necessary for generating session keys in the runtime
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: AuraAppPublic,
			pub grandpa: GrandpaAppPublic,
		}
	}

	// Typically these are not implemented manually, but rather for the pallet associated with the
	// keys. Here we are not using the pallets, and these implementations are trivial, so we just
	// re-write them.
	pub struct AuraAppPublic;
	impl BoundToRuntimeAppPublic for AuraAppPublic {
		type Public = AuraId;
	}

	pub struct GrandpaAppPublic;
	impl BoundToRuntimeAppPublic for GrandpaAppPublic {
		type Public = sp_finality_grandpa::AuthorityId;
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("frameless-runtime"),
	impl_name: create_runtime_str!("frameless-runtime"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

// /// The type that provides the genesis storage values for a new chain
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Default))]
// pub struct GenesisConfig;

// Todo Talk to Joshy about how this is working from a UTXO standpoint.
// How is alice or anyone else able to start spending given storage?
// Namely how does first item get slotted into storage?
// (hash of transaction which created UTXO and index of this UTXO in the transaction)?
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GenesisConfig {
	pub genesis_utxos: Vec<utxo::TransactionOutput>,
}

#[cfg(feature = "std")]
impl Default for GenesisConfig {
	fn default() -> Self {
		GenesisConfig { genesis_utxos: Default::default() }
	}
}

#[cfg(feature = "std")]
impl BuildStorage for GenesisConfig {
	fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
		// we have nothing to put into storage in genesis, except this:
		storage.top.insert(well_known_keys::CODE.into(), WASM_BINARY.unwrap().to_vec());

		for utxo in &self.genesis_utxos {
			storage.top.insert(BlakeTwo256::hash_of(&utxo).encode(), utxo.encode());
		}

		Ok(())
	}
}

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, BasicExtrinsic>;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Calls {
	Flipper(u8),
	Adder(u8),
	Spend(u8), // Todo Change this to be a u128
}

// this extrinsic type does nothing other than fulfill the compiler.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct BasicExtrinsic(Calls);

impl Extrinsic for BasicExtrinsic {
	type Call = Calls;
	type SignaturePayload = ();

	fn new(data: Self::Call, _: Option<Self::SignaturePayload>) -> Option<Self> {
		Some(Self(data))
	}
}

// 686561646572 raw storage key
pub const HEADER_KEY: [u8; 6] = *b"header";
pub const FLIPPER_KEY: [u8; 6] = *b"flipit";
pub const ADDER_KEY: [u8; 5] = *b"addme";

/// The main struct in this module. In frame this comes from `construct_runtime!`
pub struct Runtime;

impl_runtime_apis! {
	// https://substrate.dev/rustdocs/master/sp_api/trait.Core.html
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		// state root check
		// todo!("How to do a state_root check??");

		fn execute_block(block: Block) {
			info!(target: "frameless", "🖼️ Entering execute_block. block: {:?}", block);
			Self::initialize_block(&block.header);

			for extrinsic in block.extrinsics {
				Self::apply_extrinsic(extrinsic);
				// todo!();
			}

			Self::finalize_block();
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			info!(target: "frameless", "🖼️ Entering initialize_block. header: {:?}", header);
			sp_io::storage::set(&HEADER_KEY, &header.encode());
		}
	}

	// https://substrate.dev/rustdocs/master/sc_block_builder/trait.BlockBuilderApi.html
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			info!(target: "frameless", "🖼️ Entering apply_extrinsic: {:?}", extrinsic);

			let call = extrinsic.0;
			// todo!("Add the actual extrinsic here(which is just sending)..");

			match call {
				Calls::Flipper(_) => {
					if let Some(vec_bytes) = sp_io::storage::get(&FLIPPER_KEY) {
						let val = bool::decode(&mut &vec_bytes[..]).unwrap();
						info!(target: "frameless", "What did I READ??::: {:?}", val);
						let new_val = !val;
						sp_io::storage::set(&FLIPPER_KEY, &new_val.encode());
					}
					else {
						sp_io::storage::set(&FLIPPER_KEY, &false.encode());
						info!(target: "frameless", "Storage Initialized: False");
					}
				}
				Calls::Adder(add_val) => {
					if let Some(vec_bytes) = sp_io::storage::get(&ADDER_KEY) {
						let val = u8::decode(&mut &vec_bytes[..]).unwrap();
						info!(target: "frameless", "READ VAL FOR STORAGE FUCK RAMSEY: {:?}", &val);
						let new_val = val + add_val;
						sp_io::storage::set(&ADDER_KEY, &new_val.encode());
					}
					else {
						sp_io::storage::set(&ADDER_KEY, &0.encode());
						info!(target: "frameless", "Storage Initialized FOR ADDER: 0");
					}
				},
				Calls::Spend(tx) => {
					// Todo! spend the stuff
				},
			}
			// we don't do anything here, but we probably should...

			Ok(Ok(()))
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			info!(target: "frameless", "🖼️ Entering finalize block.");

			let raw_header = sp_io::storage::get(&HEADER_KEY)
				.expect("We initialized with header, it never got mutated, qed");
			sp_io::storage::clear(&HEADER_KEY);

			let mut header = <Block as BlockT>::Header::decode(&mut &*raw_header)
				.expect("we put a valid header in in the first place, qed");
			let raw_state_root = &sp_io::storage::root(sp_storage::StateVersion::default())[..];

			header.state_root = sp_core::H256::decode(&mut &raw_state_root[..]).unwrap();
			header
		}

		// This runtime does not expect any inherents so it does not insert any into blocks it builds.
		fn inherent_extrinsics(_data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			info!(target: "frameless", "🖼️ Entering inherent_extrinsics.");
			Vec::new()
		}

		// This runtime does not expect any inherents, so it does not do any inherent checking.
		fn check_inherents(
			block: Block,
			_data: sp_inherents::InherentData
		) -> sp_inherents::CheckInherentsResult {
			info!(target: "frameless", "🖼️ Entering check_inherents. block: {:?}", block);
			sp_inherents::CheckInherentsResult::default()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			info!(target: "frameless", "🖼️ Entering validate_transaction. source: {:?}, tx: {:?}, block hash: {:?}", source, tx, block_hash);

			// we don't know how to validate this -- It should be fine??
			// todo!("Implement validation of a UTXO transaction here.. I think.");
			let data = tx.0;
			Ok(ValidTransaction { provides: vec![data.encode()], ..Default::default() })
		}
	}

	// Ignore everything after this.

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(vec![0])
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(_header: &<Block as BlockT>::Header) {
			// we do not do anything.
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			info!(target: "frameless", "🖼️ Entering generate_session_keys. seed: {:?}", seed);
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	// Here is the Aura API for the sake of making this runtime work with the node template node
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			// Three-second blocks
			sp_consensus_aura::SlotDuration::from_millis(3000)
		}

		fn authorities() -> Vec<AuraId> {
			// The only authority is Alice. This makes things work nicely in `--dev` mode
			use sp_application_crypto::ByteArray;

			vec![
				AuraId::from_slice(&hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").to_vec()).unwrap()
			]
		}
	}

	impl sp_finality_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_finality_grandpa::AuthorityList {
			use sp_application_crypto::ByteArray;
			vec![
				(
					sp_finality_grandpa::AuthorityId::from_slice(&hex_literal::hex!("88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee").to_vec()).unwrap(),
					1
				)
			]
		}

		fn current_set_id() -> sp_finality_grandpa::SetId {
			0u64
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_finality_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				sp_runtime::traits::NumberFor<Block>,
			>,
			_key_owner_proof: sp_finality_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_finality_grandpa::SetId,
			_authority_id: sp_finality_grandpa::AuthorityId,
		) -> Option<sp_finality_grandpa::OpaqueKeyOwnershipProof> {
			None
		}
	}

}

#[cfg(test)]
mod tests {
	use super::*;

	use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};
	use sp_core::testing::SR25519;
	use sp_keystore::testing::KeyStore;
	use sp_keystore::{KeystoreExt, SyncCryptoStore};
	use hex_literal::hex;

	use std::sync::Arc;

	const ALICE_PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	// other random account generated with subkey
	const KARL_PHRASE: &str = "monitor exhibit resource stumble subject nut valid furnace obscure misery satoshi assume";
	const GENESIS_UTXO: [u8; 32] = hex!("79eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999");

	// This function basically just builds a genesis storage key/value store according to our desired mockup.
	// We start each test by giving Alice 100 utxo to start with.
	fn new_test_ext() -> sp_io::TestExternalities {

		let keystore = KeyStore::new(); // a key storage to store new key pairs during testing
		let alice_pub_key = keystore.sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();

		let mut t = GenesisConfig::default()
			.build_storage()
			.expect("Frameless system builds valid default genesis config");

		BuildStorage::assimilate_storage(
			&super::GenesisConfig {
				genesis_utxos: vec![
					utxo::TransactionOutput {
						value: 100,
						pubkey: H256::from(alice_pub_key),
					}
				],
				..Default::default()
			},
			&mut t
		)
		.expect("UTXO Pallet storage can be assimilated");

		// Todo Ask Joshy about what exactly this is doing I have a rough idea.
		let mut ext = sp_io::TestExternalities::from(t);
		ext.register_extension(KeystoreExt(Arc::new(keystore)));
		// Todo Ask Joshy is this necessary? How to do in frameless?
		// ext.execute_with(|| System::set_block_number(1));
		ext
	}

	#[test]
	fn utxo_frameless_genesis_test() {
		new_test_ext().execute_with(|| {
			let alice_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[0];
			let mut utxo_output = utxo::TransactionOutput {
				value: 100,
				pubkey: H256::from(alice_pub_key),
			};

			let key = BlakeTwo256::hash_of(&utxo_output);
			let mut val_retrieved = sp_io::storage::get(&key.encode()).unwrap();
			assert_eq!(
				utxo::TransactionOutput::decode(&mut &val_retrieved[..]).unwrap(),
				utxo_output
			);
		})
	}
}
