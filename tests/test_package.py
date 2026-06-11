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
    cid = CID(1, 0x55, mh)
    encoded = multiformats.multibase.encode("base32", cid.to_bytes())
    base, raw = multiformats.multibase.decode(encoded)
    assert base == "base32"
    assert CID.from_bytes(raw) == cid
