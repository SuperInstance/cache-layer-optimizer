"""Tests for CacheOptimizer — tuning tier sizes based on hit rates."""

import pytest
from cache_layer_optimizer.layer import CacheLayer
from cache_layer_optimizer.multitier import MultiTierCache
from cache_layer_optimizer.optimizer import CacheOptimizer


class TestOptimizerBasic:
    def test_overall_hit_rate_empty(self):
        cache = MultiTierCache.standard()
        opt = CacheOptimizer(cache)
        assert opt.overall_hit_rate() == 0.0

    def test_overall_hit_rate_all_hits(self):
        cache = MultiTierCache.standard(l1_bytes=1000, l2_bytes=10000, l3_bytes=100000)
        for i in range(10):
            cache.put(f"k{i}", i, size_bytes=1)
        for i in range(10):
            cache.get(f"k{i}")
        opt = CacheOptimizer(cache)
        assert opt.overall_hit_rate() == 1.0

    def test_optimize_returns_result(self):
        cache = MultiTierCache.standard(l1_bytes=10, l2_bytes=100, l3_bytes=1000)
        opt = CacheOptimizer(cache, target_hit_rate=0.5)
        result = opt.optimize()
        assert hasattr(result, "hit_rate_before")
        assert hasattr(result, "hit_rate_after")
        assert isinstance(result.adjustments, dict)

    def test_optimize_adjusts_tiers(self):
        # Create a scenario where L1 is tiny and gets many misses
        l1 = CacheLayer("L1", max_size_bytes=5)
        l2 = CacheLayer("L2", max_size_bytes=100)
        l3 = CacheLayer("L3", max_size_bytes=1000)
        cache = MultiTierCache([l1, l2, l3])

        # Flood with data → L1 will miss a lot
        for i in range(20):
            cache.put(f"k{i}", i, size_bytes=1)
        for i in range(20):
            cache.get(f"k{i}")

        opt = CacheOptimizer(cache, target_hit_rate=0.95, min_tier_bytes=2)
        result = opt.optimize()
        # L1 should have been grown (or L2/L3 shrunk)
        assert len(result.adjustments) > 0 or result.hit_rate_after >= opt.target_hit_rate


class TestRecommend:
    def test_recommend_does_not_modify(self):
        cache = MultiTierCache.standard(l1_bytes=100, l2_bytes=1000, l3_bytes=10000)
        original_sizes = {t.name: t.max_size_bytes for t in cache.tiers}
        opt = CacheOptimizer(cache)
        opt.recommend()
        for t in cache.tiers:
            assert t.max_size_bytes == original_sizes[t.name]
