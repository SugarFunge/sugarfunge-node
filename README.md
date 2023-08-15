![SugarFunge](/docs/sf-name.png)
# Substrate-based SugarFunge Node

The SugarFunge blockchain powers the SugarFunge Protocol. A protocol for companies looking to model their business logic in a privately-owned economy without the complexity of building and maintaining their own blockchain infrastructure. Empowering businesses with digital and physical assets with minting, crafting and trading mechanics.

Read more about [Owned Economies](https://github.com/SugarFunge/OwnedEconomies).


## Local Testnet

> **Important**: to be able to use all [sugarfunge-api](https://github.com/functionland/sugarfunge-api) endpoints without problem you must run at least two validators

<br/>

1st Validator:
```bash
cargo run --release -- --chain ./customSpecRaw.json --enable-offchain-indexing true --base-path=.tmp/node01 --port=30334 --rpc-port 9944 --rpc-cors=all --rpc-methods=Unsafe --rpc-external --validator --name "${1st Validator Name}" --node-key=${1st Validator Node key} --password-filename "${path to file}"
```

2nd Validator:
``` bash
cargo run --release -- --chain ./customSpecRaw.json --enable-offchain-indexing true --base-path=.tmp/node02 --port=30335 --rpc-port 9945 --rpc-cors=all --rpc-methods=Unsafe --rpc-external --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/${1st Validator Local node identity} --validator --name "${2nd Validator Name}" --node-key=${2nd Validator Node key} --password-filename "${path to file}"
```
