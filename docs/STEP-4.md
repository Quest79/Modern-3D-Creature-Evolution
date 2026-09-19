# Step 4 — Configurable Fitness

## Goal

Replace the hard-wired distance score with a configurable weighted fitness
system that can reward or penalize different behaviors without changing code.

## Implemented metrics

Each creature evaluation now records:

- horizontal distance traveled
- average horizontal speed
- average upright posture
- stability based on root angular velocity
- actuator effort / energy proxy

The final fitness score is a weighted sum:

```text
fitness =
    distance_weight  * distance
  + speed_weight     * average_speed
  + upright_weight   * upright
  + stability_weight * stability
  + energy_weight    * energy
```

Use a negative energy weight to penalize actuator effort.

## Energy metric

The current energy value is an actuator-effort proxy, not an exact electrical
or metabolic model. For every motor and physics step, the simulator estimates a
limited torque demand from position error, motor stiffness, damping, and joint
angular velocity, then integrates approximate mechanical work over time.

This gives evolution a useful pressure toward lower-effort motion while keeping
the calculation deterministic and inexpensive.

## Godot fitness editor

The Evolution panel now contains editable weights for:

- Distance
- Average speed
- Upright
- Stability
- Energy / effort

The values are saved in the application's settings and restored on the next
launch.

During evolution, the GUI displays the champion's full metric breakdown rather
than treating every fitness value as meters.

## CLI

The same controls are available headlessly:

```bash
evolab evolve \
  --fitness-distance 1.0 \
  --fitness-speed 0.5 \
  --fitness-upright 2.0 \
  --fitness-stability 1.0 \
  --fitness-energy -0.02
```

## Next fitness work

Planned additions include:

1. repeated trials with deterministic trial seeds
2. mean / median / worst / best aggregation
3. jump height, survival, climbing, object/payload, collision and damage metrics
4. free-form fitness expressions
5. saving fitness/world/evolution settings together as a full experiment file
6. generation timeline automation for changing fitness weights over time
