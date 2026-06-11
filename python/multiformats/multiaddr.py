"""Multiaddr: self-describing network addresses. https://github.com/multiformats/multiaddr"""

from ._multiformats import multiaddr as _multiaddr

Multiaddr = _multiaddr.Multiaddr

__all__ = ["Multiaddr"]
