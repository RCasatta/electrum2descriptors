use crate::ElectrumExtendedKey;
use bitcoin::util::base58;
use bitcoin::util::bip32::{ChainCode, ChildNumber, ExtendedPubKey, Fingerprint};
use bitcoin::{Network, PublicKey};
use std::convert::TryInto;
use std::str::FromStr;

pub struct ElectrumExtendedPubKey {
    xpub: ExtendedPubKey,
    kind: String,
}

type SentinelMap = Vec<([u8; 4], Network, String)>;
fn initialize_sentinels() -> SentinelMap {
    // electrum testnet
    // https://github.com/spesmilo/electrum/blob/928e43fc530ba5befa062db788e4e04d56324161/electrum/constants.py#L118-L124
    //     XPUB_HEADERS = {
    //         'standard':    0x043587cf,  # tpub
    //         'p2wpkh-p2sh': 0x044a5262,  # upub
    //         'p2wsh-p2sh':  0x024289ef,  # Upub
    //         'p2wpkh':      0x045f1cf6,  # vpub
    //         'p2wsh':       0x02575483,  # Vpub
    //     }
    // electrum mainnet
    // https://github.com/spesmilo/electrum/blob/928e43fc530ba5befa062db788e4e04d56324161/electrum/constants.py#L82-L88
    //     XPUB_HEADERS = {
    //         'standard':    0x0488b21e,  # xpub
    //         'p2wpkh-p2sh': 0x049d7cb2,  # ypub
    //         'p2wsh-p2sh':  0x0295b43f,  # Ypub
    //         'p2wpkh':      0x04b24746,  # zpub
    //         'p2wsh':       0x02aa7ed3,  # Zpub
    //     }

    vec![
        (
            [0x04u8, 0x35, 0x87, 0xcf],
            Network::Testnet,
            "pkh".to_string(),
        ), // tpub
        (
            [0x04u8, 0x4a, 0x52, 0x62],
            Network::Testnet,
            "sh(wpkh".to_string(),
        ), // upub
        (
            [0x02u8, 0x42, 0x89, 0xef],
            Network::Testnet,
            "sh(wsh".to_string(),
        ), // Upub
        (
            [0x04u8, 0x5f, 0x1c, 0xf6],
            Network::Testnet,
            "wpkh".to_string(),
        ), // vpub
        (
            [0x02u8, 0x57, 0x54, 0x83],
            Network::Testnet,
            "wsh".to_string(),
        ), // Vpub
        (
            [0x04u8, 0x88, 0xB2, 0x1E],
            Network::Bitcoin,
            "pkh".to_string(),
        ), // xpub
        (
            [0x04u8, 0x9d, 0x7c, 0xb2],
            Network::Bitcoin,
            "sh(wpkh".to_string(),
        ), // ypub
        (
            [0x02u8, 0x95, 0xb4, 0x3f],
            Network::Bitcoin,
            "sh(wsh".to_string(),
        ), // Ypub
        (
            [0x04u8, 0xb2, 0x47, 0x46],
            Network::Bitcoin,
            "wpkh".to_string(),
        ), // zpub
        (
            [0x02u8, 0xaa, 0x7e, 0xd3],
            Network::Bitcoin,
            "wsh".to_string(),
        ), // Zpub
    ]
}

impl FromStr for ElectrumExtendedPubKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = base58::from_check(s).map_err(|e| e.to_string())?;

        if data.len() != 78 {
            return Err(base58::Error::InvalidLength(data.len()).to_string());
        }

        let cn_int = u32::from_be_bytes(data[9..13].try_into().unwrap());
        let child_number: ChildNumber = ChildNumber::from(cn_int);
        let (network, kind) = match_electrum_xpub(&data[0..4]).map_err(|e| e.to_string())?;

        let xpub = ExtendedPubKey {
            network,
            depth: data[4],
            parent_fingerprint: Fingerprint::from(&data[5..9]),
            child_number,
            chain_code: ChainCode::from(&data[13..45]),
            public_key: PublicKey::from_slice(&data[45..78]).map_err(|e| e.to_string())?,
        };
        Ok(ElectrumExtendedPubKey { xpub, kind })
    }
}

impl ElectrumExtendedKey for ElectrumExtendedPubKey {
    /// Returns the kind
    fn kind(&self) -> &str {
        &self.kind
    }

    /// Returns the xpub as String
    fn xkeystr(&self) -> String {
        self.xpub.to_string()
    }

    /// Returns internal and external descriptor
    fn to_descriptors(&self) -> Vec<String> {
        let xpub = self.xpub.to_string();
        let closing_parenthesis = if self.kind.contains('(') { ")" } else { "" };
        (0..=1)
            .map(|i| format!("{}({}/{}/*){}", self.kind, xpub, i, closing_parenthesis))
            .collect()
    }
}

impl ElectrumExtendedPubKey {
    /// Constructs a new instance
    pub fn new(xpub: ExtendedPubKey, kind: String) -> Self {
        ElectrumExtendedPubKey { xpub, kind }
    }

