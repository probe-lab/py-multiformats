# py-multiformats

[![CI](https://github.com/probe-lab/py-multiformats/actions/workflows/ci.yml/badge.svg)](https://github.com/probe-lab/py-multiformats/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/py-multiformats)](https://pypi.org/project/py-multiformats/)
[![Python versions](https://img.shields.io/pypi/pyversions/py-multiformats)](https://pypi.org/project/py-multiformats/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](#license)
[![Built with PyO3](https://img.shields.io/badge/built%20with-PyO3-f74c00?logo=rust)](https://pyo3.rs)

Python bindings for the official Rust [multiformats](https://multiformats.io)
implementations, wrapped with [PyO3](https://pyo3.rs):

- [rust-multibase](https://github.com/multiformats/rust-multibase) — self-describing base encodings
- [rust-multihash](https://github.com/multiformats/rust-multihash) — self-describing hashes
- [rust-multiaddr](https://github.com/multiformats/rust-multiaddr) — self-describing network addresses
- [rust-cid](https://github.com/multiformats/rust-cid) — self-describing content identifiers

The package also embeds the canonical [multicodec](https://github.com/multiformats/multicodec)
and [multibase](https://github.com/multiformats/multibase) registries, compiled
in at build time from the vendored tables in `data/` and refreshed weekly from
upstream.

## Contents

- [Why this package?](#why-this-package)
- [Install](#install)
- [Usage](#usage)
- [Development](#development)
  - [Code generation](#code-generation)
- [License](#license)

## Why this package?

The existing Python options are four single-format packages
([py-multibase](https://github.com/multiformats/py-multibase),
[py-multihash](https://github.com/multiformats/py-multihash),
[py-multiaddr](https://github.com/multiformats/py-multiaddr),
[py-cid](https://github.com/ipld/py-cid)) that have mostly gone quiet, and the
pure-Python [multiformats](https://github.com/hashberg-io/multiformats)
package. This one puts all four formats behind a single typed API and lets the
Rust reference implementations do the actual work (the same code that runs
inside rust-libp2p). Spec fixes arrive by bumping a dependency, parsing and
hashing run at native speed, and the codec tables are generated from the
canonical registries instead of being copied in once and left to rot. Ships as
prebuilt abi3 wheels for CPython ≥ 3.10, no runtime dependencies.

## Install

```bash
pip install py-multiformats
```

## Usage

```python
from multiformats import multibase, multicodec, multihash
from multiformats.cid import CID
from multiformats.multiaddr import Multiaddr
from multiformats.multihash import Multihash

# multicodec — the codec registry. Every entry is a module constant.
multicodec.DAG_PB                                    # 112 (0x70)
multicodec.code("dag-pb")                            # 112, name -> code
multicodec.name(multicodec.DAG_PB)                   # "dag-pb"
multicodec.tag(multicodec.DAG_PB)                    # "ipld" (accepts constant, code, or name)
multicodec.entries()                                 # [(name, tag, code, status), ...]

# multibase — constants hold the canonical encoding names
encoded = multibase.encode(multibase.BASE58BTC, b"hello")   # "zCn8eVZg"
base, data = multibase.decode(encoded)               # ("base58btc", b"hello")
base == multibase.BASE58BTC                          # True
multibase.bases()                                    # all supported encodings

# multihash
mh = multihash.digest(multicodec.SHA2_256, b"hello world")
mh = multihash.sha2_256(b"hello world")              # same, via convenience function
mh.code == multicodec.SHA2_256                       # True
mh.name                                              # "sha2-256"
mh.size                                              # 32
mh.digest                                            # raw digest bytes
Multihash.from_bytes(mh.to_bytes()) == mh            # True
multihash.codes()                                    # name -> code table

# CID
cid = CID.decode("QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n")
cid.version                                          # 0
cid.codec == multicodec.DAG_PB                       # True
cid.codec_name                                       # "dag-pb"
cid.hash.name                                        # "sha2-256"
CID(1, multicodec.RAW, cid.hash)                     # codec by constant, code, or name
str(cid.to_v1())                                     # "bafybeihdwdce..."
cid.to_v1().encode(multibase.BASE64URL)              # any multibase encoding
CID.from_bytes(cid.to_bytes()) == cid                # True

# multiaddr
addr = Multiaddr("/ip4/127.0.0.1/tcp/4001")
list(addr)                                           # [("ip4", "127.0.0.1"), ("tcp", "4001")]
addr.protocols()                                     # ["ip4", "tcp"]
addr = addr.encapsulate("/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
addr.decapsulate("/tcp/4001")                        # Multiaddr("/ip4/127.0.0.1")
Multiaddr.from_bytes(addr.to_bytes()) == addr        # True
```

Anything that fails to parse, decode, or encode raises
`multiformats.MultiformatsError`, a subclass of `ValueError`.

## Development

You need a Rust toolchain and [uv](https://docs.astral.sh/uv/):

```bash
uv sync                                       # create the venv, install the dev tools, build the extension
uv run pytest                                 # run the test suite
uv run mypy tests/                            # type-check against the stubs
uv run maturin develop                        # rebuild the extension after Rust changes
cargo clippy --all-targets -- -D warnings     # lint the Rust side
cargo test                                    # run the Rust unit tests
```

The dev tools (maturin, pytest, mypy) are declared as a
[dependency group](https://docs.astral.sh/uv/concepts/projects/dependencies/#dependency-groups)
in `pyproject.toml`, so `uv sync` installs everything.

### Code generation

The multicodec and multibase registries are not hand-written. The canonical
tables are vendored in `data/`:

- `data/multicodec-table.csv` — verbatim copy of
  [multiformats/multicodec `table.csv`](https://github.com/multiformats/multicodec/blob/master/table.csv)
- `data/multibase-table.csv` — verbatim copy of
  [multiformats/multibase `multibase.csv`](https://github.com/multiformats/multibase/blob/master/multibase.csv)

`build.rs` turns both CSVs into Rust at build time: the registry rows as a
static `ENTRIES` table, [phf](https://docs.rs/phf) perfect hash maps for the
lookups, and a `consts` module with one constant per entry. The same entries
become the Python constants (`multicodec.DAG_PB`, `multibase.BASE58BTC`, ...)
at import time. `build.rs` also writes the `multicodec.pyi` and
`multibase.pyi` stubs — those are committed and CI fails if they drift, so
IDE autocomplete always matches the vendored tables.

To pull the latest tables from upstream:

```bash
./scripts/update-tables.sh
```

A scheduled workflow ([update-tables.yml](.github/workflows/update-tables.yml))
runs the same script weekly and opens a pull request when a registry changed.

### Releasing

Releases are tag-driven; nothing is built or uploaded from a laptop.

1. Bump `version` in `pyproject.toml` (PEP 440, e.g. `0.1.0a1` for an alpha,
   `0.1.0` for a final release) and mirror it in `Cargo.toml`
   (`0.1.0-alpha.1` in semver). Commit and push.
2. Tag the commit and push the tag:

   ```bash
   git tag v0.1.0a1
   git push origin v0.1.0a1
   ```

3. The [release workflow](.github/workflows/release.yml) triggers on `v*`
   tags: it builds abi3 wheels for Linux (manylinux + musllinux,
   x86_64/aarch64), macOS (x86_64/arm64), and Windows (x64), builds the
   sdist, and publishes everything to PyPI.

Publishing uses [trusted publishing](https://docs.pypi.org/trusted-publishers/)
(OIDC) — there are no PyPI tokens anywhere. One-time setup, already done for
this repository: a (pending) publisher on PyPI pointing at
`probe-lab/py-multiformats`, workflow `release.yml`, environment `pypi`, and
a matching `pypi` environment in the GitHub repository settings.

## License

[Apache-2.0](LICENSE)
