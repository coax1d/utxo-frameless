#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod utxo;
use parity_scale_codec::{Decode, Encode};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use log::info;

use sp_api::impl_runtime_apis;
use sp_block_builder::runtime_decl_for_BlockBuilder::BlockBuilder;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, Extrinsic, Hash},
	transaction_validity::{
		TransactionSource,
		TransactionValidity,
		ValidTransaction,
		InvalidTransaction,
		TransactionValidityError
	},
	ApplyExtrinsicResult, BoundToRuntimeAppPublic,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_storage::well_known_keys;

#[cfg(any(feature = "std", test))]
use sp_runtime::{BuildStorage, Storage};

use sp_core::{OpaqueMetadata, H256, H512, hexdisplay::HexDisplay};

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

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GenesisConfig {
	pub genesis_utxos: Vec<utxo::TransactionOutput>,
}

#[cfg(feature = "std")]
impl Default for GenesisConfig {
	fn default() -> Self {
		use hex_literal::hex;

		const ALICE_PUB_KEY_BYTES: [u8; 32] =
			hex!("d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67");

		GenesisConfig { genesis_utxos: vec![utxo::TransactionOutput {
				value: 100,
				pubkey: H256::from(ALICE_PUB_KEY_BYTES),
		}]}
	}
}

#[cfg(feature = "std")]
impl BuildStorage for GenesisConfig {
	fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
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

// this extrinsic type does nothing other than fulfill the compiler.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct BasicExtrinsic(utxo::Transaction);

impl Extrinsic for BasicExtrinsic {
	type Call = utxo::Transaction;
	type SignaturePayload = ();

	fn new(data: Self::Call, _: Option<Self::SignaturePayload>) -> Option<Self> {
		Some(Self(data))
	}
}

pub const HEADER_KEY: [u8; 6] = *b"header";

/// The main struct in this module. In frame this comes from `construct_runtime!`
pub struct Runtime;

impl_runtime_apis! {
	// https://substrate.dev/rustdocs/master/sp_api/trait.Core.html
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			info!(target: "frameless", "🖼️ Entering execute_block. block: {:?}", block);
			// state root check
			//header.state_root = sp_core::H256::decode(&mut &raw_state_root[..]).unwrap();
			//check equal to block.header.state_root
			Self::initialize_block(&block.header);

			for extrinsic in block.extrinsics {
				Self::apply_extrinsic(extrinsic);
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

			let transaction = extrinsic.0;
			// Call spend
			match utxo::spend(transaction) {
				Err(_) => {
					Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(1)))
				},
				Ok(_) => {
					Ok(Ok(()))
				}
			}
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
			info!(target: "frameless", "🖼️ new block header on finalization: {:?}", header);
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

			match utxo::validate_transaction(&tx.0) {
				Ok(mut valid) => {
					valid.provides = vec![tx.0.encode()];
					Ok(valid)
				},
				Err(_) => {
					Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(1)))
				},
			}
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
			&super::GenesisConfig::default(),
			&mut t
		)
		.expect("UTXO Pallet storage can be assimilated");

		let mut ext = sp_io::TestExternalities::from(t);
		ext.register_extension(KeystoreExt(Arc::new(keystore)));
		ext
	}

	#[test]
	fn utxo_frameless_genesis_test() {
		new_test_ext().execute_with(|| {
			let keystore = KeyStore::new();
			let alice_pub_key = keystore.sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();
			let mut utxo_output = utxo::TransactionOutput {
				value: 100,
				pubkey: H256::from(alice_pub_key),
			};

			let mut val_retrieved = sp_io::storage::get(&GENESIS_UTXO).unwrap();
			assert_eq!(
				utxo::TransactionOutput::decode(&mut &val_retrieved[..]).unwrap(),
				utxo_output
			);
		})
	}

	#[test]
	fn utxo_frameless_spend_transaction() {
		new_test_ext().execute_with(|| {
			let keystore = KeyStore::new();
			let alice_pub_key = keystore.sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();

			let mut transaction = utxo::Transaction {
				inputs: vec![
					utxo::TransactionInput {
						outpoint: H256::from(GENESIS_UTXO),
						sigscript: H512::zero(),
					},
				],
				outputs: vec![
					utxo::TransactionOutput {
						value: 25,
						pubkey: H256::from(alice_pub_key)
					}
				],
			};

			let signature =
				sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode())
				.unwrap();

			transaction.inputs[0].sigscript = H512::from(signature);

			let extrinsic = BasicExtrinsic(transaction.clone());
			println!("Extrinsic Scale encoded hex::{}", HexDisplay::from(&extrinsic.encode()));

			let new_utxo_hash_key = BlakeTwo256::hash_of(&(&transaction.encode(), 0 as u64));
			println!("New_utxo_key::{}", HexDisplay::from(&new_utxo_hash_key.encode()));
			assert_ok!(utxo::spend(transaction));
			assert!(!sp_io::storage::exists(&H256::from(GENESIS_UTXO).encode()));
			assert!(sp_io::storage::exists(&new_utxo_hash_key.encode()));

			let mut new_utxo =
					sp_io::storage::get(&new_utxo_hash_key.encode()).unwrap();
			assert_eq!(utxo::TransactionOutput::decode(&mut &new_utxo[..]).unwrap().value, 25);
			assert_eq!(utxo::TransactionOutput::decode(&mut &new_utxo[..]).unwrap().pubkey, H256::from(alice_pub_key));
		})
	}

	#[test]
	fn utxo_frameless_create_outputs_from_no_existing_utxo_fails() {
		new_test_ext().execute_with(|| {
			let keystore = KeyStore::new();
			let alice_pub_key = keystore.sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();

			let mut transaction = utxo::Transaction {
				inputs: vec![ utxo::TransactionInput {
					outpoint: H256::zero(),
					sigscript: H512::zero(),
				}],
				outputs: vec![ utxo::TransactionOutput {
					value: 100,
					pubkey: H256::from(alice_pub_key)
				}],
			};

			let signature =
				sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode())
				.unwrap();
			transaction.inputs[0].sigscript = H512::from(signature);
			let spend_result = utxo::spend(transaction).err().unwrap();
			assert_eq!(
				spend_result,
				sp_runtime::DispatchError::Other(
					"No existing UTXO for this specified outpoint, Invalid Input")
			);
		})
	}

	#[test]
	fn utxo_frameless_double_spend_same_utxo() {
		new_test_ext().execute_with(|| {
			let keystore = KeyStore::new();
			let alice_pub_key = keystore.sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();

			let mut transaction = utxo::Transaction {
				inputs: vec![
					utxo::TransactionInput {
						outpoint: H256::from(GENESIS_UTXO),
						sigscript: H512::zero()
				},
					utxo::TransactionInput {
						outpoint: H256::from(GENESIS_UTXO),
						sigscript: H512::zero()
					}
				],
				outputs: vec![
					utxo::TransactionOutput {
						value: 25,
						pubkey: H256::from(alice_pub_key),
				}],
			};

			let signature =
				sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode())
				.unwrap();
			for input in transaction.inputs.iter_mut() {
				input.sigscript = H512::from(signature.clone());
			}
			let spend_result = utxo::spend(transaction).err().unwrap();
			assert_eq!(spend_result,
				sp_runtime::DispatchError::Other("Inputs not unique")
			);
		})
	}
}
