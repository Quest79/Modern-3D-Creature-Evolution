# Step 6 — Experiment Timeline / Scheduling

## Goal

Turn the existing body, brain, fitness, and world systems into a complete
experiment that can change its rules over generations and be saved, loaded,
forked, and reproduced.

## Generation timeline

The Godot control room now has an **Experiment timeline** section.

A timeline keyframe snapshots the current:

- world / terrain configuration
- fitness weights
- population size
- mutations per child
- structural mutation chance
- motor strength
- trial duration
- trials per creature
- trial aggregation method

Choose a generation, configure the experiment settings you want from that point
forward, and press **Add Snapshot**.

Unconditional keyframes take effect starting at their generation and remain
active until a later keyframe overrides the same settings.

## Conditional events

A keyframe can also wait for a condition after its minimum generation:

- best fitness reaches a threshold
- average fitness reaches a threshold
- best distance reaches a threshold

A conditional keyframe triggers once, remains active afterward, and affects the
next generation after the condition is observed. The GUI reports newly
triggered timeline events while evolution is running.

## Repeated trials

Evolution can now evaluate each creature over multiple deterministic trial
seeds.

Available aggregation modes:

- mean
- median
- worst
- best

The trial count and aggregation mode can both be changed by timeline keyframes.

The evaluation counter reflects the actual number of simulated trials, not just
the number of genomes.

## Scheduled motor strength

SimulationConfig now includes a motor-strength multiplier. Timeline keyframes
can change it without modifying the creature genome.

The multiplier scales motor stiffness, damping, maximum torque, and the
actuator-effort proxy used by fitness.

## Population changes

Population size can change at a timeline boundary. The next generation is bred
directly to the scheduled size while preserving elitism and tournament
selection constraints.

## Champion handling

Because fitness/world rules can change during an experiment, the final champion
is the best creature from the final generation rather than an incomparable
all-time raw fitness score from an earlier ruleset.

The GUI stores the final scheduled world and motor strength with the completed
run. **Watch Champion** therefore replays the champion in the environment and
motor-strength setting under which its final score was measured.

## Experiment files

The GUI can now:

- **Save Experiment...**
- **Load Experiment...**
- **Fork...**

Experiment files use the `.evo` extension and contain human-readable JSON with:

- format version
- experiment name
- ancestor creature genome
- evolution settings
- physics/simulation settings
- fitness
- mutation settings
- world
- repeated-trial settings
- complete generation timeline

The Rust core also defines and validates the same `ExperimentFile` schema.

## Headless experiment runs

A saved experiment can be run directly:

```bash
evolab run FastWalker.evo
```

Optional examples:

```bash
evolab run FastWalker.evo --workers 24
evolab run FastWalker.evo --champion-output champion.json
evolab run FastWalker.evo --json
```

The regular `evolve` command also exposes timeline scheduling directly through
`--timeline-json`, plus:

- `--motor-strength`
- `--trials`
- `--trial-aggregation`
- `--structural-mutation-chance`

## Determinism

Trial seeds are derived deterministically from the configured world seed and
trial index. The same experiment file and worker-independent deterministic
physics settings therefore reproduce the same requested trial worlds.

## Current boundary

Step 6 schedules the systems currently implemented. Future timeline fields can
extend the same model for novelty/speciation, target tasks, moving objects,
hazards, complexity caps, GPU scheduling, and later evolution algorithms.
