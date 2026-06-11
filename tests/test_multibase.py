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
    "base256emoji": "🚀🏃✋🌈😅🌷🤤😻🌟😅👏",
}
SPEC_INPUT = b"yes mani !"


@pytest.mark.parametrize(("base", "expected"), sorted(SPEC_VECTORS.items()))
def test_encodes_spec_vector(base, expected):
    assert multibase.encode(base, SPEC_INPUT) == expected


@pytest.mark.parametrize(("base", "encoded"), sorted(SPEC_VECTORS.items()))
def test_decodes_spec_vector(base, encoded):
    assert multibase.decode(encoded) == (base, SPEC_INPUT)


def test_round_trips_identity():
    encoded = multibase.encode("identity", SPEC_INPUT)
    assert multibase.decode(encoded) == ("identity", SPEC_INPUT)


def test_base256emoji_round_trips_all_byte_values():
    data = bytes(range(256))
    assert multibase.decode(multibase.encode("base256emoji", data)) == ("base256emoji", data)


def test_base256emoji_rejects_foreign_symbol():
    with pytest.raises(MultiformatsError, match="base256emoji alphabet"):
        multibase.decode("🚀🦄")


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
