pub mod electrum_extended_priv_key;
pub mod electrum_extended_pub_key;

pub use electrum_extended_priv_key::ElectrumExtendedPrivKey;
pub use electrum_extended_pub_key::ElectrumExtendedPubKey;

pub trait ElectrumExtendedKey {
    fn to_descriptors(&self) -> Vec<String>;
}
