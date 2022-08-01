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
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;
use sp_runtime::{
	traits::{BlakeTwo256, Hash, SaturatedConversion},
	transaction_validity::{TransactionLongevity, ValidTransaction},
};

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

// How to add weight here for charging a fee.
/// Execute transaction
/// Check transaction validity
/// Update the storage
pub fn spend(transaction: Transaction) -> DispatchResult {
    let tx_validity = validate_transaction(&transaction)?;
    update_storage(&transaction)?;
    Ok(())
}

/// Called by Txpool and Runtime
/// // Todo: Understand each and everyone of these
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
    let mut output_index: u64 = 0;
    let stripped_transaction = get_stripped_transaction(&transaction);

    // Verify inputs
    for input in transaction.inputs.iter() {
        if let Some(utxo_bytes) =
            sp_io::storage::get(&input.outpoint.encode()) {
                let utxo =
                    TransactionOutput::decode(&mut &utxo_bytes[..])
                    .expect("If Transaction is stored correctly this should never happen; QED");
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
        }
        else {
            // To keep it simple we want to fail here.
            return Err("Cant Process Transaction yet");
        }
    }

    // Verify outputs
    for output in transaction.outputs.iter() {
        ensure!(output.value > 0, "Output values must be greater than zero");
        // ensure no duplicate utxos
        let new_utxo_hash_key = BlakeTwo256::hash_of(&(&transaction.encode(), output_index));
        output_index = output_index.checked_add(1).ok_or("output index overflow")?;
        ensure!(
            !sp_io::storage::exists(&new_utxo_hash_key.encode()),
            "output utxo already exists"
        );
        total_output = total_output.checked_add(output.value).ok_or("output value overflow")?;
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
fn get_stripped_transaction(transaction: &Transaction) -> Vec<u8> {
    let mut tx = transaction.clone();
    for input in tx.inputs.iter_mut() {
        input.sigscript = H512::zero();
    }
    tx.encode()
}

/// Make changes to storage
/// A key in storage is a hash of a transaction +
/// its order in the TransactionOutput Vec in Order to avoid duplications.
fn update_storage(transaction: &Transaction) -> DispatchResult {
    /// Remove UTXOS which were spent
    for input in transaction.inputs.iter() {
        sp_io::storage::clear(&input.outpoint.encode());
    }

    // Add new utxos to storage
    let mut output_index: u64 = 0;
    for output in transaction.outputs.iter() {
        let key = BlakeTwo256::hash_of(&(&transaction.encode(), output_index));
        output_index = output_index.checked_add(1).ok_or("output index overflow")?;
        sp_io::storage::set(&key.encode(), &output.encode());
    }

    Ok(())
}

mod tests {
    use super::*;

    #[test]
	fn validate_no_transaction_outputs_fails() {
        let inputs = vec![
            TransactionInput {
                ..Default::default()
            },
            TransactionInput {
                ..Default::default()
            }
        ];
        let tx = Transaction {
            inputs,
            ..Default::default()
        };

		let res = validate_transaction(&tx).err().unwrap();
        assert_eq!(res, "No outputs");
	}

    #[test]
	fn validate_no_transaction_inputs_fails() {
        let outputs = vec![
            TransactionOutput {
                ..Default::default()
            },
            TransactionOutput {
                ..Default::default()
            }
        ];
        let tx = Transaction {
            outputs,
            ..Default::default()
        };

		let res = validate_transaction(&tx).err().unwrap();
        assert_eq!(res, "No inputs");
	}

    #[test]
    fn validate_outputs_not_unique_fails() {
        let outputs = vec![
            TransactionOutput {
                ..Default::default()
            },
            TransactionOutput {
                ..Default::default()
            }
        ];
        let inputs = vec![
            TransactionInput {
                ..Default::default()
            }
        ];
        let tx = Transaction {
            inputs,
            outputs,
        };
        let res = validate_transaction(&tx).err().unwrap();
        assert_eq!(res, "Outputs not unique");
    }

    #[test]
    fn validate_inputs_not_unique_fails() {
        let inputs = vec![
            TransactionInput {
                ..Default::default()
            },
            TransactionInput {
                ..Default::default()
            }
        ];
        let outputs = vec![
            TransactionOutput {
                ..Default::default()
            }
        ];
        let tx = Transaction {
            inputs,
            outputs,
        };
        let res = validate_transaction(&tx).err().unwrap();
        assert_eq!(res, "Inputs not unique");
    }
}
