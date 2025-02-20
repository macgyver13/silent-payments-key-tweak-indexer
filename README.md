## Purpose:

- build a "key tweak indexer" that computes and stores silent payments key tweaks
  - ~~key tweak, txn output public key is computed~~
  - ~~store public key tweak in db~~
  - ~~resume from last block indexed~~
- building a server that can respond with key tweaks for a given blockhash
  - ~~webservice that responds key tweaks given a blockhash~~

### *** Experimental - use at your own risk ***

## Indexer Usage:

```
Usage: tweak-indexer [OPTIONS]

*No Options* -> start at block 709632 and index tweaks until node blockcount

Options:
  --start-height 614860 #will start at indexing from block 614860 for 10 blocks
  --end-height # describes far to index (supersedes --blocks)
  --blocks # # will process n number of blocks before quitting
```

*Note: block 614862 has a tweak

Usage: tweak-service

* http://127.0.0.1:3030/tweaks/0000000000000000000687bca986194dc2c1f949318629b44bb54ec0a94d8244
* http://127.0.0.1:3030/status
* http://127.0.0.1:3030/block_stats

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

