import hashlib

import pytest

from multiformats import Multihash, MultiformatsError, multihash

DATA = b"hello world"

# Algorithms that exist in hashlib, for cross-checking digests.
HASHLIB_EQUIVALENTS = [
    (multihash.sha1, 0x11, lambda d: hashlib.sha1(d)),
    (multihash.sha2_256, 0x12, lambda d: hashlib.sha256(d)),
    (multihash.sha2_512, 0x13, lambda d: hashlib.sha512(d)),
    (multihash.sha3_224, 0x17, lambda d: hashlib.sha3_224(d)),
    (multihash.sha3_256, 0x16, lambda d: hashlib.sha3_256(d)),
    (multihash.sha3_384, 0x15, lambda d: hashlib.sha3_384(d)),
    (multihash.sha3_512, 0x14, lambda d: hashlib.sha3_512(d)),
    (multihash.blake2b_256, 0xB220, lambda d: hashlib.blake2b(d, digest_size=32)),
    (multihash.blake2b_512, 0xB240, lambda d: hashlib.blake2b(d, digest_size=64)),
    (multihash.blake2s_128, 0xB250, lambda d: hashlib.blake2s(d, digest_size=16)),
    (multihash.blake2s_256, 0xB260, lambda d: hashlib.blake2s(d, digest_size=32)),
]


@pytest.mark.parametrize(
    ("fn", "code", "reference"),
    HASHLIB_EQUIVALENTS,
    ids=[f.__name__ for f, _, _ in HASHLIB_EQUIVALENTS],
)
def test_digest_matches_hashlib(fn, code, reference):
    mh = fn(DATA)
    assert mh.code == code
    assert mh.digest == reference(DATA).digest()
    assert mh.size == len(mh.digest)


def test_keccak_256_known_vector():
    assert (
        multihash.keccak_256(b"").digest.hex()
        == "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
    )


def test_blake3_known_vector():
    assert (
        multihash.blake3(b"").digest.hex()
        == "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
    )


def test_ripemd_160_known_vector():
    assert (
        multihash.ripemd_160(b"").digest.hex()
        == "9c1185a5c5e9fc54612808977ee8f548b2258d31"
    )


def test_identity_keeps_data():
    mh = multihash.identity(DATA)
    assert mh.code == 0x00
    assert mh.digest == DATA
    assert mh.size == len(DATA)


def test_digest_by_name_and_code_agree():
    by_name = multihash.digest("sha2-256", DATA)
    by_code = multihash.digest(0x12, DATA)
    by_fn = multihash.sha2_256(DATA)
    assert by_name == by_code == by_fn


def test_digest_rejects_unknown_name():
    with pytest.raises(MultiformatsError, match="unknown hash algorithm"):
        multihash.digest("md5", DATA)


def test_digest_rejects_unknown_code():
    with pytest.raises(MultiformatsError, match="unknown hash algorithm code"):
        multihash.digest(0xDEAD, DATA)


def test_bytes_round_trip():
    mh = multihash.sha2_256(DATA)
    raw = mh.to_bytes()
    assert raw[0] == 0x12  # varint code
    assert raw[1] == 32  # varint digest size
    assert raw[2:] == mh.digest
    assert Multihash.from_bytes(raw) == mh


def test_from_bytes_rejects_truncated_input():
    with pytest.raises(MultiformatsError, match="invalid multihash"):
        Multihash.from_bytes(b"\x12\x20abc")


def test_wrap_precomputed_digest():
    digest = hashlib.sha256(DATA).digest()
    assert Multihash.wrap(0x12, digest) == multihash.sha2_256(DATA)


def test_wrap_rejects_oversized_digest():
    with pytest.raises(MultiformatsError, match="invalid multihash"):
        Multihash.wrap(0x00, b"x" * 65)  # the bindings allocate 64 bytes max


def test_name_property():
    assert multihash.sha2_256(DATA).name == "sha2-256"
    assert Multihash.wrap(0x12345, b"\x01").name is None


def test_codes_table():
    codes = multihash.codes()
    assert codes["sha2-256"] == 0x12
    assert codes["blake3"] == 0x1E
    assert len(codes) == 20


def test_codes_agree_with_py_multicodec_table():
    from multicodec.constants import NAME_TABLE

    for name, code in multihash.codes().items():
        assert NAME_TABLE[name] == code, name


def test_equality_and_hashing():
    a = multihash.sha2_256(DATA)
    b = multihash.sha2_256(DATA)
    c = multihash.sha2_512(DATA)
    assert a == b
    assert a != c
    assert len({a, b, c}) == 2


def test_repr_names_known_algorithm():
    assert repr(multihash.sha2_256(DATA)).startswith("Multihash(sha2-256")
