"""Python bindings for the Rust multiformats implementations.

Wraps rust-multibase, rust-multihash, rust-multiaddr, and rust-cid in a
single package: https://multiformats.io

The classes live in their format submodules: `multiformats.cid.CID`,
`multiformats.multihash.Multihash`, `multiformats.multiaddr.Multiaddr`.
"""

from . import cid, multiaddr, multibase, multicodec, multihash
from ._multiformats import MultiformatsError

__all__ = [
    "MultiformatsError",
    "cid",
    "multiaddr",
    "multibase",
    "multicodec",
    "multihash",
]
