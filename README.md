# Frameless UTXO Blockchain 

### Mentions
Inspired by @JoshOrndorff & @nczhu

## Description
This is a UTXO based model using a frameless substrate node template. The point of this is to demonstrate
the ability to craft a UTXO based runtime purely with substrate without the use of frame. Any contributions welcome!

## Getting Started

### Build
```sh
cargo build --release
```
## Demo

### Start node
```sh
./target/release/utxo-node --dev --tmp
```

### Alice key information
Alice_Pub_Key:
0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67
Alice_Priv_Phrase:
"news slush supreme milk chapter athlete soap sausage put clutch what kitten"


### Structure of Transaction
```rust
Transaction {
    inputs: vec![TransactionInput {
        outpoint: GENESIS_UTXO // (79eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999)
        signature: 0 // (Dont sign the message with the signature already attached)
    }],
    outputs: vec![TransactionOutput {
        value: 25,
        pubkey: 0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67 
    }]
}
```

### Signed Transaction
```sh
0x0479eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999e0a8cbba4b64f34108b46bab7e4dfbffb85c0e7384ec995acbcaa3405772753337713e235c48d1a321dd79fad12b1a646024614fe8a326a1cd0aa9261a52638e0419000000000000000000000000000000d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67
```

### Run the following curl command and pass the signed transaction as a parameter
```sh
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"author_submitExtrinsic",
        "params": ["0x0479eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999e0a8cbba4b64f34108b46bab7e4dfbffb85c0e7384ec995acbcaa3405772753337713e235c48d1a321dd79fad12b1a646024614fe8a326a1cd0aa9261a52638e0419000000000000000000000000000000d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"]
}'
```

### New UTXO of Value 25 owned by alice will be created:
```sh
0x3d5f51643fc96a8786597ed6dd7ee97eaf80bc49598b3366f32d26727adcdc5f
```

### You can see that the old GENESIS_UTXO has been spent by running the following curl command:
```sh
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"state_getStorage",
        "params": ["0x79eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999"] 
}'
```

### Check to see that the new UTXO is in the state by running the curl command with the new UTXO:
```sh
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"state_getStorage",      
        "params": ["0x3d5f51643fc96a8786597ed6dd7ee97eaf80bc49598b3366f32d26727adcdc5f"]
}'
```

### This will yield the scale encoded UTXO:
```sh
0x19000000000000000000000000000000d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67
```

### This will give the scale encoded UTXO which can be decoded using the following:
```rust
const THING_TO_DECODE: [u8; 48] = hex!("19000000000000000000000000000000d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67");
println!("THING_TO_DECODE:{:?}", utxo::TransactionOutput::decode(&mut &THING_TO_DECODE[..]));
```
