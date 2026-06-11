import pytest

from multiformats import MultiformatsError, multicodec


def test_code_lookup():
    assert multicodec.code("dag-pb") == 0x70
    assert multicodec.code("raw") == 0x55
    assert multicodec.code("sha2-256") == 0x12


def test_name_lookup():
    assert multicodec.name(0x70) == "dag-pb"
    assert multicodec.name(0x55) == "raw"
    assert multicodec.name(0x12) == "sha2-256"


def test_lookups_are_inverse():
    for name, _, code, _ in multicodec.entries():
        assert multicodec.code(name) == code
        assert multicodec.name(code) == name


def test_tag_accepts_name_or_code():
    assert multicodec.tag("dag-pb") == "ipld"
    assert multicodec.tag(0x70) == "ipld"
    assert multicodec.tag("sha2-256") == "multihash"
    assert multicodec.tag("tcp") == "multiaddr"


def test_entries_shape():
    entries = multicodec.entries()
    assert len(entries) > 600
    assert ("identity", "multihash", 0x00, "permanent") in entries
    assert all(isinstance(name, str) and isinstance(code, int) for name, _, code, _ in entries)


def test_constants():
    assert multicodec.DAG_PB == 0x70
    assert multicodec.RAW == 0x55
    assert multicodec.SHA2_256 == 0x12
    assert multicodec.LIBP2P_KEY == 0x72
    # name sanitization: "bls12_381-g1-pub" -> BLS12_381_G1_PUB
    assert multicodec.BLS12_381_G1_PUB == multicodec.code("bls12_381-g1-pub")


def test_unknown_name_raises():
    with pytest.raises(MultiformatsError, match="unknown multicodec name"):
        multicodec.code("not-a-codec")
    with pytest.raises(AttributeError):
        multicodec.NOT_A_CODEC


def test_unknown_code_raises():
    with pytest.raises(MultiformatsError, match="unknown multicodec code"):
        multicodec.name(0x300001)  # private use area, never registered


def test_tag_rejects_wrong_type():
    with pytest.raises(MultiformatsError, match="codec must be"):
        multicodec.tag(1.5)
