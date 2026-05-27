"""Tests for CacheLayer — eviction, size limits, core ops."""

from cache_layer_optimizer.layer import CacheLayer, EvictionPolicy


class TestCacheLayerBasic:
    def test_put_and_get(self):
        layer = CacheLayer("L1", max_size_bytes=100)
        layer.put("k1", "v1", size_bytes=1)
        assert layer.get("k1") == "v1"

    def test_get_miss(self):
        layer = CacheLayer("L1", max_size_bytes=100)
        assert layer.get("missing") is None

    def test_overwrite(self):
        layer = CacheLayer("L1", max_size_bytes=100)
        layer.put("k1", "old", size_bytes=1)
        layer.put("k1", "new", size_bytes=1)
        assert layer.get("k1") == "new"
        assert layer.item_count == 1

    def test_delete(self):
        layer = CacheLayer("L1", max_size_bytes=100)
        layer.put("k1", "v1", size_bytes=1)
        assert layer.delete("k1") is True
        assert layer.get("k1") is None
        assert layer.delete("k1") is False

    def test_contains(self):
        layer = CacheLayer("L1", max_size_bytes=100)
        layer.put("k1", "v1", size_bytes=1)
        assert layer.contains("k1")
        assert not layer.contains("k2")

    def test_clear(self):
        layer = CacheLayer("L1", max_size_bytes=100)
        for i in range(10):
            layer.put(f"k{i}", i, size_bytes=1)
        layer.clear()
        assert layer.item_count == 0
        assert layer.used_bytes == 0


class TestEviction:
    def test_lru_eviction(self):
        layer = CacheLayer("L1", max_size_bytes=3, eviction=EvictionPolicy.LRU)
        layer.put("a", 1, size_bytes=1)
        layer.put("b", 2, size_bytes=1)
        layer.put("c", 3, size_bytes=1)
        # Access "a" so it's most-recently used
        layer.get("a")
        # Insert "d" → evicts "b" (LRU)
        layer.put("d", 4, size_bytes=1)
        assert layer.get("a") == 1
        assert layer.get("b") is None
        assert layer.get("d") == 4

    def test_fifo_eviction(self):
        layer = CacheLayer("L1", max_size_bytes=3, eviction=EvictionPolicy.FIFO)
        layer.put("a", 1, size_bytes=1)
        layer.put("b", 2, size_bytes=1)
        layer.put("c", 3, size_bytes=1)
        layer.put("d", 4, size_bytes=1)
        assert layer.get("a") is None  # first in, first out
        assert layer.get("d") == 4

    def test_lfu_eviction(self):
        layer = CacheLayer("L1", max_size_bytes=3, eviction=EvictionPolicy.LFU)
        layer.put("a", 1, size_bytes=1)
        layer.put("b", 2, size_bytes=1)
        layer.put("c", 3, size_bytes=1)
        # Access "a" and "b" many times
        for _ in range(10):
            layer.get("a")
            layer.get("b")
        # "c" has freq 0 → evicted first
        layer.put("d", 4, size_bytes=1)
        assert layer.get("c") is None
        assert layer.get("a") == 1

    def test_max_items_cap(self):
        layer = CacheLayer("L1", max_size_bytes=10000, max_items=3)
        layer.put("a", 1, size_bytes=1)
        layer.put("b", 2, size_bytes=1)
        layer.put("c", 3, size_bytes=1)
        layer.put("d", 4, size_bytes=1)
        assert layer.item_count == 3

    def test_large_item_eviction(self):
        layer = CacheLayer("L1", max_size_bytes=5)
        layer.put("a", 1, size_bytes=2)
        layer.put("b", 2, size_bytes=2)
        # This item is size 5 — evicts both a and b
        layer.put("big", 99, size_bytes=5)
        assert layer.get("big") == 99
        assert layer.item_count == 1


class TestUtilization:
    def test_utilization_empty(self):
        layer = CacheLayer("L1", max_size_bytes=100)
        assert layer.utilization == 0.0
        assert layer.free_bytes == 100

    def test_utilization_half(self):
        layer = CacheLayer("L1", max_size_bytes=100)
        layer.put("k", "v", size_bytes=50)
        assert abs(layer.utilization - 0.5) < 1e-9

    def test_keys(self):
        layer = CacheLayer("L1", max_size_bytes=100)
        layer.put("x", 1, size_bytes=1)
        layer.put("y", 2, size_bytes=1)
        assert set(layer.keys()) == {"x", "y"}
