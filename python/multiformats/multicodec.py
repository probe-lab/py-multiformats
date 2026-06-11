"""Multicodec: the canonical compact codec registry. https://github.com/multiformats/multicodec

Generated at build time from the registry's table.csv; every entry is also
exposed as an UPPER_SNAKE_CASE integer constant (e.g. ``DAG_PB == 0x70``).
"""

from ._multiformats import multicodec as _multicodec

code = _multicodec.code
name = _multicodec.name
tag = _multicodec.tag
entries = _multicodec.entries


def __getattr__(constant: str) -> int:
    """Forward constant lookups (DAG_PB, SHA2_256, ...) to the extension module."""
    try:
        return getattr(_multicodec, constant)
    except AttributeError:
        raise AttributeError(f"module {__name__!r} has no attribute {constant!r}") from None


__all__ = ["code", "entries", "name", "tag"]
