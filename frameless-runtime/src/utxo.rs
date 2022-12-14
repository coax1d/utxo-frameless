use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{
	H256,
	H512,
	sr25519::{Public, Signature},
};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	transaction_validity::{TransactionLongevity, ValidTransaction},
};

use log::info;

// Value to represent a fungible value of a UTXO
pub type Value = u128;
pub type DispatchResult = Result<(), sp_runtime::DispatchError>;

/// Return Err of the expression: `return Err($expression);`.
///
/// Used as `fail!(expression)`.
#[macro_export]
macro_rules! fail {
	( $y:expr ) => {{
		return Err($y.into())
	}};
}

#[macro_export]
macro_rules! ensure {
	( $x:expr, $y:expr $(,)? ) => {{
		if !$x {
			fail!($y);
		}
	}};
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Transaction {
	/// UTXOs to be used as inputs for current transaction
	pub inputs: Vec<TransactionInput>,

	/// UTXOs to be created as a result of current transaction dispatch
	pub outputs: Vec<TransactionOutput>,
}

/// Single transaction input that refers to one UTXO
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, MaxEncodedLen, TypeInfo)]
pub struct TransactionInput {
	/// Reference to an UTXO to be spent
	pub outpoint: H256,

	/// Proof that transaction owner is authorized to spend referred UTXO &
	/// that the entire transaction is untampered
	pub sigscript: H512,
}

/// Single transaction output to create upon transaction dispatch
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, MaxEncodedLen, TypeInfo)]
pub struct TransactionOutput {
	/// Value associated with this output
	pub value: Value,

	/// Public key associated with this output. In order to spend this output
	/// owner must provide a proof by hashing the whole `Transaction` and
	/// signing it with a corresponding private key.
	pub pubkey: H256,
}

/// Execute transaction
/// Check transaction validity
/// Update the storage
pub fn spend(mut transaction: Transaction) -> DispatchResult {
    info!(target: "frameless", "??????? Spending {:?}", &transaction);
    validate_transaction(&transaction)?;
    update_storage(&mut transaction)?;
    Ok(())
}

/// Called by Txpool and Runtime
/// Verify inputs and outputs are non-empty
/// All inputs map to existing unspent && unlocked outputs
/// Each input is unique.
/// Each output is unique && is non-zero
/// Total output value does not exceed total input value
/// New outputs are unique
/// Sum of total input and output does not overflow
/// verify signatures
/// outputs cannot be exploited
pub fn validate_transaction(transaction: &Transaction) -> Result<ValidTransaction, &'static str> {
    ensure!(!transaction.inputs.is_empty(), "No inputs");
    ensure!(!transaction.outputs.is_empty(), "No outputs");

    {
        // Check for uniqueness once. Afterwards dont need input_set.
        let input_set: BTreeSet<_> = transaction.inputs.iter().collect();
        ensure!(input_set.len() == transaction.inputs.len(), "Inputs not unique");
    }
    {
        // Check for uniqueness once. Afterwards dont need output_set.
        let output_set: BTreeSet<_> = transaction.outputs.iter().collect();
        ensure!(output_set.len() == transaction.outputs.len(), "Outputs not unique");
    }

    let mut total_input: Value = 0;
    let mut total_output: Value = 0;
    let stripped_transaction = get_stripped_transaction(&transaction);

    for input in transaction.inputs.iter() {
        match sp_io::storage::get(&input.outpoint.encode()) {
            Some(utxo_bytes) => {
                let utxo =
                    TransactionOutput::decode(&mut &utxo_bytes[..])
                    .expect("Should never happen; QED");
                // Check Signature
                let sig_verify_result =
                    sp_io::crypto::sr25519_verify(
                        &Signature::from_raw(*input.sigscript.as_fixed_bytes()),
                        &stripped_transaction,
                        &Public::from_h256(utxo.pubkey),
                    );
                ensure!(sig_verify_result, "Invalid Signature to spend this Input");
                total_input =
                    total_input
                    .checked_add(utxo.value)
                    .ok_or("input value overflow")?;
            },
            None => {
                // To keep it simple we want to fail here. No handling for races.
                return Err("No existing UTXO for this specified outpoint, Invalid Input");
            }
        }
    }

    // Need to keep track of the output_index in order to avoid hashing
    // collisions in storage.
    let mut output_index: u64 = 0;
    // Verify outputs
    for output in transaction.outputs.iter() {
        ensure!(output.value > 0, "Output values must be greater than zero");
        // ensure no duplicate utxo keys in the database.
        let new_utxo_hash_key = BlakeTwo256::hash_of(&(&transaction.encode(), output_index));
        output_index = output_index.checked_add(1).ok_or("output index overflow")?;
        ensure!(
            !sp_io::storage::exists(&new_utxo_hash_key.encode()),
            "output utxo already exists"
        );
        total_output = total_output
            .checked_add(output.value)
            .ok_or("output value overflow")?;
    }

    if total_output > total_input {
        return Err("Total outputs cannot exceed total inputs");
    }

    Ok(ValidTransaction {
        longevity: TransactionLongevity::max_value(),
        propagate: true,
        ..Default::default()
    })
}

/// Strip inputs of a transaction of their signature field
/// Replace signature field with H512 all zeros
/// @return: scale encoded tx
pub fn get_stripped_transaction(transaction: &Transaction) -> Vec<u8> {
    let mut tx = transaction.clone();
    for input in tx.inputs.iter_mut() {
        input.sigscript = H512::zero();
    }
    tx.encode()
}

/// Make changes to storage
/// A key in storage is a hash of a transaction with no input signatures +
/// its order in the TransactionOutput Vec in Order to avoid duplications.
fn update_storage(transaction: &mut Transaction) -> DispatchResult {
    // Remove UTXOS which were spent && strip signatures from inputs
    // To prep for storing deterministic keys.
    for input in transaction.inputs.iter_mut() {
        input.sigscript = H512::zero();
        sp_io::storage::clear(&input.outpoint.encode());
    }

    // Add new utxos to storage
    let mut output_index: u64 = 0;
    for output in transaction.outputs.iter() {
        let key = BlakeTwo256::hash_of(&(&transaction.encode(), output_index));
        output_index = output_index.checked_add(1).ok_or("output index overflow")?;
        sp_io::storage::set(&key.encode(), &output.encode());
        info!(target: "frameless", "??????? Storing UTXO {:?} at key {:?}", output, key);
    }

    Ok(())
}