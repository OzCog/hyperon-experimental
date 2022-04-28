![CI](https://github.com/trueagi-io/hyperon-experimental/actions/workflows/ci.yml/badge.svg)

# Overview

This is a reimplementation from scratch of the C++ Hyperon prototype in the Rust
programming language. This project replaces the [previous
prototype](https://github.com/trueagi-io/hyperon/tree/master).
See [Python examples](./python/tests) to become familiar with Hyperon features.

If you find troubles with the installation, see the `Troubleshooting` section below.

# Prerequisites

Install latest stable Rust, see [Rust installation
page](https://www.rust-lang.org/tools/install).

# Hyperon library

Build and test the library:
```
cd ./lib
cargo build
cargo test
```

To enable logging during tests execute:
```
RUST_LOG=hyperon=debug cargo test
```

# C and Python API

Prerequisites (must be executed in the top directory of the repository):
```
cargo install cbindgen
python -m pip install conan==1.47
python -m pip install -e ./python[dev]
```

Setup build:
```
mkdir -p build
cd build
cmake ..
```

Build and run tests:
```
make
make check
```

To run release build use following instead of `cmake ..`:
```
cmake -DCMAKE_BUILD_TYPE=Release ..
```

# Setup IDE

See [Rust Language Server](https://github.com/rust-lang/rls) page.

In order to use clangd server generate compile commands using cmake var:
```
cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=Y ..
```

# Troubleshooting

## Conan claims it cannot find out the version of the C compiler

If you see the following `cmake` output:
```
ERROR: Not able to automatically detect '/usr/bin/cc' version
ERROR: Unable to find a working compiler
WARN: Remotes registry file missing, creating default one in /root/.conan/remotes.json
ERROR: libcheck/0.15.2: 'settings.compiler' value not defined
```
Try to create the default Conan profile manually:
```
conan profile new --detect default
```
If it doesn't help, then try to manually add `compiler`, `compiler.version` and
`compiler.libcxx` values in the default Conan profile
(`~/.conan/profiles/default`). 
For example:
```
conan profile update settings.compiler=gcc default
conan profile update settings.compiler.version=7 default
conan profile update settings.compiler.libcxx=libstdc++ default
```

## Rust compiler shows errors

Please ensure you are using the latest stable version:
```
rustup update stable
```

## Other issues

Docker image can be used to run reproducible environment. See instructions
inside [Dockerfile](./.github/Dockerfile). If docker image doesn't work please
raise the
[issue](https://github.com/trueagi-io/hyperon-experimental/issues).
