"""MultiTierCache — manages an L1 → L2 → L3 hierarchy with promotion/demotion."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Hashable, Optional

from .layer import CacheLayer
from .stats import TierStats


@dataclass
class MultiTierCache:
    """A hierarchy of :class:`CacheLayer` tiers (typically L1→L2→L3).

    On a miss at tier *i*, the next tier is checked.  On a hit, the item
    is *promoted* to the fastest tier (L1).  When L1 is full, items are
    *demoted* to the next tier before eviction.

    Parameters
    ----------
    tiers : list[CacheLayer]
        Ordered from fastest (L1) to slowest (L3+).
    promote_on_hit : bool
        If ``True``, a hit in a slower tier copies the item into L1.
    """

    tiers: list[CacheLayer]
    promote_on_hit: bool = True
    _stats: list[TierStats] = field(default_factory=list, init=False, repr=False)

    def __post_init__(self) -> None:
        if not self.tiers:
            raise ValueError("Must provide at least one tier")
        self._stats = [TierStats(tier_name=t.name) for t in self.tiers]

    # ------------------------------------------------------------------
    # Core API
    # ------------------------------------------------------------------

    def get(self, key: Hashable) -> Optional[Any]:
        """Look up *key* across tiers, promoting on hit if configured."""
        for i, tier in enumerate(self.tiers):
            value = tier.get(key)
            if value is not None:
                self._stats[i].record_hit()
                # Promote to L1
                if self.promote_on_hit and i > 0:
                    size = self._estimate_size(value)
                    self._promote(key, value, size, from_tier=i)
                return value
            else:
                self._stats[i].record_miss()

        return None

    def put(self, key: Hashable, value: Any, size_bytes: int = 1) -> None:
        """Insert into L1 (fastest tier)."""
        self.tiers[0].put(key, value, size_bytes)
        self._stats[0].record_put()

    def delete(self, key: Hashable) -> bool:
        """Delete from all tiers."""
        found = False
        for tier in self.tiers:
            if tier.delete(key):
                found = True
        return found

    def clear(self) -> None:
        """Clear every tier."""
        for tier in self.tiers:
            tier.clear()

    # ------------------------------------------------------------------
    # Stats
    # ------------------------------------------------------------------

    def stats(self) -> list[TierStats]:
        """Return per-tier statistics snapshots."""
        return list(self._stats)

    def tier_stats(self, tier_name: str) -> Optional[TierStats]:
        for s in self._stats:
            if s.tier_name == tier_name:
                return s
        return None

    # ------------------------------------------------------------------
    # Internal
    # ------------------------------------------------------------------

    def _promote(self, key: Hashable, value: Any, size: int, from_tier: int) -> None:
        """Move an item up to L1, demoting from L1 if necessary."""
        target = self.tiers[0]
        # If L1 is full, demote victim to the tier the promoted item came from (or next tier)
        if target.free_bytes < size:
            # Demote the LRU from L1 down one level
            if len(target._store) > 0:
                victim_key, victim_entry = target._store.popitem(last=False)
                target._current_bytes -= victim_entry.size_bytes
                # Place into next tier
                demotion_tier_idx = min(from_tier, len(self.tiers) - 1)
                self.tiers[demotion_tier_idx].put(
                    victim_key, victim_entry.value, victim_entry.size_bytes
                )
        target.put(key, value, size)

    @staticmethod
    def _estimate_size(value: Any) -> int:
        """Rough size estimate for promotion when exact size is unknown."""
        try:
            return len(value)  # type: ignore[arg-type]
        except TypeError:
            return 1

    # Convenience -------------------------------------------------------

    @classmethod
    def standard(
        cls,
        l1_bytes: int = 1024,
        l2_bytes: int = 10_240,
        l3_bytes: int = 102_400,
        l1_items: Optional[int] = None,
        l2_items: Optional[int] = None,
        l3_items: Optional[int] = None,
    ) -> "MultiTierCache":
        """Create a typical 3-tier cache with sensible defaults."""
        return cls(
            tiers=[
                CacheLayer("L1", l1_bytes, max_items=l1_items),
                CacheLayer("L2", l2_bytes, max_items=l2_items),
                CacheLayer("L3", l3_bytes, max_items=l3_items),
            ]
        )
