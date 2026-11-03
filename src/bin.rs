#[cfg(feature = "wallet_file")]
use libelectrum2descriptors::ElectrumWalletFile;
use libelectrum2descriptors::{
    Electrum2DescriptorError, ElectrumExtendedKey, ElectrumExtendedPrivKey, ElectrumExtendedPubKey,
};
#[cfg(feature = "wallet_file")]
use std::path::Path;
use std::str::FromStr;

fn main() -> Result<(), Electrum2DescriptorError> {
    let mut args = std::env::args();
    args.next(); // first is program name
    let err_msg =
        "You must specify an extended public or private key, an electrum wallet file or a descriptor as first argument";
    let electrum_x = args
        .next()
        .ok_or_else(|| Electrum2DescriptorError::GenericBorrow(err_msg))?;
    let descriptor = ElectrumExtendedPrivKey::from_str(&electrum_x)
        .map(|e| e.to_descriptors())
        .or_else(|_| ElectrumExtendedPubKey::from_str(&electrum_x).map(|e| e.to_descriptors()));
    let descriptor = descriptor.or_else(|_| {
        ElectrumWalletFile::from_descriptor(&electrum_x)
            .map(|e| {
                let f = "electrum2descriptors.wallet";
                e.to_file(Path::new(f));
                println!("Wrote an electrum wallet file to {}", f);
                e.to_descriptors()
            })
            .flatten()
    });
    #[cfg(feature = "wallet_file")]
    let descriptor = descriptor.or_else(|_| {
        let wallet_file = Path::new(&electrum_x)
            .canonicalize()
            .map_err(|_| Electrum2DescriptorError::GenericBorrow(err_msg))?;
        if !wallet_file.exists() {
            return Err(Electrum2DescriptorError::GenericBorrow(err_msg));
        }
        let wallet = ElectrumWalletFile::from_file(wallet_file.as_path())?;
        wallet.to_descriptors()
    });

    println!("{:?}", descriptor?);
    Ok(())
}
