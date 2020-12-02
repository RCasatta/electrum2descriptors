use libelectrum2descriptors::ElectrumExtendedPubKey;
use std::str::FromStr;

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    args.next(); // first is program name
    let electrum_xpub = args
        .next()
        .ok_or_else(|| "You must specify an electrum xpub as first argument".to_string())?;
    let electrum_xpub = ElectrumExtendedPubKey::from_str(&electrum_xpub)?;
    println!("{:?}", electrum_xpub.to_descriptors());
    Ok(())
}
