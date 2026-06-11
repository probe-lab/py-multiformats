"""Multibase: self-describing base encodings. https://github.com/multiformats/multibase

Every encoding is also exposed as an UPPERCASE constant holding its
canonical name (e.g. ``BASE58BTC == "base58btc"``), usable wherever a base
name is expected.
"""

from ._multiformats import multibase as _multibase

encode = _multibase.encode
decode = _multibase.decode
bases = _multibase.bases
entries = _multibase.entries


def __getattr__(constant: str) -> str:
    """Forward constant lookups (BASE58BTC, ...) to the extension module."""
    try:
        return getattr(_multibase, constant)
    except AttributeError:
        raise AttributeError(f"module {__name__!r} has no attribute {constant!r}") from None


__all__ = ["bases", "decode", "encode", "entries"]
