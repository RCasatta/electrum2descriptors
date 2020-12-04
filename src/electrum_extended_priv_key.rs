use crate::ElectrumExtendedKey;
use bitcoin::secp256k1;
use bitcoin::util::base58;
use bitcoin::util::bip32::{ChainCode, ChildNumber, ExtendedPrivKey, Fingerprint};
use bitcoin::{Network, PrivateKey};
use std::convert::TryInto;
use std::str::FromStr;

pub struct ElectrumExtendedPrivKey {
    xprv: ExtendedPrivKey,
    kind: String,
}

impl FromStr for ElectrumExtendedPrivKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = base58::from_check(s).map_err(|e| e.to_string())?;

        if data.len() != 78 {
            return Err(base58::Error::InvalidLength(data.len()).to_string());
        }

        let cn_int = u32::from_be_bytes(data[9..13].try_into().unwrap());
        let child_number: ChildNumber = ChildNumber::from(cn_int);
        let (network, kind) = match_electrum_xprv(&data[0..4]).map_err(|e| e.to_string())?;
        let key = secp256k1::SecretKey::from_slice(&data[46..78]).map_err(|e| e.to_string())?;

        let xprv = ExtendedPrivKey {
            network,
            depth: data[4],
            parent_fingerprint: Fingerprint::from(&data[5..9]),
            child_number,
            chain_code: ChainCode::from(&data[13..45]),
            private_key: PrivateKey {
                compressed: true,
                network,
                key,
            },
        };
        Ok(ElectrumExtendedPrivKey { xprv, kind })
    }
}

impl ElectrumExtendedKey for ElectrumExtendedPrivKey {
    /// Returns internal and external descriptor
    fn to_descriptors(&self) -> Vec<String> {
        let xprv = self.xprv.to_string();
        let closing_parenthesis = if self.kind.contains('(') { ")" } else { "" };
        (0..=1)
            .map(|i| format!("{}({}/{}/*){}", self.kind, xprv, i, closing_parenthesis))
            .collect()
    }
}

impl ElectrumExtendedPrivKey {
    /// Returns the xprv
    pub fn xprv(&self) -> &ExtendedPrivKey {
        &self.xprv
    }
}

fn match_electrum_xprv(version: &[u8]) -> Result<(Network, String), base58::Error> {
    // electrum testnet
    // https://github.com/spesmilo/electrum/blob/928e43fc530ba5befa062db788e4e04d56324161/electrum/constants.py#L110-L116
    //     XPRV_HEADERS = {
    //         'standard':    0x04358394,  # tprv
    //         'p2wpkh-p2sh': 0x044a4e28,  # uprv
    //         'p2wsh-p2sh':  0x024285b5,  # Uprv
    //         'p2wpkh':      0x045f18bc,  # vprv
    //         'p2wsh':       0x02575048,  # Vprv
    //     }
    // electrum mainnet
    // https://github.com/spesmilo/electrum/blob/928e43fc530ba5befa062db788e4e04d56324161/electrum/constants.py#L74-L80
    //     XPRV_HEADERS = {
    //         'standard':    0x0488ade4,  # xprv
    //         'p2wpkh-p2sh': 0x049d7878,  # yprv
    //         'p2wsh-p2sh':  0x0295b005,  # Yprv
    //         'p2wpkh':      0x04b2430c,  # zprv
    //         'p2wsh':       0x02aa7a99,  # Zprv
    //     }
    match version {
        [0x04u8, 0x35, 0x83, 0x94] => Ok((Network::Testnet, "pkh".to_string())), // tprv
        [0x04u8, 0x4a, 0x4e, 0x28] => Ok((Network::Testnet, "sh(wpkh".to_string())), // uprv
        [0x02u8, 0x42, 0x85, 0xb5] => Ok((Network::Testnet, "sh(wsh".to_string())), // Uprv
        [0x04u8, 0x5f, 0x18, 0xbc] => Ok((Network::Testnet, "wpkh".to_string())), // vprv
        [0x02u8, 0x57, 0x50, 0x48] => Ok((Network::Testnet, "wsh".to_string())), // Vprv
        [0x04u8, 0x88, 0xad, 0xE4] => Ok((Network::Bitcoin, "pkh".to_string())), // xprv
        [0x04u8, 0x9d, 0x78, 0x78] => Ok((Network::Bitcoin, "sh(wpkh".to_string())), // yprv
        [0x02u8, 0x95, 0xb0, 0x05] => Ok((Network::Bitcoin, "sh(wsh".to_string())), // Yprv
        [0x04u8, 0xb2, 0x43, 0x0c] => Ok((Network::Bitcoin, "wpkh".to_string())), // zprv
        [0x02u8, 0xaa, 0x7a, 0x99] => Ok((Network::Bitcoin, "wsh".to_string())), // Zprv
        _ => Err(base58::Error::InvalidVersion(version.to_vec())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_vprv() {
        let electrum_xprv = ElectrumExtendedPrivKey::from_str("yprvAHwhK6RbpuS3dgCYHM5jc2ZvEKd7Bi61u9FVhYMpgMSuZS613T1xxQeKTffhrHY79hZ5PsskBjcc6C2V7DrnsMsNaGDaWev3GLRQRgV7hxF").unwrap();
        assert_eq!(electrum_xprv.xprv.to_string(),"xprv9y7S1RkggDtZnP1RSzJ7PwUR4MUfF66Wz2jGv9TwJM52WLGmnnrQLLzBSTi7rNtBk4SGeQHBj5G4CuQvPXSn58BmhvX9vk6YzcMm37VuNYD");
        assert_eq!(electrum_xprv.kind, "sh(wpkh");
        let descriptors = electrum_xprv.to_descriptors();
        assert_eq!(descriptors[0], "sh(wpkh(xprv9y7S1RkggDtZnP1RSzJ7PwUR4MUfF66Wz2jGv9TwJM52WLGmnnrQLLzBSTi7rNtBk4SGeQHBj5G4CuQvPXSn58BmhvX9vk6YzcMm37VuNYD/0/*))");
        assert_eq!(descriptors[1], "sh(wpkh(xprv9y7S1RkggDtZnP1RSzJ7PwUR4MUfF66Wz2jGv9TwJM52WLGmnnrQLLzBSTi7rNtBk4SGeQHBj5G4CuQvPXSn58BmhvX9vk6YzcMm37VuNYD/1/*))");
        let xprv = electrum_xprv.xprv();
        assert_eq!(xprv.to_string(), "xprv9y7S1RkggDtZnP1RSzJ7PwUR4MUfF66Wz2jGv9TwJM52WLGmnnrQLLzBSTi7rNtBk4SGeQHBj5G4CuQvPXSn58BmhvX9vk6YzcMm37VuNYD");
    }
}
