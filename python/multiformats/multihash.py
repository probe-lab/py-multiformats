"""Multihash: self-describing hashes. https://github.com/multiformats/multihash"""

from ._multiformats import multihash as _multihash

Multihash = _multihash.Multihash
digest = _multihash.digest
codes = _multihash.codes

sha2_256 = _multihash.sha2_256
sha2_512 = _multihash.sha2_512
sha3_224 = _multihash.sha3_224
sha3_256 = _multihash.sha3_256
sha3_384 = _multihash.sha3_384
sha3_512 = _multihash.sha3_512
keccak_224 = _multihash.keccak_224
keccak_256 = _multihash.keccak_256
keccak_384 = _multihash.keccak_384
keccak_512 = _multihash.keccak_512
blake3 = _multihash.blake3
blake2b_256 = _multihash.blake2b_256
blake2b_512 = _multihash.blake2b_512
blake2s_128 = _multihash.blake2s_128
blake2s_256 = _multihash.blake2s_256
ripemd_160 = _multihash.ripemd_160
ripemd_256 = _multihash.ripemd_256
ripemd_320 = _multihash.ripemd_320

__all__ = [
    "Multihash",
    "blake2b_256",
    "blake2b_512",
    "blake2s_128",
    "blake2s_256",
    "blake3",
    "codes",
    "digest",
    "keccak_224",
    "keccak_256",
    "keccak_384",
    "keccak_512",
    "ripemd_160",
    "ripemd_256",
    "ripemd_320",
    "sha2_256",
    "sha2_512",
    "sha3_224",
    "sha3_256",
    "sha3_384",
    "sha3_512",
]
