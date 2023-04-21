use crate::{ElectrumExtendedKey, ElectrumExtendedPrivKey, ElectrumExtendedPubKey};
use bitcoin::bip32::{ExtendedPrivKey, ExtendedPubKey};
use regex::Regex;
use serde::{de, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, io::BufReader, path::Path, str::FromStr, string::ToString};

/// Representation of an electrum wallet file. Has custom serialization and de-serialization routines to more accurately represent what we need, and the electrum wallet file format.
#[derive(Clone, Debug, PartialEq)]
pub struct ElectrumWalletFile {
    addresses: Addresses,
    wallet_type: WalletType,
    keystores: Vec<Keystore>,
}

impl ElectrumWalletFile {
    /// Construct a wallet
    pub fn new(keystores: &[Keystore], min_signatures: u8) -> Result<Self, String> {
        let wallet = if keystores.len() == 1 {
            ElectrumWalletFile {
                addresses: Addresses::new(),
                wallet_type: WalletType::Standard,
                keystores: keystores.to_vec(),
            }
        } else if keystores.len() >= 255 {
            return Err(format!(
                "keystore sizes aboce 255 are not currently supported. {}",
                keystores.len()
            ));
        } else {
            ElectrumWalletFile {
                addresses: Addresses::new(),
                wallet_type: WalletType::Multisig(min_signatures, keystores.len() as u8),
                keystores: keystores.to_vec(),
            }
        };
        wallet.validate()?;
        Ok(wallet)
    }

    /// Getter for addresses
    pub fn addresses(&self) -> &Addresses {
        &self.addresses
    }

    /// Getter for wallet_type
    pub fn wallet_type(&self) -> &WalletType {
        &self.wallet_type
    }

    /// Getter for keystores
    pub fn keystores(&self) -> &Vec<Keystore> {
        &self.keystores
    }

    /// Parse an electrum wallet file
    pub fn from_file(wallet_file: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(wallet_file).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let wallet = serde_json::from_reader(reader).map_err(|e| e.to_string())?;
        Ok(wallet)
    }

    /// Write to an electrum wallet file
    pub fn to_file(&self, wallet_file: &Path) -> Result<(), String> {
        let file = std::fs::File::create(wallet_file).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, self).map_err(|e| e.to_string())
    }

    /// Construct from an output descriptor. Only the external descriptor is needed, the change descriptor is implied.
    pub fn from_descriptor(desc: &str) -> Result<Self, String> {
        let wallet = if desc.contains("(sortedmulti(") {
            ElectrumWalletFile::from_descriptor_multisig(desc)
        } else {
            ElectrumWalletFile::from_descriptor_singlesig(desc)
        }?;
        wallet.validate()?;
        Ok(wallet)
    }

    /// Construct from a single signature output descriptor. Only the external descriptor is needed, the change descriptor is implied.
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

        Ok(ElectrumWalletFile {
            addresses: Addresses::new(),
            keystores: vec![keystore],
            wallet_type: WalletType::Standard,
        })
    }

    /// Construct from a multisig output descriptor. Only the external descriptor is needed, the change descriptor is implied.
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

            Ok(ElectrumWalletFile {
                addresses: Addresses::new(),
                keystores,
                wallet_type: WalletType::Multisig(x.parse().unwrap(), y as u8),
            })
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
                let exkey = self.keystores[0].get_xkey()?;
                let desc_ext = exkey.kind().to_string() + "(" + &exkey.xkey_str() + "/0/*)";
                let desc_chg = exkey.kind().to_string() + "(" + &exkey.xkey_str() + "/1/*)";
                Ok(vec![desc_ext, desc_chg])
            }
            WalletType::Multisig(x, _y) => {
                let xkeys = self
                    .keystores
                    .iter()
                    .map(|ks| ks.get_xkey())
                    .collect::<Result<Vec<Box<dyn ElectrumExtendedKey>>, _>>()?;
                let prefix = match xkeys[0].kind() as &str {
                    "pkh" => "sh",
                    kind => kind,
                }
                .to_string();
                let prefix = format!("{}(sortedmulti({}", prefix, x);

                let mut desc = xkeys.iter().fold(prefix, |acc, exkey| {
                    acc + &(",".to_string() + &exkey.xkey_str() + "/0/*")
                });
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

    /// validate the internal structure
    fn validate(&self) -> Result<(), String> {
        let expected_keystores: usize = match self.wallet_type {
            WalletType::Standard => 1,
            WalletType::Multisig(_x, y) => y.into(),
        };

        if self.keystores.len() != expected_keystores {
            return Err(format!(
                "Wrong number of keystores: {}; expected: {}",
                self.keystores.len(),
                expected_keystores
            ));
        }

        if let WalletType::Multisig(x, _y) = self.wallet_type {
            if x as usize > expected_keystores {
                return Err(format!(
                    "Minimum number of signatures {} must not be greater than keystores {}",
                    x, expected_keystores
                ));
            }
        }

        Ok(())
    }
}

