## Purpose:

- build a "key tweak indexer" that computes and stores silent payments key tweaks
  - user provides a sender public key and receiver sp address
  - ~~key tweak, txn output public key is computed~~
  - store sender public key, txn output public key, tweak in db
- building a server that can respond with key tweaks for a given blockhash
  - webservice that responds key tweaks given a blockhash

### *** Experimental - use at your own risk ***

## Resources:

* [BIP352](https://github.com/bitcoin/bips/blob/master/bip-0352.mediawiki)
* [BIP352 Tracker](https://github.com/bitcoin/bitcoin/issues/28536)
* [Main Website](https://silentpayments.xyz/)
* [How Silent Payments Work](https://bitcoin.design/guide/how-it-works/silent-payments/)
* [Block Filters](https://en.bitcoin.it/wiki/BIP_0157)
* [Developer Podcast](https://podcasts.apple.com/us/podcast/silent-payments-a-bitcoin-username-with-josibake/id1415720320?i=1000656901291)
* https://medium.com/@ottosch/how-silent-payments-work-41bea907d6b0
* https://delvingbitcoin.org/t/silent-payments-light-client-protocol/891/1

## Implementations:

Python:

* https://github.com/bitcoin/bips/blob/master/bip-0352/reference.py

Rust:

* https://github.com/cygnet3/rust-silentpayments
* https://github.com/cygnet3/sp-client

Wallets:

* https://silentpayments.xyz/docs/wallets/

