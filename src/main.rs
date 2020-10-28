use bitcoin::util::bip32::{ExtendedPubKey, ChildNumber, Fingerprint, ChainCode};
use std::str::FromStr;
use bitcoin::util::base58;
use bitcoin::{Network, PublicKey};
use std::convert::TryInto;

struct ElectrumExtendedPubKey {
    xpub: ExtendedPubKey,
    kind: String,
}

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    args.next(); // first is program name
    let electrum_xpub = args.next().ok_or_else(|| "You must specify an electrum xpub as first argument".to_string())?;
    let electrum_xpub = ElectrumExtendedPubKey::from_str(&electrum_xpub)?;
    println!("{}", electrum_xpub.to_descriptor());
    Ok(())
}

impl FromStr for ElectrumExtendedPubKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = base58::from_check(s).map_err(|e| e.to_string())?;

        if data.len() != 78 {
            return Err( base58::Error::InvalidLength(data.len()).to_string() );
        }

        let cn_int = u32::from_be_bytes(data[9..13].try_into().unwrap());
        let child_number: ChildNumber = ChildNumber::from(cn_int);
        let (network, kind) = match_electrum_xpub(&data[0..4]).map_err(|e| e.to_string())?;

        let xpub = ExtendedPubKey {
            network: network,
            depth: data[4],
            parent_fingerprint: Fingerprint::from(&data[5..9]),
            child_number: child_number,
            chain_code: ChainCode::from(&data[13..45]),
            public_key: PublicKey::from_slice(
                &data[45..78]).map_err(|e|
                base58::Error::Other(e.to_string())).map_err(|e| e.to_string())?,
        };
        Ok(ElectrumExtendedPubKey { xpub, kind })
    }
}

impl ElectrumExtendedPubKey {
    /// Returns internal and external descriptor
    pub fn to_descriptor(&self) -> String {
        let xpub = self.xpub.to_string();
        let closing_parenthesis = if  self.kind.contains('(') {
            ")"
        } else {
            ""
        };
        format!(
            "{}({}/0/*){}\n{}({}/1/*){}",
            self.kind, xpub, closing_parenthesis,  self.kind, xpub, closing_parenthesis
        )
    }
}

fn match_electrum_xpub(version: &[u8]) -> Result<(Network, String), base58::Error> {
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
    match version {
        [0x04u8, 0x35, 0x87, 0xcf] => Ok((Network::Testnet, "pkh".to_string())), // tpub
        [0x04u8, 0x4a, 0x52, 0x62] => Ok((Network::Testnet, "sh(wpkh".to_string())), // upub
        [0x02u8, 0x42, 0x89, 0xef] => Ok((Network::Testnet, "sh(wsh".to_string())),  // Upub
        [0x04u8, 0x5f, 0x1c, 0xf6] => Ok((Network::Testnet, "wpkh".to_string())),      // vpub
        [0x02u8, 0x57, 0x54, 0x83] => Ok((Network::Testnet, "wsh".to_string())),       // Vpub
        [0x04u8, 0x88, 0xB2, 0x1E] => Ok((Network::Bitcoin, "pkh".to_string())), // xpub
        [0x04u8, 0x9d, 0x7c, 0xb2] => Ok((Network::Bitcoin, "sh(wpkh".to_string())), // ypub
        [0x02u8, 0x95, 0xb4, 0x3f] => Ok((Network::Bitcoin, "sh(wsh".to_string())),  // Ypub
        [0x04u8, 0xb2, 0x47, 0x46] => Ok((Network::Bitcoin, "wpkh".to_string())),      // zpub
        [0x02u8, 0xaa, 0x7e, 0xd3] => Ok((Network::Bitcoin, "wsh".to_string())),       // Zpub
        _ => return Err(base58::Error::InvalidVersion(version.to_vec())),
    }
}

#[cfg(test)]
mod tests {
    use crate::ElectrumExtendedPubKey;
    use std::str::FromStr;

    #[test]
    fn test_vpub() {
        let electrum_xpub = ElectrumExtendedPubKey::from_str("vpub5VXaSncXqxLbdmvrC4Y8z9CszPwuEscADoetWhfrxDFzPUbL5nbVtanYDkrVEutkv9n5A5aCcvRC9swbjDKgHjCZ2tAeae8VsBuPbS8KpXv").unwrap();
        assert_eq!(electrum_xpub.xpub.to_string(),"tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp");
        assert_eq!(electrum_xpub.kind,"wpkh");
        assert_eq!(electrum_xpub.to_descriptor(),"wpkh(tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp/0/*)\nwpkh(tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp/1/*)");
    }
}