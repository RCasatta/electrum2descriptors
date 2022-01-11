#[cfg(feature = "wallet_file")]
use bdk::{bitcoin::Network, database::MemoryDatabase, wallet::AddressIndex, Wallet};
#[cfg(feature = "wallet_file")]
use libelectrum2descriptors::ElectrumWalletFile;
#[cfg(feature = "wallet_file")]
use rstest::rstest;
#[cfg(feature = "wallet_file")]
use std::path::{Path, PathBuf};
#[cfg(feature = "wallet_file")]
use tempfile::tempdir;

#[cfg(feature = "wallet_file")]
#[rstest]
#[case::default_legacy("default_legacy", 
    "pkh(tprv8ZgxMBicQKsPeYnCHtn5QZqhTgkkDmXebfQMXWmX7ThXJFCbzDTKFNRsB43GUmHzu2pdGcnnegFy175kFcgZQYC5BFPnRdYDPQyqetpyjb5/0/*)",
    "pkh(tprv8ZgxMBicQKsPeYnCHtn5QZqhTgkkDmXebfQMXWmX7ThXJFCbzDTKFNRsB43GUmHzu2pdGcnnegFy175kFcgZQYC5BFPnRdYDPQyqetpyjb5/1/*)")]
#[case::default_legacy_watch("default_legacy_watch", 
    "pkh(tpubD6NzVbkrYhZ4Y1ozBYSfoyVp2iGgP6iZAy18p2opXjVv8jTNccGuRs3jMCMe4ncfwy2RUJsoZLSXsGiFhN47xFbJgtRvCuV3RP3UnxpsrZt/0/*)",
    "pkh(tpubD6NzVbkrYhZ4Y1ozBYSfoyVp2iGgP6iZAy18p2opXjVv8jTNccGuRs3jMCMe4ncfwy2RUJsoZLSXsGiFhN47xFbJgtRvCuV3RP3UnxpsrZt/1/*)")]
#[case::default_segwit("default_segwit", 
    "wpkh(tprv8cvkZzx9zA7EfFDbH945mK23r7hg6EHXUk79wVUSRukwyctFS1AdpSpkZcykAMDveCj8RA3R4jwFTKMwMbWexJox8NMqq7YphJLDumfCSfu/0/*)",
    "wpkh(tprv8cvkZzx9zA7EfFDbH945mK23r7hg6EHXUk79wVUSRukwyctFS1AdpSpkZcykAMDveCj8RA3R4jwFTKMwMbWexJox8NMqq7YphJLDumfCSfu/1/*)")]
#[case::multisig_hw_segwit("multisig_hw_segwit", 
    "wsh(sortedmulti(2,tpubDEcw4ooTbmw62zBKdkYepoP3z4WWugdeRzPHHAbk8XVsPfBE9AAZMNghiqwtdFgtabaeppBTPmezUkRkQZidLcSJp3XTASbMakHcYauWehZ/0/*,tpubDEbkvhmJoZMq3SUNqEf3aEsubvqsCUPc7rroHkGERgS7qA1gQVMxUPrgzth6x43odirLohwf4aMHpvcnWi3jCB2xkizv8T4B2KqLRZVLC6K/0/*))",
    "wsh(sortedmulti(2,tpubDEcw4ooTbmw62zBKdkYepoP3z4WWugdeRzPHHAbk8XVsPfBE9AAZMNghiqwtdFgtabaeppBTPmezUkRkQZidLcSJp3XTASbMakHcYauWehZ/1/*,tpubDEbkvhmJoZMq3SUNqEf3aEsubvqsCUPc7rroHkGERgS7qA1gQVMxUPrgzth6x43odirLohwf4aMHpvcnWi3jCB2xkizv8T4B2KqLRZVLC6K/1/*))")]
#[case::multisig_legacy("multisig_legacy", 
    "sh(sortedmulti(2,tprv8ZgxMBicQKsPeLPWr5WbJDAhANr6irc1Yf7eUNCYjGYap27HU4bDBXWGMT3X75FhDyxNXr6pK4QeHcCBvkqchQzK8wZ4JbGv5X5MWtXQtqy/0/*,tpubD6NzVbkrYhZ4Y1ozBYSfoyVp2iGgP6iZAy18p2opXjVv8jTNccGuRs3jMCMe4ncfwy2RUJsoZLSXsGiFhN47xFbJgtRvCuV3RP3UnxpsrZt/0/*))",
    "sh(sortedmulti(2,tprv8ZgxMBicQKsPeLPWr5WbJDAhANr6irc1Yf7eUNCYjGYap27HU4bDBXWGMT3X75FhDyxNXr6pK4QeHcCBvkqchQzK8wZ4JbGv5X5MWtXQtqy/1/*,tpubD6NzVbkrYhZ4Y1ozBYSfoyVp2iGgP6iZAy18p2opXjVv8jTNccGuRs3jMCMe4ncfwy2RUJsoZLSXsGiFhN47xFbJgtRvCuV3RP3UnxpsrZt/1/*))")]
