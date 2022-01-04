use libelectrum2descriptors::{
    electrum_wallet_file::*, ElectrumExtendedKey, ElectrumExtendedPrivKey, ElectrumExtendedPubKey,
};
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
        .or_else(|_| ElectrumExtendedPubKey::from_str(&electrum_x).map(|e| e.to_descriptors()))
        .or_else(|_| {
            let wallet_file = Path::new(&electrum_x)
                .canonicalize()
                .map_err(|_| err_msg.clone())?;
            if !wallet_file.exists() {
                return Err(err_msg);
            }
            let json = get_json_from_wallet_file(wallet_file.as_path())?;
            electrum_wallet_to_descriptors(json)
        })?;

    println!("{:?}", descriptor);
    Ok(())
}
