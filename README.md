# py-multiformats

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

## Development

Requires a Rust toolchain and [uv](https://docs.astral.sh/uv/):

```bash
uv venv                                       # create the virtual environment
uv pip install maturin pytest mypy            # install the dev tools
uv run maturin develop                        # build the Rust extension into the venv
uv run pytest                                 # run the test suite
uv run mypy tests/                            # type-check against the stubs
cargo clippy --all-targets -- -D warnings     # lint the Rust side
cargo test                                    # run the Rust unit tests
```

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
