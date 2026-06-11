"""CID: self-describing content identifiers. https://github.com/multiformats/cid"""

from ._multiformats import cid as _cid

CID = _cid.CID

__all__ = ["CID"]
