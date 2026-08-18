"""Multibase: self-describing base encodings. https://github.com/multiformats/multibase

Every encoding is a member of the ``Multibase`` enum (e.g.
``Multibase.BASE58BTC``), also reachable as an UPPERCASE module attribute.
Members behave as their canonical name string (``Multibase.BASE58BTC ==
"base58btc"``), so they're usable wherever a base name is expected, and
carry the registry's prefix, description, and status alongside it.
"""

from enum import Enum

from ._multiformats import multibase as _multibase

encode = _multibase.encode
decode = _multibase.decode
bases = _multibase.bases
entries = _multibase.entries


class _MultibaseValue(str):
    """str mix-in for `Multibase` members: carries the registry's prefix,
    description, and status alongside the canonical name, which is the
    member's string value."""

    def __new__(cls, name, prefix, description, status):
        obj = super().__new__(cls, name)
        obj._value_ = name
        obj.prefix = prefix
        obj.description = description
        obj.status = status
        return obj


Multibase = Enum(
    "Multibase",
    {
        name.replace("-", "_").upper(): (name, prefix, description, status)
        for name, prefix, description, status in entries()
    },
    type=_MultibaseValue,
    module=__name__,
)
Multibase.__doc__ = (
    "A multibase registry entry, e.g. `Multibase.BASE58BTC`. Behaves as its "
    "canonical name string and carries `.prefix`, `.description`, `.status`."
)


def __getattr__(name: str) -> Multibase:
    """Forward constant lookups (BASE58BTC, ...) to the `Multibase` enum."""
    try:
        return Multibase[name]
    except KeyError:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}") from None


__all__ = ["Multibase", "bases", "decode", "encode", "entries"]
