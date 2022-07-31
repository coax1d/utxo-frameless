use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{
	crypto::ByteArray,
	H256,
	H512,
	sr25519::{Public, Signature},
};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;
use sp_runtime::{
	traits::{BlakeTwo256, Hash, SaturatedConversion},
	transaction_validity::{TransactionLongevity, ValidTransaction},
};

// Value to represent a fungible value of a UTXO
pub type Value = u128;
pub type DispatchResult = Result<(), sp_runtime::DispatchError>;

#[macro_export]
macro_rules! ensure {
	( $x:expr, $y:expr $(,)? ) => {{
		if !$x {
			$crate::fail!($y);
		}
	}};
}

#[macro_export]
macro_rules! assert_noop {
	(
		$x:expr,
		$y:expr $(,)?
	) => {
		let h = $crate::storage_root($crate::StateVersion::V1);
		$crate::assert_err!($x, $y);
		assert_eq!(h, $crate::storage_root($crate::StateVersion::V1));
	};
}

#[macro_export]
macro_rules! assert_ok {
	( $x:expr $(,)? ) => {
		let is = $x;
		match is {
			Ok(_) => (),
			_ => assert!(false, "Expected Ok(_). Got {:#?}", is),
		}
	};
	( $x:expr, $y:expr $(,)? ) => {
		assert_eq!($x, Ok($y));
	};
}

/// Single transaction to be dispatched - Extrinsic
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Transaction {
	/// UTXOs to be used as inputs for current transaction
	pub inputs: Vec<TransactionInput>,

	/// UTXOs to be created as a result of current transaction dispatch
	pub outputs: Vec<TransactionOutput>,
}

/// Single transaction input that refers to one UTXO
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, MaxEncodedLen, TypeInfo)]
pub struct TransactionInput {
	/// Reference to an UTXO to be spent
	pub outpoint: H256,

	/// Proof that transaction owner is authorized to spend referred UTXO &
	/// that the entire transaction is untampered
	pub sigscript: H512,
}

/// Single transaction output to create upon transaction dispatch
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, MaxEncodedLen, TypeInfo)]
pub struct TransactionOutput {
	/// Value associated with this output
	pub value: Value,

	/// Public key associated with this output. In order to spend this output
	/// owner must provide a proof by hashing the whole `Transaction` and
	/// signing it with a corresponding private key.
	pub pubkey: H256,
}

pub fn spend(transaction: Transaction) -> DispatchResult {
    let hash = BlakeTwo256::hash_of(&TransactionOutput{ ..Default::default() });
    Ok(())
}

pub fn validate_transaction() -> Result<ValidTransaction, &'static str> {
    Ok(ValidTransaction {
        ..Default::default()
    })
}

fn update_storage() -> DispatchResult {
    Ok(())
}
