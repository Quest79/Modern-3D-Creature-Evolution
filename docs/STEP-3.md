# Step 3 — Evolvable Expression-Tree Brains

## Goal

Move joint control out of fixed sine-wave code and into the creature genome so
evolution can change both morphology and behavior.

## Implemented

Each creature now carries a serialized expression-tree brain. Every motorized
joint can have its own output expression.

### Sensors

The first controller sensor set includes:

- simulation time
- root height
- root X/Y/Z linear velocity
- root X/Y/Z angular velocity
- root quaternion X/Y/Z/W orientation

### Expression nodes

The initial 3DVCE-style expression language supports:

- constant
- sensor
- add
- subtract
- multiply
- negate
- sine
- cosine
- clamp

Expression trees are bounded in depth and non-finite results are sanitized
before they reach the physics motor.

### Outputs

A brain output targets one joint. The evaluated expression becomes the joint's
motor target angle, clamped by that joint's physical angular limits.

The original three-segment walker is automatically converted to expression
trees equivalent to its previous sine controllers. Older JSON genomes that do
not yet contain a brain still load: their legacy motor settings are used as a
fallback and are converted to explicit outputs as soon as they are mutated.

### Brain mutation

Brain and body now mutate in the same genome.

Current brain mutations can:

- perturb constants
- replace a complete joint-output expression with a new random tree
- wrap an existing expression with sine, cosine, negate, add, or multiply

New structural joints automatically receive a controller output. Removing a
limb also removes its obsolete output.

### Population evolution

The existing population loop now evaluates the expression-tree controller on
every physics step. Selection therefore acts on the combined body + brain.

Generation events now report champion brain-node count in addition to body
complexity. Godot displays that value while evolution is running.

## Next milestones

The next controller milestones are:

1. joint-angle and joint-velocity sensors
2. contact/touch sensors
3. explicit per-trait sensor allow/deny configuration
4. subtree crossover between parent brains
5. finer subtree-level mutations instead of whole-output replacement
6. recurrent/controller state
7. additional controller families such as CPG, MLP, RNN/GRU, and NEAT

The expression-tree controller remains one pluggable brain type rather than
locking the project to a single controller architecture.
