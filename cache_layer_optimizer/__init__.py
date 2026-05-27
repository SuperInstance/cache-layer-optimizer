"""cache-layer-optimizer: Multi-tier cache optimization strategies.

Provides L1/L2/L3 cache layer management, intelligent eviction,
placement policies, hit-rate-based optimization, and per-tier metrics.
"""

from .layer import CacheLayer, EvictionPolicy
from .multitier import MultiTierCache
from .optimizer import CacheOptimizer, OptimizationResult
from .policy import PlacementPolicy, PlacementDecision
from .stats import TierStats

__all__ = [
    "CacheLayer",
    "EvictionPolicy",
    "MultiTierCache",
    "CacheOptimizer",
    "OptimizationResult",
    "PlacementPolicy",
    "PlacementDecision",
    "TierStats",
]

__version__ = "0.1.0"
