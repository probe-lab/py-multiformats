import pytest

import multiformats
from multiformats import MultiformatsError
from multiformats.cid import CID


def test_top_level_exports():
    assert set(multiformats.__all__) == {
        "MultiformatsError",
        "cid",
        "multiaddr",
        "multibase",
        "multicodec",
        "multihash",
    }


def test_classes_live_in_their_submodules_only():
    assert multiformats.cid.CID is CID
    assert multiformats.multiaddr.Multiaddr is not None
    assert multiformats.multihash.Multihash is not None
    for name in ("CID", "Multiaddr", "Multihash"):
        with pytest.raises(AttributeError):
            getattr(multiformats, name)


def test_error_is_value_error_subclass():
    assert issubclass(MultiformatsError, ValueError)


def test_formats_interoperate():
    """A CID's multihash, re-encoded through multibase, survives the round trip."""
    mh = multiformats.multihash.sha2_256(b"interop")
    cid = CID(1, multiformats.multicodec.RAW, mh)
    assert str(cid) == "bafkreid3qss2f5en43dcgtoknq53imyuxipbihkob42cmouap5kh3gsodm"
    encoded = cid.encode(multiformats.multibase.BASE32)
    base, raw = multiformats.multibase.decode(encoded)
    assert base == multiformats.multibase.BASE32
    assert CID.from_bytes(raw) == cid
