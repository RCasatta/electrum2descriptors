use crate::{ElectrumExtendedKey, ElectrumExtendedPrivKey, ElectrumExtendedPubKey};
use regex::Regex;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;

/// Representation of an electrum wallet file
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ElectrumWalletFile {
    pub addresses: Addresses,
    pub keystore: Option<Keystore>,
    pub wallet_type: WalletType,
    #[serde(default, rename = "x1/")]
    pub x1: Option<Keystore>,
    #[serde(default, rename = "x2/")]
    pub x2: Option<Keystore>,
    #[serde(default, rename = "x3/")]
    pub x3: Option<Keystore>,
    #[serde(default, rename = "x4/")]
    pub x4: Option<Keystore>,
    #[serde(default, rename = "x5/")]
    pub x5: Option<Keystore>,
    #[serde(default, rename = "x6/")]
    pub x6: Option<Keystore>,
    #[serde(default, rename = "x7/")]
    pub x7: Option<Keystore>,
}

/// Representation of the addresses section of an electrum wallet file
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Addresses {
    pub change: Vec<String>,
    pub receiving: Vec<String>,
}

/// Representation of a keystore section of an electrum wallet file. Can be single sig "keystore" or multisig "x1/" "x2/" ...
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Keystore {
    #[serde(default = "Keystore::default_type")]
    pub r#type: String,
    pub xprv: Option<String>,
    pub xpub: String,
}

/// Representation of the wallet_type section of an electrum wallet file
#[derive(Clone, Debug)]
pub enum WalletType {
    Standard,
    Multisig(u8, u8),
}

impl ElectrumWalletFile {
    /// Parse an electrum wallet file
    pub fn from_file(wallet_file: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(wallet_file).map_err(|e| e.to_string())?;
        let wallet = serde_json::from_reader(file).map_err(|e| e.to_string())?;
        Ok(wallet)
    }

    /// Write to an electrum wallet file
    pub fn to_file(&self, wallet_file: &Path) -> Result<(), String> {
        let file = std::fs::File::create(wallet_file).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, self).map_err(|e| e.to_string())
    }

    /// Generate output descriptors matching the electrum wallet
    pub fn to_descriptors(&self) -> Result<Vec<String>, String> {
        match self.wallet_type {
            WalletType::Standard => {
                let exkey = self
                    .keystore
                    .as_ref()
                    .ok_or("missing keystore")?
                    .get_xkey()?;
                let desc_ext = exkey.kind().to_string() + "(" + &exkey.xkeystr() + "/0/*)";
                let desc_chg = exkey.kind().to_string() + "(" + &exkey.xkeystr() + "/1/*)";
                Ok(vec![desc_ext, desc_chg])
            }
            WalletType::Multisig(x, y) => {
                let mut desc = String::new();
                for i in 0..y {
                    let query = format!("x{}", i + 1);
                    let keystore = match query.as_str() {
                        "x1" => &self.x1,
                        "x2" => &self.x2,
                        "x3" => &self.x3,
                        "x4" => &self.x4,
                        "x5" => &self.x5,
                        "x6" => &self.x6,
                        "x7" => &self.x7,
                        _ => {
                            return Err("unknown keystore".to_string());
                        }
                    }
                    .as_ref()
                    .ok_or(format!("missing keystore: {}", query))?;
                    let exkey = keystore.get_xkey()?;
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
}

impl Keystore {
    /// Get the xprv if available or else the xpub.
    fn get_xkey(&self) -> Result<Box<dyn ElectrumExtendedKey>, String> {
        if let Some(xprv) = &self.xprv {
            let exprv = ElectrumExtendedPrivKey::from_str(xprv)?;
            return Ok(Box::new(exprv));
        }

        let expub = ElectrumExtendedPubKey::from_str(&self.xpub)?;
        Ok(Box::new(expub))
    }

    /// Default keystore type to use if nothing else was specified
    fn default_type() -> String {
        "bip32".to_string()
    }
}

impl fmt::Display for WalletType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for WalletType {
    type Err = String;

    /// Parse WalletType from a string representation
    fn from_str(wallet_type: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r#"(standard)|(\d+)(of)(\d+)"#).map_err(|e| e.to_string())?;
        let captures = re.captures(wallet_type).map(|captures| {
            captures
                .iter()
                .skip(1)
                .flatten()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
        });
        match captures.as_deref() {
            Some(["standard"]) => Ok(WalletType::Standard),
            Some([x, "of", y]) => Ok(WalletType::Multisig(x.parse().unwrap(), y.parse().unwrap())),
            _ => Err(format!("Unknown wallet type: {}", wallet_type)),
        }
    }
}

impl<'de> Deserialize<'de> for WalletType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        WalletType::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for WalletType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match *self {
            WalletType::Standard => "standard".to_string(),
            WalletType::Multisig(x, y) => format!("{}of{}", x, y),
        };
        serializer.serialize_str(&s)
    }
}
