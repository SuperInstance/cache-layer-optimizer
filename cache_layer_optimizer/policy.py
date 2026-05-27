"""PlacementPolicy — decides which tier should store a given key."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any, Hashable

from .layer import CacheLayer


class PlacementDecision(Enum):
    """Recommended target tier."""
    L1 = "L1"
    L2 = "L2"
    L3 = "L3"
    REJECT = "REJECT"  # Item too large for any tier


@dataclass
class PlacementPolicy:
    """Decides where to place items based on size and frequency heuristics.

    Parameters
    ----------
    tiers : list[CacheLayer]
        Ordered from fastest (L1) to slowest (L3+).
    l1_size_threshold : float
        Items larger than this fraction of L1's capacity go to L2+.
    l2_size_threshold : float
        Items larger than this fraction of L2's capacity go to L3+.
    """

    tiers: list[CacheLayer]
    l1_size_threshold: float = 0.1
    l2_size_threshold: float = 0.25

    def decide(self, key: Hashable, size_bytes: int) -> PlacementDecision:
        """Choose the best tier for an item of the given size."""
        if len(self.tiers) == 0:
            return PlacementDecision.REJECT

        l1 = self.tiers[0]
        if size_bytes > l1.max_size_bytes:
            # Won't fit in L1 at all
            if len(self.tiers) >= 2 and size_bytes <= self.tiers[1].max_size_bytes:
                if size_bytes <= self.tiers[1].max_size_bytes * self.l2_size_threshold:
                    return PlacementDecision.L2
                if len(self.tiers) >= 3 and size_bytes <= self.tiers[2].max_size_bytes:
                    return PlacementDecision.L3
                return PlacementDecision.L2
            if len(self.tiers) >= 3 and size_bytes <= self.tiers[2].max_size_bytes:
                return PlacementDecision.L3
            return PlacementDecision.REJECT

        if size_bytes <= l1.max_size_bytes * self.l1_size_threshold:
            return PlacementDecision.L1

        # Medium-sized: try L2 first
        if len(self.tiers) >= 2 and size_bytes <= self.tiers[1].max_size_bytes:
            return PlacementDecision.L2

        return PlacementDecision.L1

    def place(self, key: Hashable, value: Any, size_bytes: int) -> PlacementDecision:
        """Decide and actually place the item into the chosen tier."""
        decision = self.decide(key, size_bytes)
        tier = self._tier_for_decision(decision)
        if tier is not None:
            tier.put(key, value, size_bytes)
        return decision

    def _tier_for_decision(self, decision: PlacementDecision) -> CacheLayer | None:
        mapping = {
            PlacementDecision.L1: 0,
            PlacementDecision.L2: 1,
            PlacementDecision.L3: 2,
        }
        idx = mapping.get(decision)
        if idx is not None and idx < len(self.tiers):
            return self.tiers[idx]
        return None