impl Serialize for ElectrumWalletFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // We don't know the length of the map at this point, so it's None
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("addresses", &self.addresses)?;
        map.serialize_entry("wallet_type", &self.wallet_type)?;
        match self.wallet_type {
            WalletType::Standard => {
                map.serialize_entry("keystore", &self.keystores[0])?;
            }
            WalletType::Multisig(_x, _y) => {
                self.keystores
                    .iter()
                    .enumerate()
                    .map(|(i, keystore)| {
                        let key = format!("x{}/", i + 1);
                        map.serialize_entry(&key, &keystore)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for ElectrumWalletFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Addrs,
            Keyst,
            WalTyp,
            Ignore,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(
                            "`addresses` or `keystore` or `wallet_type` or 'x1/` or `x2/`",
                        )
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        let re = Regex::new(r#"(x)(\d+)(/)|([a-z_\-0-9]+)"#).unwrap();
                        let captures = re.captures(value).map(|captures| {
                            captures
                                .iter()
                                .skip(1)
                                .flatten()
                                .map(|c| c.as_str())
                                .collect::<Vec<_>>()
                        });
                        match captures.as_deref() {
                            Some(["x", _i, "/"]) => Ok(Field::Keyst),
                            Some(["keystore"]) => Ok(Field::Keyst),
                            Some(["addresses"]) => Ok(Field::Addrs),
                            Some(["wallet_type"]) => Ok(Field::WalTyp),
                            _ => Ok(Field::Ignore),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ElectrumWalletFileVisitor;

        impl<'de> de::Visitor<'de> for ElectrumWalletFileVisitor {
            type Value = ElectrumWalletFile;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ElectrumWalletFile")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ElectrumWalletFile, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut addresses = Addresses::new();
                let mut keystores = Vec::new();
                let mut wallet_type = WalletType::Standard;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Addrs => {
                            addresses = map.next_value()?;
                        }
                        Field::Keyst => {
                            keystores.push(map.next_value()?);
                        }
                        Field::WalTyp => {
                            wallet_type = map.next_value()?;
                        }
                        Field::Ignore => {
                            let _ignore = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let wallet = ElectrumWalletFile {
                    addresses,
                    keystores,
                    wallet_type,
                };
                wallet.validate().map_err(de::Error::custom)?;
                Ok(wallet)
            }
        }

        const FIELDS: &[&str] = &[
            "addresses",
            "addr_history",
            "channel_backups",
            "keystore",
            "wallet_type",
            "x1/",
            "x2/",
            "x3/",
        ];
        deserializer.deserialize_struct("ElectrumWalletFile", FIELDS, ElectrumWalletFileVisitor)
    }
}

/// Representation of the addresses section of an electrum wallet file
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Addresses {
    pub change: Vec<String>,
    pub receiving: Vec<String>,
}

impl Addresses {
    fn new() -> Self {
        Addresses {
            change: Vec::new(),
            receiving: Vec::new(),
        }
    }
}

/// Representation of a keystore section of an electrum wallet file. Can be single sig "keystore" or multisig "x1/" "x2/" ...
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Keystore {
    #[serde(default = "Keystore::default_type")]
    pub r#type: String,
    pub xprv: Option<String>,
    pub xpub: String,
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
            ElectrumExtendedPubKey::new(ExtendedPubKey::from_priv(&secp, &xprv), kind.to_string())
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

/// Representation of the wallet_type section of an electrum wallet file. Has custom serialization and de-serialization implementatoin.
#[derive(Clone, Debug, PartialEq)]
pub enum WalletType {
    Standard,
    Multisig(u8, u8),
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
