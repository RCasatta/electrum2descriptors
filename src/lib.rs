pub mod electrum_extended_priv_key;
pub mod electrum_extended_pub_key;
#[cfg(feature = "wallet_file")]
pub mod electrum_wallet_file;

pub use electrum_extended_priv_key::ElectrumExtendedPrivKey;
pub use electrum_extended_pub_key::ElectrumExtendedPubKey;
#[cfg(feature = "wallet_file")]
pub use electrum_wallet_file::ElectrumWalletFile;

pub trait ElectrumExtendedKey {
    /// Returns internal and external descriptor
    fn to_descriptors(&self) -> Vec<String>;

    /// Returns the bitcoin extended key (xpub or xprv) as String
    fn xkey_str(&self) -> String;

    /// Returns the kind of script
    fn kind(&self) -> &str;
}
