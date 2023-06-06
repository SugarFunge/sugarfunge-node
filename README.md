![SugarFunge](/docs/sf-name.png)
# Substrate-based SugarFunge Node

The SugarFunge blockchain powers the SugarFunge Protocol. A protocol for companies looking to model their business logic in a privately-owned economy without the complexity of building and maintaining their own blockchain infrastructure. Empowering businesses with digital and physical assets with minting, crafting and trading mechanics.

Read more about [Owned Economies](https://github.com/SugarFunge/OwnedEconomies).

## Local Testnet

Alice:
```
cargo run --release -- --chain=local --enable-offchain-indexing true --alice --base-path=.tmp/a --port=30334 --ws-port 9944 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external
```

Bob:
```
cargo run --release -- --chain=local --enable-offchain-indexing true --bob --base-path=.tmp/b --port=30335 --ws-port 9945 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWNxmYfzomt7EXfMSLuoaK68JzXnZkNjXyAYAwNrQTDx7Y
```

Charlie:
```
cargo run --release -- --chain=local --enable-offchain-indexing true --charlie --base-path=.tmp/c --port=30336 --ws-port 9946 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWNxmYfzomt7EXfMSLuoaK68JzXnZkNjXyAYAwNrQTDx7Y
```

Where *12D3KooWNxmYfzomt7EXfMSLuoaK68JzXnZkNjXyAYAwNrQTDx7Y* represents the Local node identity shown when running the alice node:

```
2023-11-03 15:32:14 Substrate Node
2023-11-03 15:32:14 ✌️  version 3.0.0-monthly-2021-09+1-bf52814-x86_64-macos
2023-11-03 15:32:14 ❤️  by Substrate DevHub <https://github.com/substrate-developer-hub>, 2017-2021
2023-11-03 15:32:14 📋 Chain specification: My Custom Testnet
2023-11-03 15:32:14 🏷 Node name: MyNode01
2023-11-03 15:32:14 👤 Role: AUTHORITY
2023-11-03 15:32:14 💾 Database: RocksDb at /tmp/node01/chains/local_testnet/db
2023-11-03 15:32:14 ⛓  Native runtime: node-template-100 (node-template-1.tx1.au1)
2023-11-03 15:32:15 🔨 Initializing Genesis block/state (state: 0x2bde…8f66, header-hash: 0x6c78…37de)
2023-11-03 15:32:15 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.
2023-11-03 15:32:15 ⏱  Loaded block-time = 6s from block 0x6c78abc724f83285d1487ddcb1f948a2773cb38219c4674f84c727833be737de
2023-11-03 15:32:15 Using default protocol ID "sup" because none is configured in the chain specs
2023-11-03 15:32:15 🏷 Local node identity is: 12D3KooWNxmYfzomt7EXfMSLuoaK68JzXnZkNjXyAYAwNrQTDx7Y
2023-11-03 15:32:15 📦 Highest known block at #0
2023-11-03 15:32:15 〽️ Prometheus exporter started at 127.0.0.1:9615
2023-11-03 15:32:15 Listening for new connections on 127.0.0.1:9945.
2023-11-03 15:32:20 💤 Idle (0 peers), best: #0 (0x6c78…37de), finalized #0 (0x6c78…37de), ⬇ 0 ⬆ 0
```
