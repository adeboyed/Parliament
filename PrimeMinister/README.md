# PrimeMinister

> **Prime Minister, is the cluster master, taking workload requests from users and assigning them to workers. **

Prime Minister can either be run in single or multi-master mode.
Multiple masters requires use of the additional consensus module to perform negotiation of the masters.

### Directories

/minister - Cluster master code
/consensus - Code for consensus module
/shared - Code shared between minister and consensus


### Build

1. Install Rust
2. Build using cargo

```bash
❯ cargo build
```

### Test

```bash
❯ cargo test
```

### Run

1. For single master mode
```bash
❯ cargo run -p minister
```

1. For multi-master mode

Run on every master machine
IP address and ports must be statically defined, see main.rs for more details
```bash
❯ cargo run -p minister
❯ cargo run -p consensus
```




