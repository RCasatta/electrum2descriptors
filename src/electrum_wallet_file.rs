use crate::{ElectrumExtendedKey, ElectrumExtendedPrivKey, ElectrumExtendedPubKey};
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use regex::Regex;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, path::Path, str::FromStr, string::ToString};

/// Representation of an electrum wallet file
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Addresses {
    pub change: Vec<String>,
    pub receiving: Vec<String>,
}

/// Representation of a keystore section of an electrum wallet file. Can be single sig "keystore" or multisig "x1/" "x2/" ...
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Keystore {
    #[serde(default = "Keystore::default_type")]
    pub r#type: String,
    pub xprv: Option<String>,
    pub xpub: String,
}

/// Representation of the wallet_type section of an electrum wallet file
#[derive(Clone, Debug, PartialEq)]
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

    /// Convert from an output descriptor. Only the external descriptor is needed, the change descriptor is implied.
    pub fn from_descriptor(desc: &str) -> Result<Self, String> {
        if desc.contains("sortedmulti") {
            ElectrumWalletFile::from_descriptor_multisig(desc)
        } else {
            ElectrumWalletFile::from_descriptor_singlesig(desc)
        }
    }

    fn from_descriptor_singlesig(desc: &str) -> Result<Self, String> {
        let re =
            Regex::new(r#"(pkh|sh\(wpkh|sh\(wsh|wpkh|wsh)\((([tx]p(ub|rv)[0-9A-Za-z]+)/0/\*)\)+"#)
                .map_err(|e| e.to_string())?;
        let captures = re.captures(desc).map(|captures| {
            captures
                .iter()
                .skip(1)
                .take(3)
                .flatten()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
        });
        let keystore = match captures.as_deref() {
            Some([kind, _, xkey]) => Keystore::new(kind, xkey)?,
            _ => return Err(format!("Unknown descriptor format: {:?}", captures)),
        };

        let wallet = ElectrumWalletFile {
            addresses: Addresses::new(),
            keystore: Some(keystore),
            wallet_type: WalletType::Standard,
            x1: None,
            x2: None,
            x3: None,
            x4: None,
            x5: None,
            x6: None,
            x7: None,
        };
        Ok(wallet)
    }

    fn from_descriptor_multisig(desc: &str) -> Result<Self, String> {
        let re = Regex::new(
            r#"(sh|sh\(wsh|wsh)\(sortedmulti\((\d),([tx]p(ub|rv)[0-9A-Za-z]+/0/\*,?)+\)+"#,
        )
        .map_err(|e| e.to_string())?;
        let captures = re.captures(desc).map(|captures| {
            captures
                .iter()
                .skip(1)
                .take(2)
                .flatten()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
        });
        if let Some([kind, x]) = captures.as_deref() {
            let kind = match *kind {
                "wsh" => "wsh",
                "sh" => "pkh",
                "sh(wsh" => "sh(wsh",
                _ => return Err(format!("unknown nultisig kind: {}", kind)),
            };
            let re = Regex::new(r#"[tx]p[ur][bv][0-9A-Za-z]+"#).map_err(|e| e.to_string())?;
            let keystores = re
                .captures_iter(desc)
                .map(|cap| Keystore::new(kind, &cap[0]))
                .collect::<Result<Vec<Keystore>, _>>()?;
            let y = keystores.len();
            if y < 2 {
                return Err(
                    "Multisig with less than two signers doesn't make a lot of sense".to_string(),
                );
            }

            let wallet = ElectrumWalletFile {
                addresses: Addresses::new(),
                keystore: None,
                wallet_type: WalletType::Multisig(x.parse().unwrap(), y as u8),
                x1: Some(keystores[0].clone()),
                x2: Some(keystores[1].clone()),
                x3: if y >= 3 {
                    Some(keystores[2].clone())
                } else {
                    None
                },
                x4: if y >= 4 {
                    Some(keystores[3].clone())
                } else {
                    None
                },
                x5: if y >= 5 {
                    Some(keystores[4].clone())
                } else {
                    None
                },
                x6: if y >= 6 {
                    Some(keystores[5].clone())
                } else {
                    None
                },
                x7: if y >= 7 {
                    Some(keystores[6].clone())
                } else {
                    None
                },
            };
            Ok(wallet)
        } else {
            Err(format!(
                "Unknown multisig descriptor format: {:?}",
                captures
            ))
        }
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

impl Addresses {
    fn new() -> Self {
        Addresses {
            change: Vec::new(),
            receiving: Vec::new(),
        }
    }
}

impl Keystore {
    /// Construct a Keystore from script kind and xpub or xprv
    fn new(kind: &str, xkey: &str) -> Result<Self, String> {
        let xprv = ExtendedPrivKey::from_str(xkey);
        let exprv = if let Ok(xprv) = xprv {
            Some(ElectrumExtendedPrivKey::new(xprv, kind.to_string()).electrum_xprv()?)
        } else {
            None
        };

        let expub = if let Ok(xprv) = xprv {
            let secp = bitcoin::secp256k1::Secp256k1::new();
            ElectrumExtendedPubKey::new(
                ExtendedPubKey::from_private(&secp, &xprv),
                kind.to_string(),
            )
        } else {
            ElectrumExtendedPubKey::new(
                ExtendedPubKey::from_str(xkey).map_err(|e| e.to_string())?,
                kind.to_string(),
            )
        }
        .electrum_xpub()?;

        Ok(Keystore {
            r#type: Keystore::default_type(),
            xprv: exprv,
            xpub: expub,
        })
    }

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
