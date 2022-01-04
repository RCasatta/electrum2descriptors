use crate::{ElectrumExtendedKey, ElectrumExtendedPrivKey, ElectrumExtendedPubKey};
use regex::Regex;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
enum WalletType {
    Standard,
    Multisig(u8, u8),
}

pub fn electrum_wallet_to_descriptors(json: serde_json::Value) -> Result<Vec<String>, String> {
    let wallet_type = json
        .get("wallet_type")
        .ok_or_else(|| "Failed to parse wallet_type".to_string())?
        .to_string();
    let wallet_type = get_wallet_type(&wallet_type)?;
    match wallet_type {
        WalletType::Standard => {
            let keystore = json
                .get("keystore")
                .ok_or_else(|| "Failed to parse the keystore for single sig".to_string())?;
            let exkey = get_xkey_from_keystore(keystore)?;
            let desc_ext = exkey.kind().to_string() + "(" + &exkey.xkeystr() + "/0/*)";
            let desc_chg = exkey.kind().to_string() + "(" + &exkey.xkeystr() + "/1/*)";
            Ok(vec![desc_ext, desc_chg])
        }
        WalletType::Multisig(x, y) => {
            let mut desc = String::new();
            for i in 1..(y + 1) {
                let query = format!("x{}/", i);
                let keystore = json
                    .get(query)
                    .ok_or_else(|| "Failed to parse the keystore for multi sig".to_string())?;
                let exkey = get_xkey_from_keystore(keystore)?;
                if desc.is_empty() {
                    let prefix = match &exkey.kind().to_string() as &str {
                        "pkh" => "sh",
                        kind => kind,
                    }
                    .to_string();
                    desc = prefix + &format!("(sortedmulti({}", x);
                }
                desc += &(",".to_string() + &exkey.xkeystr() + "/0/*");
            }
            desc += "))";
            let opening = desc.matches('(').count();
            let closing = desc.matches(')').count();
            if opening > closing {
                desc += ")"
            };
            let desc_chg = desc.replace("/0/*", "/1/*");

            Ok(vec![desc, desc_chg])
        }
    }
}

pub fn get_xkey_from_keystore(
    keystore: &serde_json::Value,
) -> Result<Box<dyn ElectrumExtendedKey>, String> {
    if let Some(xprv) = keystore.get("xprv") {
        if let Some(xprv) = xprv.as_str() {
            let exprv = ElectrumExtendedPrivKey::from_str(xprv)?;
            return Ok(Box::new(exprv));
        }
    }

    let xpub = keystore
        .get("xpub")
        .ok_or("Failed to find the xpub")?
        .as_str()
        .ok_or("Failed to convert the xpub to str")?;
    let expub = ElectrumExtendedPubKey::from_str(xpub)?;
    Ok(Box::new(expub))
}

pub fn get_json_from_wallet_file(wallet: &Path) -> Result<serde_json::Value, String> {
    let file =
        std::fs::File::open(wallet).map_err(|_| "Failed to read the wallet file.".to_string())?;
    let json: serde_json::Value = serde_json::from_reader(file)
        .map_err(|_| "Failed to parse the wallet file.".to_string())?;

    Ok(json)
}

impl fmt::Display for WalletType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn get_wallet_type(wallet_type: &str) -> Result<WalletType, String> {
    let re = Regex::new(r#"(standard)|(\d+)(of)(\d+)"#).map_err(|e| e.to_string())?;
    let captures = re.captures(wallet_type).map(|captures| {
        captures
            .iter()
            .skip(1)
            .flatten()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
    });
    let wallet_type = match captures.as_deref() {
        Some(["standard"]) => WalletType::Standard,
        Some([x, "of", y]) => WalletType::Multisig(
            x.parse()
                .map_err(|e| format!("cannot parse number: {}", e))?,
            y.parse()
                .map_err(|e| format!("cannot parse number: {}", e))?,
        ),
        _ => return Err(format!("Unknown wallet type: {}", wallet_type)),
    };

    Ok(wallet_type)
}
