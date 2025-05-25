# Spatial Indexing Roadmap

_Last updated: 2025-05-08_

As simulation scales, naive O(n²) loops for fog-of-war and raycasting will become a bottleneck. This document outlines spatial indexing strategies to improve query performance:

## 1. Uniform Grid (Spatial Hash)

- **Structure**: Divide world into an M×M grid of cells (buckets).
- **Insert/Update**: Each moving entity goes into its cell(s) each tick (O(n)).
- **Radius Query**: For an agent, examine its cell + 8 neighbors for nearby entities (O(k), k≪n).
- **Raycasting**: Step cell-by-cell along ray (Digital Differential Analyzer), testing only local bucket entities.
- **Pros**: Simple to implement, low overhead, good for roughly uniform distributions.
- **Cons**: Clustering degrades performance; cell size tuning required.

## 2. Quadtree (BSP Tree)

- **Structure**: Recursively partition region into quadrants until leaf node contains ≤K entities.
- **Insert/Update**: Reinsert moving entities or rebuild tree periodically.
- **Radius Query**: Descend only nodes whose bounding box intersects the query circle.
- **Raycasting**: Traverse nodes overlapping the ray path; prune large regions.
- **Pros**: Adapts to uneven densities, fewer false-positive tests.
- **Cons**: More complex splits/merges, higher constant factors.

## 3. Other Structures

- **k-d Tree / R-Tree**: Good for static or mostly-static scenes, but costly to rebalance on highly dynamic data.
- **BVH**: Bounding volume hierarchies excel at ray-heavy workloads; require per-tick refitting.

## 4. Recommended Path

1. **Prototype Uniform Grid**:
   - Choose cell size ≈ max(scan range, view_range).
   - Maintain 2D Vec of buckets and update entities per tick.
   - Adapt build_view() and scan() to query only relevant cells.
2. **Profile & Tune**:
   - Measure query times, bucket distribution, and worst-case hotspots.
3. **Upgrade to Quadtree** if grid becomes imbalanced or world size grows.

This roadmap can be revisited when large-scale performance becomes critical. For now, focus on core NN-agent features before optimizing spatial queries.
