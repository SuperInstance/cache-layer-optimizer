"""Tests for MultiTierCache — hierarchy, promotion, demotion."""

from cache_layer_optimizer.layer import CacheLayer
from cache_layer_optimizer.multitier import MultiTierCache


class TestMultiTierBasic:
    def test_put_and_get_l1(self):
        cache = MultiTierCache.standard(l1_bytes=100, l2_bytes=1000, l3_bytes=10000)
        cache.put("k1", "v1", size_bytes=1)
        assert cache.get("k1") == "v1"

    def test_miss_all_tiers(self):
        cache = MultiTierCache.standard()
        assert cache.get("nonexistent") is None

    def test_delete_across_tiers(self):
        cache = MultiTierCache.standard(l1_bytes=10, l2_bytes=100, l3_bytes=1000)
        cache.put("k1", "v1", size_bytes=1)
        assert cache.delete("k1") is True
        assert cache.get("k1") is None

    def test_clear_all_tiers(self):
        cache = MultiTierCache.standard()
        for i in range(5):
            cache.put(f"k{i}", i, size_bytes=1)
        cache.clear()
        for i in range(5):
            assert cache.get(f"k{i}") is None


class TestPromotion:
    def test_promote_on_hit_disabled(self):
        l1 = CacheLayer("L1", max_size_bytes=5)
        l2 = CacheLayer("L2", max_size_bytes=100)
        cache = MultiTierCache([l1, l2], promote_on_hit=False)
        # Put directly into L2
        l2.put("k1", "v1", size_bytes=1)
        assert cache.get("k1") == "v1"
        # Should NOT be in L1
        assert l1.get("k1") is None

    def test_promote_on_hit_enabled(self):
        l1 = CacheLayer("L1", max_size_bytes=100)
        l2 = CacheLayer("L2", max_size_bytes=100)
        cache = MultiTierCache([l1, l2], promote_on_hit=True)
        # Put into L2 directly
        l2.put("k1", "v1", size_bytes=1)
        assert cache.get("k1") == "v1"
        # Should now be in L1 too
        assert l1.get("k1") == "v1"


class TestStats:
    def test_stats_recorded(self):
        cache = MultiTierCache.standard(l1_bytes=100, l2_bytes=1000, l3_bytes=10000)
        cache.put("k1", "v1", size_bytes=1)
        cache.get("k1")  # hit
        cache.get("missing")  # miss at all tiers

        stats = cache.stats()
        assert stats[0].hits == 1  # L1 hit
        assert stats[0].puts == 1
        assert stats[0].misses == 1  # the "missing" lookup

    def test_tier_stats_by_name(self):
        cache = MultiTierCache.standard()
        s = cache.tier_stats("L2")
        assert s is not None
        assert s.tier_name == "L2"
        assert cache.tier_stats("L99") is None

    def test_requires_at_least_one_tier(self):
        import pytest
        with pytest.raises(ValueError):
            MultiTierCache([])

    def test_standard_convenience(self):
        cache = MultiTierCache.standard()
        assert len(cache.tiers) == 3
        assert cache.tiers[0].name == "L1"
