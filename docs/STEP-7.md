# Step 7 — Results / History / Analysis

## Goal

Persist enough information from every generation to inspect what evolution did,
compare champions, trace ancestry, measure diversity, and provide foundations
for quality-diversity algorithms such as Pareto search and MAP-Elites.

## Persistent results

Evolution can now write a complete `.evoresults` JSON file containing:

- the evolution configuration
- wall-clock runtime
- final champion and metrics
- every generation summary
- the champion from every generation
- lineage records for every evaluated individual
- the full genome for every evaluated individual
- mutation records and exact trial/world seeds for every evaluated individual
- diversity statistics
- analysis-class summaries
- distance/energy Pareto fronts
- MAP-Elites-style occupied analysis cells

The Godot GUI automatically writes the most recent run to its user-data
directory and loads it when evolution finishes.

Headless evolution can write the same format with:

```bash
evolab evolve ... --result-output results.evoresults
evolab run FastWalker.evo --result-output results.evoresults
```

## Individual IDs and lineage

Every individual now receives a deterministic numeric ID.

Each evaluated individual records:

- generation
- parent ID or parent IDs
- fitness
- all fitness metrics
- segment count
- joint count
- brain-node count
- analysis class
- mutation descriptions
- exact evaluation trial/world seeds
- complete genome, including the brain

Elites copied into a later generation receive a new ID with the previous elite
recorded as their parent. Crossover children record both selected parents when
they are different.

This provides explicit ancestry without storing a full duplicate genome for
every population member.

## Champion archive

Every generation stores its complete champion genome plus:

- individual ID
- parents
- analysis class
- fitness
- metric breakdown

The Results window can load any archived generation champion back into the
main viewer for inspection or further mutation.

It can also compare the selected champion's metrics against the final
generation champion.

## History graph

The Results window includes a generation graph for:

- best fitness
- average fitness
- median fitness

The generation browser shows best, average, worst, distance, champion identity,
complexity, diversity, Pareto size, and MAP-Elites occupancy.

## Diversity

Each generation records:

- fitness standard deviation
- average and standard deviation of segment count
- average and standard deviation of brain-node count
- unique morphology count
- analysis-class count

Unique morphology currently uses body topology plus segment dimensions as a
descriptive signature.

## Analysis classes

Step 7 provides species-style browsing groundwork without pretending that the
current GA has speciation selection.

An **analysis class** groups creatures using:

- segment count
- brain-size bucket

The Results UI shows class membership and performance. These classes do not yet
alter mating or survival. True speciation will be added with algorithms such as
NEAT/speciation later.

## Pareto analysis

Each generation calculates a nondominated front using:

- maximize distance
- minimize actuator effort / energy proxy

This is analysis only. The current weighted-fitness GA still performs
selection. The data structures are ready to support NSGA-II or another
multi-objective selection algorithm later.

## MAP-Elites groundwork

Each generation also produces an analysis-only MAP-Elites grid keyed by:

- segment-count bin
- brain-node-count bin

The highest-fitness individual in each occupied cell is recorded.

This is not yet MAP-Elites as the active evolution algorithm; it establishes
the archive representation and UI-visible occupancy needed for that later
algorithm.

## Godot Results window

The new **Results** button opens four views:

- **History** — fitness graph, generation list, detailed metrics
- **Champions** — generation champion archive with load/compare controls
- **Lineage** — every evaluated creature, parent relationships, mutation history, trial seeds, ancestry, exact historical replay, load, compare, and fork controls
- **Classes** — descriptive morphology/brain analysis groups
- **Analysis** — diversity history, analysis classes, Pareto front, and MAP-Elites occupancy

The Results button becomes available after a completed persisted evolution run.
Results files can also be loaded later with **Load Results...**.

Historical replay reconstructs the generation's recorded simulation settings and
world, then runs the retained genome through the normal buffered/interpolated
viewer. This means old creatures can be inspected at any playback speed without
requiring a separate video capture.

## Current boundary

Step 7 adds observation and persistence, not new selection algorithms.
NSGA-II, MAP-Elites, novelty search, true speciation, and lineage-tree
visualization can now build on the persisted analysis structures.
