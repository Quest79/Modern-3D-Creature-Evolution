# Step 2 — Creature Morphology and Mutation

## Implemented

The first Step 2 milestone turns the fixed three-part demo into a reusable,
mutation-ready genome.

### Genome

A creature genome contains:

- body segments with stable IDs
- dimensions
- initial positions
- density and friction
- parent/child joint topology
- attachment anchors
- joint axes
- angular limits
- motor amplitude
- motor frequency
- motor phase
- stiffness
- damping
- maximum torque

The genome is serializable as human-readable JSON.

### Deterministic mutation

Mutation is seed-driven and reproducible. Using the same parent genome, seed,
mutation count, and limits produces the same child genome.

Implemented mutations:

- resize a segment
- change density/friction
- change motor amplitude/frequency/phase/torque
- change joint limits
- move a joint attachment
- add a new segment and motorized joint
- remove a leaf segment

Structural mutation obeys configurable minimum and maximum segment counts.

### Random creature generation

A random morphology can be generated from a numeric seed and target segment
count, then receive additional numeric/structural mutations.

### GUI

The Godot frontend now exposes:

- Seed
- Mutation operations
- Random creature segment count
- Maximum segment count
- Seed creature
- Mutate Current
- Random
- Save Genome
- Load Genome
- live creature playback

The current genome can be repeatedly mutated, saved, reloaded, and simulated.

### CLI examples

Generate a six-segment creature and write it to JSON:

```bash
evolab genome-generate --output creature.json --seed 42 --random-segments 6 --mutations 12
```

Stream a saved genome to the viewer transport:

```bash
evolab creature-stream --event-port 47821 --genome creature.json
```

Mutate a loaded parent before simulating it:

```bash
evolab creature-stream --event-port 47821 --genome creature.json --seed 99 --mutations 20
```

## Next Step 2 milestone

The next milestone is population-level evolution:

1. create many offspring from a parent/population
2. evaluate each morphology independently
3. calculate a first fitness score
4. selection + elitism
5. generation loop
6. champion/history tracking
7. show generation progress and the current champion in Godot

That is the point where the project changes from a creature simulator into an
actual evolutionary system.
