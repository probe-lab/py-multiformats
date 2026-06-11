import pytest

from multiformats import CID, MultiformatsError, multicodec, multihash

# The "hello world" example file on IPFS (dag-pb, sha2-256).
CID_V0 = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
CID_V1 = "bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"


def test_decodes_cid_v0():
    cid = CID.decode(CID_V0)
    assert cid.version == 0
    assert cid.codec == multicodec.DAG_PB
    assert cid.hash.name == "sha2-256"
    assert str(cid) == CID_V0


def test_decodes_cid_v1():
    cid = CID.decode(CID_V1)
    assert cid.version == 1
    assert cid.codec == multicodec.DAG_PB
    assert str(cid) == CID_V1


def test_v0_to_v1_conversion():
    v0 = CID.decode(CID_V0)
    v1 = v0.to_v1()
    assert v1.version == 1
    assert str(v1) == CID_V1
    assert v1.hash == v0.hash
    assert v1.to_v1() == v1  # already v1: identity


def test_constructs_v1_from_parts():
    mh = multihash.sha2_256(b"hello world")
    cid = CID(1, multicodec.RAW, mh)
    assert cid.version == 1
    assert cid.codec == multicodec.RAW
    assert cid.hash == mh
    assert CID.decode(str(cid)) == cid


def test_constructs_v0_from_parts():
    mh = multihash.sha2_256(b"hello world")
    cid = CID(0, multicodec.DAG_PB, mh)
    assert cid.version == 0
    assert str(cid).startswith("Qm")


def test_constructs_with_codec_name():
    mh = multihash.sha2_256(b"hello world")
    assert CID(1, "raw", mh) == CID(1, multicodec.RAW, mh)
    assert CID(1, "dag-pb", mh) == CID(1, multicodec.DAG_PB, mh)


def test_codec_name_resolves_via_py_multicodec():
    assert CID.decode(CID_V0).codec_name == "dag-pb"
    mh = multihash.sha2_256(b"hello world")
    assert CID(1, multicodec.RAW, mh).codec_name == "raw"


def test_codec_name_is_none_for_unregistered_code():
    mh = multihash.sha2_256(b"hello world")
    # 0x300001 is in the multicodec private use area, never registered
    assert CID(1, 0x300001, mh).codec_name is None


def test_rejects_unknown_codec_name():
    mh = multihash.sha2_256(b"hello world")
    with pytest.raises(MultiformatsError, match="unknown multicodec name"):
        CID(1, "not-a-codec", mh)


def test_rejects_codec_of_wrong_type():
    mh = multihash.sha2_256(b"hello world")
    with pytest.raises(MultiformatsError, match="codec must be"):
        CID(1, 1.5, mh)


def test_v0_rejects_non_dag_pb():
    mh = multihash.sha2_256(b"hello world")
    with pytest.raises(MultiformatsError, match="invalid CID"):
        CID(0, multicodec.RAW, mh)


def test_rejects_unknown_version():
    mh = multihash.sha2_256(b"hello world")
    with pytest.raises(MultiformatsError, match="invalid CID"):
        CID(7, multicodec.RAW, mh)


def test_bytes_round_trip():
    for s in (CID_V0, CID_V1):
        cid = CID.decode(s)
        assert CID.from_bytes(cid.to_bytes()) == cid


def test_encode_default_matches_str():
    cid = CID.decode(CID_V1)
    assert cid.encode() == str(cid) == cid.encode("base32")


def test_encode_in_other_bases_round_trips():
    cid = CID.decode(CID_V1)
    for base in ("base58btc", "base36", "base64url"):
        assert CID.decode(cid.encode(base)) == cid


def test_v0_only_encodes_as_base58btc():
    cid = CID.decode(CID_V0)
    assert cid.encode("base58btc") == CID_V0
    with pytest.raises(MultiformatsError):
        cid.encode("base32")


def test_decode_rejects_garbage():
    with pytest.raises(MultiformatsError, match="invalid CID"):
        CID.decode("not-a-cid")


def test_from_bytes_rejects_garbage():
    with pytest.raises(MultiformatsError, match="invalid CID"):
        CID.from_bytes(b"\x01\x02\x03")


def test_equality_and_hashing():
    a = CID.decode(CID_V0)
    b = CID.decode(CID_V0)
    c = a.to_v1()
    assert a == b
    assert a != c  # different version => different CID
    assert len({a, b, c}) == 2


def test_repr_contains_string_form():
    assert repr(CID.decode(CID_V0)) == f'CID("{CID_V0}")'
