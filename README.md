# py-multiformats

[![CI](https://github.com/probe-lab/py-multiformats/actions/workflows/ci.yml/badge.svg)](https://github.com/probe-lab/py-multiformats/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/py-multiformats)](https://pypi.org/project/py-multiformats/)
[![Python versions](https://img.shields.io/pypi/pyversions/py-multiformats)](https://pypi.org/project/py-multiformats/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Built with PyO3](https://img.shields.io/badge/built%20with-PyO3-f74c00?logo=rust)](https://pyo3.rs)

Python bindings for the official Rust [multiformats](https://multiformats.io) implementations,
wrapped with [PyO3](https://pyo3.rs):

- [rust-multibase](https://github.com/multiformats/rust-multibase) — self-describing base encodings
- [rust-multihash](https://github.com/multiformats/rust-multihash) — self-describing hashes
- [rust-multiaddr](https://github.com/multiformats/rust-multiaddr) — self-describing network addresses
- [rust-cid](https://github.com/multiformats/rust-cid) — self-describing content identifiers

It also embeds the canonical [multicodec](https://github.com/multiformats/multicodec)
and [multibase](https://github.com/multiformats/multibase) registries, generated
at build time from the vendored tables in `data/` (refreshed weekly from
upstream by a scheduled workflow).

## Install

```bash
pip install py-multiformats
```

## Usage

```python
from multiformats import CID, Multiaddr, Multihash, multibase, multicodec, multihash

# multicodec — the codec registry
multicodec.code("dag-pb")                            # 112 (0x70)
multicodec.name(0x70)                                # "dag-pb"
multicodec.tag("dag-pb")                             # "ipld" (accepts name or code)
multicodec.DAG_PB                                    # 112 — every entry as a constant
multicodec.entries()                                 # [(name, tag, code, status), ...]

# multibase
encoded = multibase.encode("base58btc", b"hello")    # "zCn8eVZg"
encoded = multibase.encode(multibase.BASE58BTC, b"hello")  # same, via constant
base, data = multibase.decode(encoded)               # ("base58btc", b"hello")
multibase.bases()                                    # all supported encodings

# multihash
mh = multihash.sha2_256(b"hello world")
mh = multihash.digest("sha2-256", b"hello world")    # by name ...
mh = multihash.digest(0x12, b"hello world")          # ... or by code
mh.code                                              # 18 (0x12)
mh.name                                              # "sha2-256"
mh.size                                              # 32
mh.digest                                            # raw digest bytes
Multihash.from_bytes(mh.to_bytes()) == mh            # True
multihash.codes()                                    # name -> code table

# CID
cid = CID.decode("QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n")
cid.version                                          # 0
cid.codec                                            # 112 (0x70)
cid.codec_name                                       # "dag-pb"
cid.hash.name                                        # "sha2-256"
CID(1, "raw", cid.hash)                              # codec by multicodec name or code
str(cid.to_v1())                                     # "bafybeihdwdce..."
cid.to_v1().encode("base64url")                      # any multibase encoding
CID.from_bytes(cid.to_bytes()) == cid                # True

# multiaddr
addr = Multiaddr("/ip4/127.0.0.1/tcp/4001")
list(addr)                                           # [("ip4", "127.0.0.1"), ("tcp", "4001")]
addr.protocols()                                     # ["ip4", "tcp"]
addr = addr.encapsulate("/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
addr.decapsulate("/tcp/4001")                        # Multiaddr("/ip4/127.0.0.1")
Multiaddr.from_bytes(addr.to_bytes()) == addr        # True
```

All parse/decode failures raise `multiformats.MultiformatsError`, a subclass of `ValueError`.

## Why this package?

There are existing Python implementations — the legacy single-format packages
([py-multibase](https://github.com/multiformats/py-multibase),
[py-multihash](https://github.com/multiformats/py-multihash),
[py-multiaddr](https://github.com/multiformats/py-multiaddr),
[py-cid](https://github.com/ipld/py-cid)) and the pure-Python
[multiformats](https://github.com/hashberg-io/multiformats) package. This one
takes a different approach:

- **One package, one API.** All four formats plus the multicodec/multibase
  registries live behind a single import with a consistent, typed API and a
  shared exception type. The legacy packages are split across four
  repositories with diverging conventions, and most have been dormant for
  years.
- **The reference implementations do the work.** Parsing, validation, and
  hashing are the official Rust crates maintained by the multiformats
  organization — the same code battle-tested inside rust-libp2p and the IPFS
  ecosystem. Spec conformance fixes land upstream and arrive here by bumping
  a dependency, instead of being re-implemented by hand.
- **Native speed.** Hashing and parsing run as compiled Rust. Pure-Python
  multihash/CID handling is orders of magnitude slower, which matters when
  processing CIDs or multiaddrs in bulk.
- **Registry-faithful by construction.** The multicodec and multibase tables
  are generated from the canonical registry CSVs and refreshed automatically,
  rather than copied into source once and left to rot.
- **Zero-friction install.** abi3 wheels for Linux/macOS/Windows work on any
  CPython ≥ 3.10 — no Rust toolchain, no compilation, no runtime
  dependencies.

## Development

Requires a Rust toolchain and [uv](https://docs.astral.sh/uv/):

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

The multicodec and multibase registries are not hand-written. Their canonical
tables are vendored in `data/` and compiled into the extension at build time:

- `data/multicodec-table.csv` — verbatim copy of
  [multiformats/multicodec `table.csv`](https://github.com/multiformats/multicodec/blob/master/table.csv)
- `data/multibase-table.csv` — verbatim copy of
  [multiformats/multibase `multibase.csv`](https://github.com/multiformats/multibase/blob/master/multibase.csv)

`build.rs` parses both CSVs and writes `multicodec_gen.rs` and
`multibase_gen.rs` into cargo's `OUT_DIR` (they are generated artifacts, not
checked in). Each contains the registry rows as a static `ENTRIES` table,
compile-time perfect hash maps for the lookups ([phf](https://docs.rs/phf)),
and a `consts` module with one constant per entry. The same entries are
registered as the Python constants (`multicodec.DAG_PB`,
`multibase.BASE58BTC`, ...) at import time.

To refresh the tables from upstream, run:

```bash
./scripts/update-tables.sh
```

A scheduled workflow ([update-tables.yml](.github/workflows/update-tables.yml))
runs the same script weekly and opens a pull request when a registry changed;
the regular CI validates the regenerated code.

## License

MIT OR Apache-2.0
