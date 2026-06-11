"""Multibase: self-describing base encodings. https://github.com/multiformats/multibase"""

from ._multiformats import multibase as _multibase

encode = _multibase.encode
decode = _multibase.decode
bases = _multibase.bases

__all__ = ["bases", "decode", "encode"]
