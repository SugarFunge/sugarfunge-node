![SugarFunge](/docs/sf-name.png)
# Substrate-based SugarFunge Node

The SugarFunge blockchain powers the SugarFunge Protocol. A protocol for companies looking to model their business logic in a privately-owned economy without the complexity of building and maintaining their own blockchain infrastructure. Empowering businesses with digital and physical assets with minting, crafting and trading mechanics.

Read more about [Owned Economies](https://github.com/SugarFunge/OwnedEconomies).

## Local Testnet

alice:
```
cargo run --release -- --chain=local --enable-offchain-indexing true --alice --base-path=.tmp/a --port=30334 --rpc-cors=all --rpc-methods=Unsafe --rpc-external
```

bob:
```
cargo run --release -- --chain=local --enable-offchain-indexing true --bob --base-path=.tmp/b --port=30335 --rpc-cors=all --rpc-methods=Unsafe --rpc-external --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWNxmYfzomt7EXfMSLuoaK68JzXnZkNjXyAYAwNrQTDx7Y
```

charlie:
```
cargo run --release -- --chain=local --enable-offchain-indexing true --charlie --base-path=.tmp/c --port=30336 --rpc-cors=all --rpc-methods=Unsafe --rpc-external --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWNxmYfzomt7EXfMSLuoaK68JzXnZkNjXyAYAwNrQTDx7Y
```