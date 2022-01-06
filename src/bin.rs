#[cfg(feature = "wallet_file")]
use libelectrum2descriptors::ElectrumWalletFile;
use libelectrum2descriptors::{
    ElectrumExtendedKey, ElectrumExtendedPrivKey, ElectrumExtendedPubKey,
};
#[cfg(feature = "wallet_file")]
use std::path::Path;
use std::str::FromStr;

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    args.next(); // first is program name
    let err_msg =
        "You must specify an extended public or private key or an electrum wallet file as first argument".to_string();
    let electrum_x = args.next().ok_or_else(|| err_msg.clone())?;
    let descriptor = ElectrumExtendedPrivKey::from_str(&electrum_x)
        .map(|e| e.to_descriptors())
        .or_else(|_| ElectrumExtendedPubKey::from_str(&electrum_x).map(|e| e.to_descriptors()));
    #[cfg(feature = "wallet_file")]
    let descriptor = descriptor.or_else(|_| {
        let wallet_file = Path::new(&electrum_x)
            .canonicalize()
            .map_err(|_| err_msg.clone())?;
        if !wallet_file.exists() {
            return Err(err_msg);
        }
        let wallet = ElectrumWalletFile::from_file(wallet_file.as_path())?;
        wallet.to_descriptors()
    });

    println!("{:?}", descriptor?);
    Ok(())
}
