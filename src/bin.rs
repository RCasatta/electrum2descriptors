use libelectrum2descriptors::{
    electrumextendedkey::ElectrumExtendedKey, electrumextendedprivkey::ElectrumExtendedPrivKey,
    electrumextendedpubkey::ElectrumExtendedPubKey,
};
use std::str::FromStr;

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    args.next(); // first is program name
    let electrum_x = args
        .next()
        .ok_or_else(|| "You must specify an electrum xpub as first argument".to_string())?;
    let electrum_x: Box<dyn ElectrumExtendedKey> = match &electrum_x[1..4] {
        "prv" => Box::new(ElectrumExtendedPrivKey::from_str(&electrum_x)?),
        "pub" => Box::new(ElectrumExtendedPubKey::from_str(&electrum_x)?),
        id => return Err(format!("Invalid identifier: {}", id)),
    };
    println!("{:?}", electrum_x.to_descriptors());
    Ok(())
}
