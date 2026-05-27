"""TierStats — per-layer hit / miss / eviction metrics."""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class TierStats:
    """Accumulates performance metrics for a single cache tier.

    Attributes
    ----------
    tier_name : str
        Name of the tier (e.g. ``"L1"``).
    hits : int
        Number of successful lookups.
    misses : int
        Number of failed lookups.
    evictions : int
        Number of items evicted (not tracked here at the layer level yet;
        increment manually or via the cache layer).
    puts : int
        Number of ``put`` operations.
    """

    tier_name: str
    hits: int = 0
    misses: int = 0
    evictions: int = 0
    puts: int = 0

    # ------------------------------------------------------------------
    # Convenience recorders
    # ------------------------------------------------------------------

    def record_hit(self) -> None:
        self.hits += 1

    def record_miss(self) -> None:
        self.misses += 1

    def record_eviction(self) -> None:
        self.evictions += 1

    def record_put(self) -> None:
        self.puts += 1

    # ------------------------------------------------------------------
    # Derived metrics
    # ------------------------------------------------------------------

    @property
    def total_lookups(self) -> int:
        return self.hits + self.misses

    def hit_rate(self) -> float:
        """Return hit rate in 0.0–1.0."""
        if self.total_lookups == 0:
            return 0.0
        return self.hits / self.total_lookups

    def miss_rate(self) -> float:
        if self.total_lookups == 0:
            return 0.0
        return self.misses / self.total_lookups

    def eviction_rate(self) -> float:
        """Evictions per put."""
        if self.puts == 0:
            return 0.0
        return self.evictions / self.puts

    def summary(self) -> dict[str, int | float]:
        return {
            "tier": self.tier_name,
            "hits": self.hits,
            "misses": self.misses,
            "evictions": self.evictions,
            "puts": self.puts,
            "hit_rate": round(self.hit_rate(), 4),
            "miss_rate": round(self.miss_rate(), 4),
            "eviction_rate": round(self.eviction_rate(), 4),
        }

    def reset(self) -> None:
        self.hits = 0
        self.misses = 0
        self.evictions = 0
        self.puts = 0
