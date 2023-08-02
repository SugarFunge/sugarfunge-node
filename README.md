[//]: # (SBP-M1 review: needs updating to Functionland, with link to forked repo)
![SugarFunge](/docs/sf-name.png)
# Substrate-based SugarFunge Node

The SugarFunge blockchain powers the SugarFunge Protocol. A protocol for companies looking to model their business logic in a privately-owned economy without the complexity of building and maintaining their own blockchain infrastructure. Empowering businesses with digital and physical assets with minting, crafting and trading mechanics.

Read more about [Owned Economies](https://github.com/SugarFunge/OwnedEconomies).

## Local Testnet

[//]: # (SBP-M1 review: testnet does not peer using below commands due to addition of pallet-node-authorization to runtime, requiring peer ids to be added to chainspec. Readme needs updating to reflect this.)
alice:
```
cargo run --release -- --chain=local --enable-offchain-indexing true --alice --base-path=.tmp/a --port=30334 --ws-port 9944 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external
```

bob:
```
cargo run --release -- --chain=local --enable-offchain-indexing true --bob --base-path=.tmp/b --port=30335 --ws-port 9945 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWNxmYfzomt7EXfMSLuoaK68JzXnZkNjXyAYAwNrQTDx7Y
```

[//]: # (SBP-M1 review: as per local_testnet_config in chainspec, only alice and bob are required, charlie can be removed)
charlie:
```
cargo run --release -- --chain=local --enable-offchain-indexing true --charlie --base-path=.tmp/c --port=30336 --ws-port 9946 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWNxmYfzomt7EXfMSLuoaK68JzXnZkNjXyAYAwNrQTDx7Y
```

[//]: # (SBP-M1 review: insufficient documentation)
[//]: # (SBP-M1 review: limited tests)
[//]: # (SBP-M1 review: LICENSE file empty)
[//]: # (SBP-M1 review: `docker run functionland/node:release` fails with `/run_node.sh: 11: wait: Illegal option -n`)