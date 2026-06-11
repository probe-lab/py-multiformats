import multiformats
from multiformats import CID, Multiaddr, Multihash, MultiformatsError


def test_top_level_exports():
    assert set(multiformats.__all__) == {
        "CID",
        "Multiaddr",
        "Multihash",
        "MultiformatsError",
        "cid",
        "multiaddr",
        "multibase",
        "multicodec",
        "multihash",
    }


def test_error_is_value_error_subclass():
    assert issubclass(MultiformatsError, ValueError)


def test_classes_are_importable_from_submodules():
    assert CID is multiformats.cid.CID
    assert Multiaddr is multiformats.multiaddr.Multiaddr
    assert Multihash is multiformats.multihash.Multihash


def test_formats_interoperate():
    """A CID's multihash, re-encoded through multibase, survives the round trip."""
    mh = multiformats.multihash.sha2_256(b"interop")
    cid = CID(1, multiformats.multicodec.RAW, mh)
    assert cid.__str__() == "bafkreid3qss2f5en43dcgtoknq53imyuxipbihkob42cmouap5kh3gsodm"
    encoded = cid.encode(multiformats.multibase.BASE32)
    base, raw = multiformats.multibase.decode(encoded)
    assert base == multiformats.multibase.BASE32
    assert CID.from_bytes(raw) == cid
