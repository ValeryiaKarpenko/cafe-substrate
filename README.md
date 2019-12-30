# cafe

A new SRML-based Substrate node, ready for hacking.

# Building

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install required tools:

```bash
./scripts/init.sh
```

Build the WebAssembly binary:

```bash
./scripts/build.sh
```

Build all native code:

```bash
cargo build
```

# Run

You can start a development chain with:

```bash
./target/release/cafe --dev --ws-external
```

You can delete a development chain with:

```bash
./target/release/cafe purge-chain --dev -y
```
You can test:

```bash
cargo test -p cafe-runtime cafe
```
