# ink! project template
[![Built with ink!](https://raw.githubusercontent.com/paritytech/ink/master/.images/badge.svg)](https://github.com/paritytech/ink) [![continuous-intergration/ink-project-template](https://github.com/paritytech/ink-project-template/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/paritytech/ink-project-template/actions/workflows/ci.yml)

## Local development setup

### Rust and Cargo
Follow the [instruction](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install Rust and Cargo.

#### Cargo clippy linter
Follow the [instruction](https://github.com/rust-lang/rust-clippy#usage) to install `clippy`.

### ink! smart contract tools
Follow the [instruction](https://use.ink/getting-started/setup) to install `ink!` smart contract tools.

### pre-commit
Follow the [instruction](https://pre-commit.com/#installation) to install `pre-commit` tool.

#### Install the git hook script
```sh
pre-commit install
```

## Development
Below you can find some base commands, for more information check [official documentation](https://use.ink/).


### Format code
```sh
cargo +nightly fmt
```

### Run clippy linter
```sh
cargo +nightly clippy --all-features
```

### Check that smart contracts build to WASM
```sh
cargo contract check
```

### Build smart contracts
```sh
cargo contract build --release
```

### Run pre-commit
```sh
pre-commit run --all-files
```

## CI Jobs
This repository contains predefined GitHub actions for quality assurance.

### GitHub runners
- Linux -> `ubuntu_20_64_core`

### Jobs
- Formatting check -> `cargo +nightly fmt --check`
- Linter check -> `cargo +nightly clippy --all-features`
- Building smart contracts -> `cargo contract build`
- Testing smart contracts -> `cargo test --features e2e-tests`