#[case::multisig_segwit("multisig_segwit", 
    "wsh(sortedmulti(2,tprv8dNybiDsdyms39SAWTxyiNHABTTgiqmJpScmxGrdKEuZ7TwXcaYXT4f4ddVjWiiQs9zowHqyDmvaebN6fU2Lu6iAYnYuepiLkvzGdcZZi8D/0/*,tpubD9cniQzQ8XnuagyP9Xwg3sWCX77wQPWoLPW7jqzcPn37r8hq2X86uztCEyFbMY16amzwdJ1CcNRXhF3vykn1wuDv2ULzryRtaCcN5Cr8F9y/0/*))",
    "wsh(sortedmulti(2,tprv8dNybiDsdyms39SAWTxyiNHABTTgiqmJpScmxGrdKEuZ7TwXcaYXT4f4ddVjWiiQs9zowHqyDmvaebN6fU2Lu6iAYnYuepiLkvzGdcZZi8D/1/*,tpubD9cniQzQ8XnuagyP9Xwg3sWCX77wQPWoLPW7jqzcPn37r8hq2X86uztCEyFbMY16amzwdJ1CcNRXhF3vykn1wuDv2ULzryRtaCcN5Cr8F9y/1/*))")]
#[case::multisig_wrapped_watch("multisig_wrapped_watch", 
    "sh(wsh(sortedmulti(3,tpubDEsqS36T4DVsKJd9UH8pAKzrkGBYPLEt9jZMwpKtzh1G6mgYehfHt9WCgk7MJG5QGSFWf176KaBNoXbcuFcuadAFKxDpUdMDKGBha7bY3QM/0/*,tpubDF3cpwfs7fMvXXuoQbohXtLjNM6ehwYT287LWtmLsd4r77YLg6MZg4vTETx5MSJ2zkfigbYWu31VA2Z2Vc1cZugCYXgS7FQu6pE8V6TriEH/0/*,tpubDE1SKfcW76Tb2AASv5bQWMuScYNAdoqLHoexw13sNDXwmUhQDBbCD3QAedKGLhxMrWQdMDKENzYtnXPDRvexQPNuDrLj52wAjHhNEm8sJ4p/0/*,tpubDFLc6oXwJmhm3FGGzXkfJNTh2KitoY3WhmmQvuAjMhD8YbyWn5mAqckbxXfm2etM3p5J6JoTpSrMqRSTfMLtNW46poDaEZJ1kjd3csRSjwH/0/*,tpubDEWD9NBeWP59xXmdqSNt4VYdtTGwbpyP8WS962BuqpQeMZmX9Pur14dhXdZT5a7wR1pK6dPtZ9fP5WR493hPzemnBvkfLLYxnUjAKj1JCQV/0/*,tpubDEHyZkkwd7gZWCTgQuYQ9C4myF2hMEmyHsBCCmLssGqoqUxeT3gzohF5uEVURkf9TtmeepJgkSUmteac38FwZqirjApzNX59XSHLcwaTZCH/0/*,tpubDEqLouCekwnMUWN486kxGzD44qVgeyuqHyxUypNEiQt5RnUZNJe386TKPK99fqRV1vRkZjYAjtXGTECz98MCsdLcnkM67U6KdYRzVubeCgZ/0/*)))",
    "sh(wsh(sortedmulti(3,tpubDEsqS36T4DVsKJd9UH8pAKzrkGBYPLEt9jZMwpKtzh1G6mgYehfHt9WCgk7MJG5QGSFWf176KaBNoXbcuFcuadAFKxDpUdMDKGBha7bY3QM/1/*,tpubDF3cpwfs7fMvXXuoQbohXtLjNM6ehwYT287LWtmLsd4r77YLg6MZg4vTETx5MSJ2zkfigbYWu31VA2Z2Vc1cZugCYXgS7FQu6pE8V6TriEH/1/*,tpubDE1SKfcW76Tb2AASv5bQWMuScYNAdoqLHoexw13sNDXwmUhQDBbCD3QAedKGLhxMrWQdMDKENzYtnXPDRvexQPNuDrLj52wAjHhNEm8sJ4p/1/*,tpubDFLc6oXwJmhm3FGGzXkfJNTh2KitoY3WhmmQvuAjMhD8YbyWn5mAqckbxXfm2etM3p5J6JoTpSrMqRSTfMLtNW46poDaEZJ1kjd3csRSjwH/1/*,tpubDEWD9NBeWP59xXmdqSNt4VYdtTGwbpyP8WS962BuqpQeMZmX9Pur14dhXdZT5a7wR1pK6dPtZ9fP5WR493hPzemnBvkfLLYxnUjAKj1JCQV/1/*,tpubDEHyZkkwd7gZWCTgQuYQ9C4myF2hMEmyHsBCCmLssGqoqUxeT3gzohF5uEVURkf9TtmeepJgkSUmteac38FwZqirjApzNX59XSHLcwaTZCH/1/*,tpubDEqLouCekwnMUWN486kxGzD44qVgeyuqHyxUypNEiQt5RnUZNJe386TKPK99fqRV1vRkZjYAjtXGTECz98MCsdLcnkM67U6KdYRzVubeCgZ/1/*)))")]