    /// Returns the xpub
    pub fn xpub(&self) -> &ExtendedPubKey {
        &self.xpub
    }

    /// converts to electrum format
    pub fn electrum_xpub(&self) -> Result<String, String> {
        let sentinels = initialize_sentinels();
        let sentinel = sentinels
            .iter()
            .find(|sent| sent.1 == self.xpub.network && sent.2 == self.kind)
            .ok_or_else(|| "unknown type".to_string())?;
        let mut data = Vec::from(&sentinel.0[..]);
        data.push(self.xpub.depth);
        data.extend(self.xpub.parent_fingerprint.as_bytes());
        let child_number: u32 = self.xpub.child_number.into();
        data.extend(child_number.to_be_bytes());
        data.extend(self.xpub.chain_code.as_bytes());
        data.extend(&self.xpub.public_key.to_bytes());

        if data.len() != 78 {
            return Err(base58::Error::InvalidLength(data.len()).to_string());
        }

        Ok(base58::check_encode_slice(&data))
    }
}

fn match_electrum_xpub(version: &[u8]) -> Result<(Network, String), base58::Error> {
    let sentinels = initialize_sentinels();
    let sentinel = sentinels
        .iter()
        .find(|sent| sent.0 == version)
        .ok_or_else(|| {
            base58::Error::InvalidExtendedKeyVersion(version[0..4].try_into().unwrap())
        })?;
    Ok((sentinel.1, sentinel.2.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::secp256k1::Secp256k1;
    use miniscript::descriptor::DescriptorPublicKey;
    use miniscript::{DescriptorTrait, TranslatePk2};
    use std::str::FromStr;

    #[test]
    fn test_vpub_from_electrum() {
        let electrum_xpub = ElectrumExtendedPubKey::from_str("vpub5VXaSncXqxLbdmvrC4Y8z9CszPwuEscADoetWhfrxDFzPUbL5nbVtanYDkrVEutkv9n5A5aCcvRC9swbjDKgHjCZ2tAeae8VsBuPbS8KpXv").unwrap();
        assert_eq!(electrum_xpub.xpub.to_string(),"tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp");
        assert_eq!(electrum_xpub.kind, "wpkh");
        let descriptors = electrum_xpub.to_descriptors();
        assert_eq!(descriptors[0], "wpkh(tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp/0/*)");
        assert_eq!(descriptors[1], "wpkh(tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp/1/*)");
        let xpub = electrum_xpub.xpub();
        assert_eq!(xpub.to_string(), "tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp");
    }

    #[test]
    fn test_vpub_to_electrum() {
        let electrum_xpub = ElectrumExtendedPubKey::new(
            ExtendedPubKey::from_str("tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp").unwrap(), 
            "wpkh".to_string(),
        );
        assert_eq!(electrum_xpub.xpub.to_string(),"tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp");
        assert_eq!(electrum_xpub.kind, "wpkh");
        assert_eq!(electrum_xpub.electrum_xpub().unwrap(), "vpub5VXaSncXqxLbdmvrC4Y8z9CszPwuEscADoetWhfrxDFzPUbL5nbVtanYDkrVEutkv9n5A5aCcvRC9swbjDKgHjCZ2tAeae8VsBuPbS8KpXv");
    }

    #[test]
    fn test_vpub_roundtrip() {
        let elxpub = "vpub5VXaSncXqxLbdmvrC4Y8z9CszPwuEscADoetWhfrxDFzPUbL5nbVtanYDkrVEutkv9n5A5aCcvRC9swbjDKgHjCZ2tAeae8VsBuPbS8KpXv";
        let electrum_xpub = ElectrumExtendedPubKey::from_str(elxpub).unwrap();
        assert_eq!(electrum_xpub.electrum_xpub().unwrap(), elxpub);
        assert_ne!(elxpub, electrum_xpub.xpub.to_string());
    }

    #[test]
    fn test_slip121_vectors() {
        // from https://github.com/satoshilabs/slips/blob/master/slip-0132.md
        test_first_address("xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj","1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA");
        test_first_address("ypub6Ww3ibxVfGzLrAH1PNcjyAWenMTbbAosGNB6VvmSEgytSER9azLDWCxoJwW7Ke7icmizBMXrzBx9979FfaHxHcrArf3zbeJJJUZPf663zsP","37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf");
        test_first_address("zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs","bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu");
    }

    fn test_first_address(electrum_xpub: &str, expected_first_address: &str) {
        let electrum_xpub = ElectrumExtendedPubKey::from_str(electrum_xpub).unwrap();
        let descriptors = electrum_xpub.to_descriptors();
        let descriptor: miniscript::Descriptor<DescriptorPublicKey> =
            descriptors[0].parse().unwrap();
        let first_address = descriptor
            .derive(0)
            .translate_pk2(|xpk| xpk.derive_public_key(&Secp256k1::verification_only()))
            .unwrap()
            .address(electrum_xpub.xpub.network)
            .unwrap()
            .to_string();
        assert_eq!(expected_first_address, first_address);
    }
}
