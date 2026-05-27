"""CacheOptimizer — tunes tier sizes based on observed hit rates."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from .layer import CacheLayer
from .multitier import MultiTierCache
from .stats import TierStats


@dataclass
class OptimizationResult:
    """Result of a single optimization pass."""
    adjustments: dict[str, int]  # tier_name → new max_size_bytes
    hit_rate_before: float
    hit_rate_after: float
    iterations: int = 0


@dataclass
class CacheOptimizer:
    """Analyzes hit-rate statistics and recommends / applies tier resizing.

    The optimizer periodically examines per-tier hit rates and redistributes
    capacity toward tiers with low hit rates (implying they're undersized).

    Parameters
    ----------
    cache : MultiTierCache
        The cache hierarchy to optimize.
    target_hit_rate : float
        Desired overall hit rate (0.0–1.0). Default 0.9.
    min_tier_bytes : int
        Minimum bytes a tier must never shrink below.
    max_iterations : int
        Safety cap on tuning iterations per ``optimize()`` call.
    """

    cache: MultiTierCache
    target_hit_rate: float = 0.9
    min_tier_bytes: int = 256
    max_iterations: int = 10

    def overall_hit_rate(self) -> float:
        """Compute the current overall hit rate across all tiers."""
        total_hits = sum(s.hits for s in self.cache.stats())
        total_lookups = sum(s.hits + s.misses for s in self.cache.stats())
        if total_lookups == 0:
            return 0.0
        return total_hits / total_lookups

    def optimize(self) -> OptimizationResult:
        """Run an optimization pass.

        Strategy: identify the tier with the lowest hit rate and the tier
        with the highest. Shrink the high-hit-rate tier and grow the low-hit-rate
        tier by a calculated step.

        Returns an :class:`OptimizationResult` describing what changed.
        """
        hit_rate_before = self.overall_hit_rate()
        adjustments: dict[str, int] = {}

        for _ in range(self.max_iterations):
            if self.overall_hit_rate() >= self.target_hit_rate:
                break

            stats = self.cache.stats()
            if len(stats) < 2:
                break

            # Find weakest and strongest tiers
            worst = min(stats, key=lambda s: s.hit_rate())
            best = max(stats, key=lambda s: s.hit_rate())

            if worst.tier_name == best.tier_name:
                break

            worst_tier = self._find_tier(worst.tier_name)
            best_tier = self._find_tier(best.tier_name)
            if worst_tier is None or best_tier is None:
                break

            # Calculate step: 10% of best tier's size
            step = max(best_tier.max_size_bytes // 10, 1)

            # Don't shrink best below minimum
            new_best_size = max(best_tier.max_size_bytes - step, self.min_tier_bytes)
            actual_step = best_tier.max_size_bytes - new_best_size

            if actual_step == 0:
                break

            best_tier.max_size_bytes = new_best_size
            worst_tier.max_size_bytes += actual_step

            adjustments[worst.tier_name] = worst_tier.max_size_bytes
            adjustments[best.tier_name] = best_tier.max_size_bytes

        return OptimizationResult(
            adjustments=adjustments,
            hit_rate_before=hit_rate_before,
            hit_rate_after=self.overall_hit_rate(),
        )

    def recommend(self) -> dict[str, int]:
        """Return recommended sizes without applying them."""
        # Snapshot current sizes
        original = {t.name: t.max_size_bytes for t in self.cache.tiers}
        result = self.optimize()
        # Capture new sizes
        recommended = {t.name: t.max_size_bytes for t in self.cache.tiers}
        # Restore
        for t in self.cache.tiers:
            t.max_size_bytes = original[t.name]
        return recommended

    def _find_tier(self, name: str) -> CacheLayer | None:
        for t in self.cache.tiers:
            if t.name == name:
                return t
        return None
