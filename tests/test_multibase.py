import pytest

from multiformats import MultiformatsError, multibase

# Test vectors from the multibase spec (https://github.com/multiformats/multibase),
# input "yes mani !".
SPEC_VECTORS = {
    "base2": "001111001011001010111001100100000011011010110000101101110011010010010000000100001",
    "base8": "7362625631006654133464440102",
    "base10": "9573277761329450583662625",
    "base16": "f796573206d616e692021",
    "base16upper": "F796573206D616E692021",
    "base32": "bpfsxgidnmfxgsibb",
    "base32upper": "BPFSXGIDNMFXGSIBB",
    "base32hex": "vf5in683dc5n6i811",
    "base32hexupper": "VF5IN683DC5N6I811",
    "base32pad": "cpfsxgidnmfxgsibb",
    "base32padupper": "CPFSXGIDNMFXGSIBB",
    "base32hexpad": "tf5in683dc5n6i811",
    "base32hexpadupper": "TF5IN683DC5N6I811",
    "base32z": "hxf1zgedpcfzg1ebb",
    "base36": "k2lcpzo5yikidynfl",
    "base36upper": "K2LCPZO5YIKIDYNFL",
    "base58flickr": "Z7Pznk19XTTzBtx",
    "base58btc": "z7paNL19xttacUY",
    "base64": "meWVzIG1hbmkgIQ",
    "base64pad": "MeWVzIG1hbmkgIQ==",
    "base64url": "ueWVzIG1hbmkgIQ",
    "base64urlpad": "UeWVzIG1hbmkgIQ==",
}
SPEC_INPUT = b"yes mani !"
EMOJI_SPEC_VECTOR = "🚀🏃✋🌈😅🌷🤤😻🌟😅👏"


@pytest.mark.parametrize(("base", "expected"), sorted(SPEC_VECTORS.items()))
def test_encodes_spec_vector(base, expected):
    assert multibase.encode(base, SPEC_INPUT) == expected


@pytest.mark.parametrize(("base", "encoded"), sorted(SPEC_VECTORS.items()))
def test_decodes_spec_vector(base, encoded):
    assert multibase.decode(encoded) == (base, SPEC_INPUT)


def test_constants_hold_canonical_names():
    assert multibase.BASE58BTC == "base58btc"
    assert multibase.BASE32 == "base32"
    assert multibase.IDENTITY == "identity"
    assert {getattr(multibase, name.upper()) for name in multibase.bases()} == set(
        multibase.bases()
    )


def test_encode_accepts_constants():
    assert multibase.encode(multibase.BASE58BTC, SPEC_INPUT) == SPEC_VECTORS["base58btc"]


def test_unknown_constant_raises():
    with pytest.raises(AttributeError):
        multibase.BASE59


def test_round_trips_identity():
    encoded = multibase.encode("identity", SPEC_INPUT)
    assert multibase.decode(encoded) == ("identity", SPEC_INPUT)


def test_encodes_base256emoji_spec_vector():
    assert multibase.encode("base256emoji", SPEC_INPUT) == EMOJI_SPEC_VECTOR


@pytest.mark.xfail(
    reason="rust-multibase base256emoji decoding is broken upstream: the match-lookup "
    "crate builds its alphabet table from char_indices() byte offsets",
    strict=True,
)
def test_decodes_base256emoji_spec_vector():
    assert multibase.decode(EMOJI_SPEC_VECTOR) == ("base256emoji", SPEC_INPUT)


def test_bases_lists_all_supported_encodings():
    names = multibase.bases()
    assert set(SPEC_VECTORS) < set(names)
    assert "identity" in names
    assert "base256emoji" in names
    assert len(names) == 24


def test_encode_rejects_unknown_base():
    with pytest.raises(MultiformatsError, match="unknown multibase encoding"):
        multibase.encode("base59", b"data")


def test_decode_rejects_invalid_string():
    with pytest.raises(MultiformatsError, match="invalid multibase string"):
        multibase.decode("!not-multibase")


def test_decode_rejects_corrupt_payload():
    with pytest.raises(MultiformatsError):
        multibase.decode("z0OIl")  # 0, O, I, l are not in the base58btc alphabet


def test_errors_are_value_errors():
    with pytest.raises(ValueError):
        multibase.decode("!not-multibase")
