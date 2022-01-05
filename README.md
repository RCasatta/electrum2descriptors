# electrum2descriptors

[![crates.io](https://img.shields.io/crates/v/electrum2descriptors.svg)](https://crates.io/crates/electrum2descriptors)

Converts [slip-0132](https://github.com/satoshilabs/slips/blob/master/slip-0132.md) extended keys (like the vpub, ypub, yprv, etc. used by Electrum) into [output descriptors](https://github.com/bitcoin/bitcoin/blob/master/doc/descriptors.md)

## Usage

```
$ cargo install electrum2descriptors
$ electrum2descriptors vpub5VXaSncXqxLbdmvrC4Y8z9CszPwuEscADoetWhfrxDFzPUbL5nbVtanYDkrVEutkv9n5A5aCcvRC9swbjDKgHjCZ2tAeae8VsBuPbS8KpXv
["wpkh(tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp/0/*)", "wpkh(tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp/1/*)"]
```

or

```
git clone https://github.com/RCasatta/electrum2descriptors
cd electrum2descriptors
cargo run -- vpub5VXaSncXqxLbdmvrC4Y8z9CszPwuEscADoetWhfrxDFzPUbL5nbVtanYDkrVEutkv9n5A5aCcvRC9swbjDKgHjCZ2tAeae8VsBuPbS8KpXv
["wpkh(tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp/0/*)", "wpkh(tpubD9ZjaMn3rbP1cAVwJy6UcEjFfTLT7W6DbfHdS3Wn48meExtVfKmiH9meWCrSmE9qXLYbGcHC5LxLcdfLZTzwme23qAJoRzRhzbd68dHeyjp/1/*)"]
```

can also convert electrum wallet files to descriptors

```
$ cargo run -- tests/wallets/default_segwit 
["wpkh(tprv8cvkZzx9zA7EfFDbH945mK23r7hg6EHXUk79wVUSRukwyctFS1AdpSpkZcykAMDveCj8RA3R4jwFTKMwMbWexJox8NMqq7YphJLDumfCSfu/0/*)", "wpkh(tprv8cvkZzx9zA7EfFDbH945mK23r7hg6EHXUk79wVUSRukwyctFS1AdpSpkZcykAMDveCj8RA3R4jwFTKMwMbWexJox8NMqq7YphJLDumfCSfu/1/*)"]
```
