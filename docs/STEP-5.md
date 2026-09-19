# Step 5 — World / Terrain Editor

## Goal

Make the simulated environment configurable from the GUI and guarantee that the
world shown in Godot is the same geometry the Rust physics backend evaluates.

## Terrain

The world can use four terrain modes:

- Flat
- Slope
- Hills
- Stairs

Terrain parameters are editable:

- slope angle
- hill height
- hill wavelength
- stair height
- stair depth

Hills and stairs are represented by deterministic static collision tiles in the
Rust backend. Flat and slope terrain use a single large collider when no
gap/pit cutouts are needed.

## World physics

The editor exposes:

- gravity X / Y / Z
- ground friction
- deterministic world seed

These values are passed to every relevant simulation path, including creature
runs, champion playback, evolution, the single-box viewer, and benchmarks.

## Obstacles

Any combination of these obstacle types can be enabled:

- walls
- blocks
- gaps
- pits

The editor also exposes:

- obstacle count
- obstacle spacing
- obstacle size
- gap width
- pit depth

Obstacle placement and size variation are deterministic for a given world seed.

Gaps remove ground collision tiles. Pits replace the normal floor with a lower
floor. Walls and blocks are static collision boxes.

## One authoritative world

`WorldConfig` lives in the Rust core and is the authoritative description of
the environment.

Rust generates a list of exact static collision boxes from that configuration.
The CLI exposes the same generator through `world-geometry`. Godot calls that
command when world settings change and renders the returned boxes, so the 3D
preview is generated from the same geometry used by physics rather than from a
separate approximation.

When a simulation or evolution run starts, the backend also sends its actual
world geometry in the start event and Godot refreshes the preview from that
data.

## Persistence

World editor values are saved to the existing Godot settings file and restored
on the next launch.

World controls are locked while a simulation/evolution job is running so the
visible configuration cannot silently diverge from the world currently being
evaluated.

## CLI

All simulation commands accept a serialized world configuration through
`--world-json`:

```text
evolab creature-stream --event-port 47821 --world-json <WorldConfig JSON>
evolab evolve --world-json <WorldConfig JSON>
evolab stream --event-port 47821 --world-json <WorldConfig JSON>
evolab probe --world-json <WorldConfig JSON>
```

The exact generated geometry can be inspected with:

```text
evolab world-geometry --world-json <WorldConfig JSON>
```

## Current limits

Step 5 covers static terrain and static obstacles. Moving platforms, movable
objects/payloads, fluids, wind/drag, resources, hazards/damage, and generation
timeline changes remain later milestones.