fn parse_wallet(
    #[case] wallet_name: &str,
    #[case] expected_descriptor_ext: &str,
    #[case] expected_descriptor_chg: &str,
) {
    let desc = wallet_name_to_descriptors(wallet_name);
    assert_eq!(desc, vec![expected_descriptor_ext, expected_descriptor_chg]);
    let addr = first_address_from_descriptor(&desc[0], Network::Testnet);
    let exp = first_address_from_wallet_file(wallet_name);
    assert_eq!(addr, exp);
}

#[cfg(feature = "wallet_file")]
fn wallet_name_to_descriptors(wallet_name: &str) -> Vec<String> {
    let wallet_file = get_test_wallet_file(wallet_name);
    let wallet = ElectrumWalletFile::from_file(wallet_file.as_path()).unwrap();
    wallet.to_descriptors().unwrap()
}

#[cfg(feature = "wallet_file")]
fn get_test_wallet_file(wallet_name: &str) -> PathBuf {
    let test_dir = Path::new(file!()).canonicalize().unwrap();
    let test_dir = test_dir.as_path().parent().unwrap();
    let wallet_file = test_dir.join("wallets/".to_string() + wallet_name);
    assert!(
        wallet_file.exists(),
        "File not found: {}",
        wallet_file.to_str().unwrap()
    );
    wallet_file.to_path_buf()
}

#[cfg(feature = "wallet_file")]
fn first_address_from_descriptor(desc: &str, network: Network) -> String {
    let wallet = Wallet::new_offline(desc, None, network, MemoryDatabase::default()).unwrap();
    wallet.get_address(AddressIndex::New).unwrap().to_string()
}

#[cfg(feature = "wallet_file")]
fn first_address_from_wallet_file(wallet_name: &str) -> String {
    let wallet_file = get_test_wallet_file(wallet_name);
    let wallet = ElectrumWalletFile::from_file(wallet_file.as_path()).unwrap();
    wallet.addresses.receiving[0].clone()
}

/// Since converting a wallet with imported keys or addresses can't be converted to a descriptor anyway, we just leave a not so descriptive error message due to a different json format of such wallets.
#[cfg(feature = "wallet_file")]
#[rstest]
#[case::imported_addr("imported_addr")]
#[case::imported_privkey("imported_privkey")]
#[should_panic(expected = "missing field `change`")]
fn parse_imported(#[case] wallet_name: &str) {
    let wallet_file = get_test_wallet_file(wallet_name);
    let _wallet = ElectrumWalletFile::from_file(wallet_file.as_path()).unwrap();
}

#[cfg(feature = "wallet_file")]
#[rstest]
#[case::default_legacy("default_legacy", 
    "pkh(tprv8ZgxMBicQKsPeYnCHtn5QZqhTgkkDmXebfQMXWmX7ThXJFCbzDTKFNRsB43GUmHzu2pdGcnnegFy175kFcgZQYC5BFPnRdYDPQyqetpyjb5/0/*)")]
#[case::default_legacy_watch("default_legacy_watch", 
    "pkh(tpubD6NzVbkrYhZ4Y1ozBYSfoyVp2iGgP6iZAy18p2opXjVv8jTNccGuRs3jMCMe4ncfwy2RUJsoZLSXsGiFhN47xFbJgtRvCuV3RP3UnxpsrZt/0/*)")]
#[case::default_segwit("default_segwit", 
    "wpkh(tprv8cvkZzx9zA7EfFDbH945mK23r7hg6EHXUk79wVUSRukwyctFS1AdpSpkZcykAMDveCj8RA3R4jwFTKMwMbWexJox8NMqq7YphJLDumfCSfu/0/*)")]
