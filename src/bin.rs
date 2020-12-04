use libelectrum2descriptors::{
    ElectrumExtendedKey, ElectrumExtendedPrivKey, ElectrumExtendedPubKey,
};
use std::str::FromStr;

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    args.next(); // first is program name
    let electrum_x = args.next().ok_or_else(|| {
        "You must specify an extended public or private key as first argument".to_string()
    })?;
    let descriptor = ElectrumExtendedPrivKey::from_str(&electrum_x)
        .map(|e| e.to_descriptors())
        .or_else(|_| ElectrumExtendedPubKey::from_str(&electrum_x).map(|e| e.to_descriptors()))?;

    println!("{:?}", descriptor);
    Ok(())
}
