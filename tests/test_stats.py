"""Tests for TierStats — metrics tracking."""

from cache_layer_optimizer.stats import TierStats


class TestTierStatsBasic:
    def test_initial_state(self):
        s = TierStats("L1")
        assert s.hits == 0
        assert s.misses == 0
        assert s.evictions == 0
        assert s.puts == 0
        assert s.hit_rate() == 0.0
        assert s.miss_rate() == 0.0

    def test_record_hit(self):
        s = TierStats("L1")
        s.record_hit()
        s.record_hit()
        assert s.hits == 2

    def test_record_miss(self):
        s = TierStats("L1")
        s.record_miss()
        assert s.misses == 1

    def test_hit_rate(self):
        s = TierStats("L1")
        for _ in range(7):
            s.record_hit()
        for _ in range(3):
            s.record_miss()
        assert abs(s.hit_rate() - 0.7) < 1e-9
        assert abs(s.miss_rate() - 0.3) < 1e-9

    def test_eviction_rate(self):
        s = TierStats("L1")
        s.record_put()
        s.record_put()
        s.record_put()
        s.record_eviction()
        assert abs(s.eviction_rate() - 1 / 3) < 1e-9

    def test_total_lookups(self):
        s = TierStats("L1")
        s.record_hit()
        s.record_miss()
        assert s.total_lookups == 2

    def test_summary(self):
        s = TierStats("L1")
        s.record_hit()
        s.record_miss()
        summary = s.summary()
        assert summary["tier"] == "L1"
        assert summary["hits"] == 1
        assert summary["misses"] == 1
        assert summary["hit_rate"] == 0.5

    def test_reset(self):
        s = TierStats("L1")
        s.record_hit()
        s.record_miss()
        s.record_put()
        s.record_eviction()
        s.reset()
        assert s.hits == 0
        assert s.misses == 0
        assert s.evictions == 0
        assert s.puts == 0
