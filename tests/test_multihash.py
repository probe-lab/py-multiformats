import hashlib

import pytest

from multiformats import MultiformatsError, multicodec, multihash
from multiformats.multihash import Multihash

DATA = b"hello world"

# Algorithms that exist in hashlib, for cross-checking digests.
HASHLIB_EQUIVALENTS = [
    ("sha1", multicodec.SHA1, lambda d: hashlib.sha1(d)),
    ("sha2-256", multicodec.SHA2_256, lambda d: hashlib.sha256(d)),
    ("sha2-512", multicodec.SHA2_512, lambda d: hashlib.sha512(d)),
    ("sha3-224", multicodec.SHA3_224, lambda d: hashlib.sha3_224(d)),
    ("sha3-256", multicodec.SHA3_256, lambda d: hashlib.sha3_256(d)),
    ("sha3-384", multicodec.SHA3_384, lambda d: hashlib.sha3_384(d)),
    ("sha3-512", multicodec.SHA3_512, lambda d: hashlib.sha3_512(d)),
    ("blake2b-256", multicodec.BLAKE2B_256, lambda d: hashlib.blake2b(d, digest_size=32)),
    ("blake2b-512", multicodec.BLAKE2B_512, lambda d: hashlib.blake2b(d, digest_size=64)),
    ("blake2s-128", multicodec.BLAKE2S_128, lambda d: hashlib.blake2s(d, digest_size=16)),
    ("blake2s-256", multicodec.BLAKE2S_256, lambda d: hashlib.blake2s(d, digest_size=32)),
]


@pytest.mark.parametrize(
    ("name", "code", "reference"),
    HASHLIB_EQUIVALENTS,
    ids=[name for name, _, _ in HASHLIB_EQUIVALENTS],
)
def test_digest_matches_hashlib(name, code, reference):
    mh = multihash.digest(name, DATA)
    assert mh.code == code
    assert mh.digest == reference(DATA).digest()
    assert mh.size == len(mh.digest)


def test_convenience_functions_match_digest():
    """Every supported algorithm except identity and sha1 has a module-level
    function named after it (e.g. sha2-256 -> multihash.sha2_256)."""
    for name in multihash.codes():
        fn = getattr(multihash, name.replace("-", "_"), None)
        if name in ("identity", "sha1"):
            assert fn is None  # deliberately not exposed as functions
        else:
            assert fn(DATA) == multihash.digest(name, DATA)


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
    mh = multihash.digest("identity", DATA)
    assert mh.code == multicodec.IDENTITY
    assert mh.digest == DATA
    assert mh.size == len(DATA)


def test_digest_by_name_and_code_agree():
    by_name = multihash.digest("sha2-256", DATA)
    by_code = multihash.digest(multicodec.SHA2_256, DATA)
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
    assert Multihash.wrap(multicodec.SHA2_256, digest) == multihash.sha2_256(DATA)


def test_wrap_rejects_oversized_digest():
    with pytest.raises(MultiformatsError, match="invalid multihash"):
        Multihash.wrap(multicodec.IDENTITY, b"x" * 65)  # the bindings allocate 64 bytes max


def test_name_property():
    assert multihash.sha2_256(DATA).name == "sha2-256"
    assert Multihash.wrap(0x12345, b"\x01").name is None


def test_codes_table():
    codes = multihash.codes()
    assert codes["sha2-256"] == 0x12
    assert codes["blake3"] == 0x1E
    assert len(codes) == 20


def test_codes_agree_with_multicodec_registry():
    for name, code in multihash.codes().items():
        assert multicodec.code(name) == code, name
        assert multicodec.tag(code) == "multihash", name


def test_equality_and_hashing():
    a = multihash.sha2_256(DATA)
    b = multihash.sha2_256(DATA)
    c = multihash.sha2_512(DATA)
    assert a == b
    assert a != c
    assert len({a, b, c}) == 2


def test_repr_names_known_algorithm():
    assert repr(multihash.sha2_256(DATA)).startswith("Multihash(sha2-256")
