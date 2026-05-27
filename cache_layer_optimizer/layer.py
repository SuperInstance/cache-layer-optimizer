"""CacheLayer — a single cache tier with size limits and eviction policies."""

from __future__ import annotations

import time
from collections import OrderedDict
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Hashable, Optional


class EvictionPolicy(Enum):
    """Eviction strategy for a cache layer."""
    LRU = "lru"
    LFU = "lfu"
    FIFO = "fifo"


@dataclass
class _Entry:
    value: Any
    size_bytes: int
    freq: int = 0
    inserted_at: float = 0.0
    last_access: float = 0.0


@dataclass
class CacheLayer:
    """A single cache tier (L1 / L2 / L3) with configurable eviction.

    Parameters
    ----------
    name : str
        Human-readable tier name, e.g. ``"L1"``.
    max_size_bytes : int
        Maximum bytes this tier may hold before eviction kicks in.
    max_items : int | None
        Optional hard cap on the number of items. ``None`` means unlimited.
    eviction : EvictionPolicy
        Which eviction strategy to use when the layer is full.
    """

    name: str
    max_size_bytes: int
    max_items: Optional[int] = None
    eviction: EvictionPolicy = EvictionPolicy.LRU

    # internal state
    _store: OrderedDict[Hashable, _Entry] = field(
        default_factory=OrderedDict, init=False, repr=False
    )
    _current_bytes: int = field(default=0, init=False, repr=False)

    # ------------------------------------------------------------------
    # Core operations
    # ------------------------------------------------------------------

    def get(self, key: Hashable) -> Optional[Any]:
        """Retrieve a value. Returns ``None`` on miss."""
        entry = self._store.get(key)
        if entry is None:
            return None
        entry.freq += 1
        entry.last_access = time.monotonic()
        if self.eviction == EvictionPolicy.LRU:
            self._store.move_to_end(key)
        return entry.value

    def put(self, key: Hashable, value: Any, size_bytes: int = 1) -> None:
        """Store *value* under *key*, evicting if necessary."""
        # If key already exists, remove old entry first
        if key in self._store:
            self._remove_entry(key)

        entry = _Entry(
            value=value,
            size_bytes=size_bytes,
            freq=0,
            inserted_at=time.monotonic(),
            last_access=time.monotonic(),
        )

        # Evict until there's room
        while self._current_bytes + size_bytes > self.max_size_bytes and self._store:
            self._evict_one()
        if self.max_items is not None:
            while len(self._store) >= self.max_items and self._store:
                self._evict_one()

        self._store[key] = entry
        self._current_bytes += size_bytes

    def delete(self, key: Hashable) -> bool:
        """Remove *key*. Returns ``True`` if it existed."""
        if key not in self._store:
            return False
        self._remove_entry(key)
        return True

    def contains(self, key: Hashable) -> bool:
        return key in self._store

    def clear(self) -> None:
        """Drop all items."""
        self._store.clear()
        self._current_bytes = 0

    # ------------------------------------------------------------------
    # Introspection
    # ------------------------------------------------------------------

    @property
    def item_count(self) -> int:
        return len(self._store)

    @property
    def used_bytes(self) -> int:
        return self._current_bytes

    @property
    def free_bytes(self) -> int:
        return self.max_size_bytes - self._current_bytes

    @property
    def utilization(self) -> float:
        """Return utilization as 0.0–1.0."""
        if self.max_size_bytes == 0:
            return 0.0
        return self._current_bytes / self.max_size_bytes

    def keys(self) -> list[Hashable]:
        return list(self._store.keys())

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _remove_entry(self, key: Hashable) -> None:
        entry = self._store.pop(key)
        self._current_bytes -= entry.size_bytes

    def _evict_one(self) -> Hashable:
        """Evict exactly one item according to the configured policy."""
        if not self._store:
            raise RuntimeError("Nothing to evict")

        if self.eviction == EvictionPolicy.LRU:
            key, entry = self._store.popitem(last=False)
        elif self.eviction == EvictionPolicy.LFU:
            key = min(self._store, key=lambda k: self._store[k].freq)
            entry = self._store.pop(key)
        elif self.eviction == EvictionPolicy.FIFO:
            key, entry = self._store.popitem(last=False)
        else:
            raise ValueError(f"Unknown eviction policy: {self.eviction}")

        self._current_bytes -= entry.size_bytes
        return key
