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

```bash
python -m venv venv && venv/bin/pip install maturin pytest mypy
venv/bin/maturin develop        # build the Rust extension into the venv
venv/bin/pytest                 # run the test suite
```

## License

MIT OR Apache-2.0
