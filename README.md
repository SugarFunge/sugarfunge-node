![SugarFunge](/docs/sf-name.png)
# Substrate-based SugarFunge Node

The SugarFunge blockchain powers the SugarFunge Protocol. A protocol for companies looking to model their business logic in a privately-owned economy without the complexity of building and maintaining their own blockchain infrastructure. Empowering businesses with digital and physical assets with minting, crafting and trading mechanics.

Read more about [Owned Economies](https://github.com/SugarFunge/OwnedEconomies).


## Local Testnet

> **Important**: to be able to use all [sugarfunge-api](https://github.com/SugarFunge/sugarfunge-api.git) endpoints without problem you must run at least two of these commands and make sure the same version of Polkadot is being used.

<br/>

Alice:
```bash
cargo run --release -- --chain=local --enable-offchain-indexing true --alice --base-path=.tmp/a --port=30334 --rpc-port 9944 --rpc-cors=all --rpc-methods=Unsafe --rpc-external
```

Bob:
``` bash
cargo run --release -- --chain=local --enable-offchain-indexing true --bob --base-path=.tmp/b --port=30335 --rpc-port 9945 --rpc-cors=all --rpc-methods=Unsafe --rpc-external --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/${Alice Local node identity}
```

Charlie:
```bash
cargo run --release -- --chain=local --enable-offchain-indexing true --charlie --base-path=.tmp/c --port=30336 --rpc-port 9946 --rpc-cors=all --rpc-methods=Unsafe --rpc-external --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/${Alice Local node identity}
```

Where `Alice Local node identity` can be obtained from the console logs of the Alice command. Local node identity _ej. **12D3KooWCHDDz4kHRN2dEqn6F4ev5YfPk4o2H5MWwboqZt55myPy**_
