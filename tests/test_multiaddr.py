import pytest

from multiformats import MultiformatsError
from multiformats.multiaddr import Multiaddr

ADDR = "/ip4/127.0.0.1/tcp/4001"
PEER = "/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"


def test_parses_and_formats():
    addr = Multiaddr(ADDR)
    assert str(addr) == ADDR
    assert repr(addr) == f'Multiaddr("{ADDR}")'


def test_binary_encoding():
    # ip4 (code 0x04) 127.0.0.1, tcp (code 0x06) port 4001 (0x0fa1)
    assert Multiaddr(ADDR).to_bytes().hex() == "047f000001060fa1"


def test_bytes_round_trip():
    for s in (ADDR, ADDR + PEER, "/ip6/::1/udp/9090/quic-v1", "/dns4/example.com/tcp/443/wss"):
        addr = Multiaddr(s)
        assert Multiaddr.from_bytes(addr.to_bytes()) == addr


def test_items_and_iteration():
    addr = Multiaddr("/ip6/::1/udp/9090/quic-v1")
    expected = [("ip6", "::1"), ("udp", "9090"), ("quic-v1", None)]
    assert addr.items() == expected
    assert list(addr) == expected
    assert len(addr) == 3


def test_protocols():
    assert Multiaddr(ADDR + PEER).protocols() == ["ip4", "tcp", "p2p"]


def test_encapsulate_string_and_multiaddr():
    base = Multiaddr(ADDR)
    assert str(base.encapsulate(PEER)) == ADDR + PEER
    assert str(base.encapsulate(Multiaddr(PEER))) == ADDR + PEER
    assert str(base) == ADDR  # original unchanged


def test_decapsulate_removes_suffix():
    addr = Multiaddr(ADDR + PEER)
    assert str(addr.decapsulate(PEER)) == ADDR
    assert str(addr.decapsulate("/tcp/4001")) == "/ip4/127.0.0.1"


def test_decapsulate_missing_returns_unchanged():
    addr = Multiaddr(ADDR)
    assert addr.decapsulate("/udp/1234") == addr


def test_decapsulate_everything_yields_empty():
    addr = Multiaddr(ADDR)
    empty = addr.decapsulate("/ip4/127.0.0.1")
    assert len(empty) == 0
    assert empty.to_bytes() == b""


def test_rejects_invalid_address():
    with pytest.raises(MultiformatsError, match="invalid multiaddr"):
        Multiaddr("/ip4/999.0.0.1")
    with pytest.raises(MultiformatsError, match="invalid multiaddr"):
        Multiaddr("no-leading-slash")


def test_from_bytes_rejects_garbage():
    with pytest.raises(MultiformatsError, match="invalid multiaddr"):
        Multiaddr.from_bytes(b"\xff\xff\xff")


def test_encapsulate_rejects_wrong_type():
    with pytest.raises(MultiformatsError, match="expected a Multiaddr"):
        Multiaddr(ADDR).encapsulate(42)


def test_equality_and_hashing():
    a = Multiaddr(ADDR)
    b = Multiaddr(ADDR)
    c = Multiaddr("/ip4/127.0.0.1/tcp/4002")
    assert a == b
    assert a != c
    assert len({a, b, c}) == 2