#[case::multisig_hw_segwit("multisig_hw_segwit", 
    "wsh(sortedmulti(2,tpubDEcw4ooTbmw62zBKdkYepoP3z4WWugdeRzPHHAbk8XVsPfBE9AAZMNghiqwtdFgtabaeppBTPmezUkRkQZidLcSJp3XTASbMakHcYauWehZ/0/*,tpubDEbkvhmJoZMq3SUNqEf3aEsubvqsCUPc7rroHkGERgS7qA1gQVMxUPrgzth6x43odirLohwf4aMHpvcnWi3jCB2xkizv8T4B2KqLRZVLC6K/0/*))")]
#[case::multisig_legacy("multisig_legacy", 
    "sh(sortedmulti(2,tprv8ZgxMBicQKsPeLPWr5WbJDAhANr6irc1Yf7eUNCYjGYap27HU4bDBXWGMT3X75FhDyxNXr6pK4QeHcCBvkqchQzK8wZ4JbGv5X5MWtXQtqy/0/*,tpubD6NzVbkrYhZ4Y1ozBYSfoyVp2iGgP6iZAy18p2opXjVv8jTNccGuRs3jMCMe4ncfwy2RUJsoZLSXsGiFhN47xFbJgtRvCuV3RP3UnxpsrZt/0/*))")]
#[case::multisig_segwit("multisig_segwit", 
    "wsh(sortedmulti(2,tprv8dNybiDsdyms39SAWTxyiNHABTTgiqmJpScmxGrdKEuZ7TwXcaYXT4f4ddVjWiiQs9zowHqyDmvaebN6fU2Lu6iAYnYuepiLkvzGdcZZi8D/0/*,tpubD9cniQzQ8XnuagyP9Xwg3sWCX77wQPWoLPW7jqzcPn37r8hq2X86uztCEyFbMY16amzwdJ1CcNRXhF3vykn1wuDv2ULzryRtaCcN5Cr8F9y/0/*))")]
#[case::multisig_wrapped_watch("multisig_wrapped_watch", 
    "sh(wsh(sortedmulti(3,tpubDEsqS36T4DVsKJd9UH8pAKzrkGBYPLEt9jZMwpKtzh1G6mgYehfHt9WCgk7MJG5QGSFWf176KaBNoXbcuFcuadAFKxDpUdMDKGBha7bY3QM/0/*,tpubDF3cpwfs7fMvXXuoQbohXtLjNM6ehwYT287LWtmLsd4r77YLg6MZg4vTETx5MSJ2zkfigbYWu31VA2Z2Vc1cZugCYXgS7FQu6pE8V6TriEH/0/*,tpubDE1SKfcW76Tb2AASv5bQWMuScYNAdoqLHoexw13sNDXwmUhQDBbCD3QAedKGLhxMrWQdMDKENzYtnXPDRvexQPNuDrLj52wAjHhNEm8sJ4p/0/*,tpubDFLc6oXwJmhm3FGGzXkfJNTh2KitoY3WhmmQvuAjMhD8YbyWn5mAqckbxXfm2etM3p5J6JoTpSrMqRSTfMLtNW46poDaEZJ1kjd3csRSjwH/0/*,tpubDEWD9NBeWP59xXmdqSNt4VYdtTGwbpyP8WS962BuqpQeMZmX9Pur14dhXdZT5a7wR1pK6dPtZ9fP5WR493hPzemnBvkfLLYxnUjAKj1JCQV/0/*,tpubDEHyZkkwd7gZWCTgQuYQ9C4myF2hMEmyHsBCCmLssGqoqUxeT3gzohF5uEVURkf9TtmeepJgkSUmteac38FwZqirjApzNX59XSHLcwaTZCH/0/*,tpubDEqLouCekwnMUWN486kxGzD44qVgeyuqHyxUypNEiQt5RnUZNJe386TKPK99fqRV1vRkZjYAjtXGTECz98MCsdLcnkM67U6KdYRzVubeCgZ/0/*)))")]
fn descriptor_electrum_wallet_roundtrip(#[case] wallet_name: &str, #[case] descriptor: &str) {
    let wallet = ElectrumWalletFile::from_descriptor(descriptor).unwrap();

    let tempdir = tempdir().unwrap();
    let filename = tempdir.path().join(wallet_name);
    wallet.to_file(&filename).unwrap();
    // Testing that electrum can load the files was done manually.

    let imported = ElectrumWalletFile::from_file(&filename).unwrap();
    assert_eq!(wallet, imported);

    let desc = wallet.to_descriptors().unwrap();
    assert_eq!(desc[0], descriptor);
}
