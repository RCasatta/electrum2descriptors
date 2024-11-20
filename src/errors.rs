use bitcoin::{base58, bip32, secp256k1};
#[cfg(feature = "wallet_file")]
use serde_json::Error as SerdeError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Electrum2DescriptorError {
    #[cfg(feature = "wallet_file")]
    #[error(transparent)]
    Serde(#[from] SerdeError),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
    #[error(transparent)]
    Base58Error(#[from] base58::Error),
    #[error(transparent)]
    Secp256k1Error(#[from] secp256k1::Error),
    #[error(transparent)]
    Bip32Error(#[from] bip32::Error),
    #[cfg(feature = "wallet_file")]
    #[error(transparent)]
    RegexError(#[from] regex::Error),

    #[error("Unknown type")]
    UnknownType,
    #[error("Unknown wallet type: {0}")]
    UnknownWalletType(String),
    #[error("Multisig with less than two signers doesn't make a lot of sense")]
    MultisigFewSigners,
    #[error("Unknown multisig descriptor format: {0}")]
    UnknownDescriptorFormat(String),
    #[error("Wrong number of keystores: {0}; expected: {1}")]
    WrongNumberOfKeyStores(usize, usize),
    #[error("Minimum number of signatures {0} must not be greater than keystores {1}")]
    NumberSignaturesKeyStores(u8, usize),
    #[error("keystore sizes above 255 are not currently supported. {0}")]
    TooManyKeyStores(usize),
    #[error("Unknown script kind: {0}")]
    UnknownScriptKind(String),
    #[error("{0}")]
    GenericBorrow(&'static str),
}
