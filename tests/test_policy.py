"""Tests for PlacementPolicy — tier placement decisions."""

from cache_layer_optimizer.layer import CacheLayer
from cache_layer_optimizer.policy import PlacementDecision, PlacementPolicy


class TestPlacementDecide:
    def test_small_item_goes_l1(self):
        policy = PlacementPolicy([
            CacheLayer("L1", max_size_bytes=1000),
            CacheLayer("L2", max_size_bytes=10000),
            CacheLayer("L3", max_size_bytes=100000),
        ])
        assert policy.decide("k1", 10) == PlacementDecision.L1

    def test_medium_item_goes_l2(self):
        policy = PlacementPolicy([
            CacheLayer("L1", max_size_bytes=100),
            CacheLayer("L2", max_size_bytes=10000),
            CacheLayer("L3", max_size_bytes=100000),
        ])
        # Size > 10% of L1 (10) but fits in L2
        assert policy.decide("k1", 50) == PlacementDecision.L2

    def test_large_item_goes_l3(self):
        policy = PlacementPolicy([
            CacheLayer("L1", max_size_bytes=100),
            CacheLayer("L2", max_size_bytes=500),
            CacheLayer("L3", max_size_bytes=100000),
        ])
        assert policy.decide("k1", 800) == PlacementDecision.L3

    def test_too_large_rejected(self):
        policy = PlacementPolicy([
            CacheLayer("L1", max_size_bytes=100),
            CacheLayer("L2", max_size_bytes=500),
            CacheLayer("L3", max_size_bytes=1000),
        ])
        assert policy.decide("k1", 5000) == PlacementDecision.REJECT

    def test_empty_tiers_reject(self):
        policy = PlacementPolicy([])
        assert policy.decide("k1", 10) == PlacementDecision.REJECT


class TestPlacementPlace:
    def test_place_actually_stores(self):
        policy = PlacementPolicy([
            CacheLayer("L1", max_size_bytes=1000),
            CacheLayer("L2", max_size_bytes=10000),
            CacheLayer("L3", max_size_bytes=100000),
        ])
        decision = policy.place("k1", "v1", size_bytes=10)
        assert decision == PlacementDecision.L1
        assert policy.tiers[0].get("k1") == "v1"

    def test_place_rejected_not_stored(self):
        policy = PlacementPolicy([
            CacheLayer("L1", max_size_bytes=10),
        ])
        decision = policy.place("k1", "v1", size_bytes=100)
        assert decision == PlacementDecision.REJECT
        assert policy.tiers[0].get("k1") is None
