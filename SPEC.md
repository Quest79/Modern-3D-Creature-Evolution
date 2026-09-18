# Modern 3D Creature Evolution — Full Specification

## Project Goal

Build a fully fledged virtual-evolution application capable of evolving complete 3D creatures using methods similar to 3DVCE, while also supporting newer evolutionary approaches and modern hardware acceleration.

The system must let users configure:

- starting creature/body conditions
- allowable body changes and mutation ranges
- creature brains/controllers
- sensors and actuators
- evolutionary algorithms
- fitness/objective functions
- environments and physics
- parameters that change between generations
- automated curriculum/conditional changes
- CPU, GPU, multi-GPU, and distributed execution
- visualization, replay, genealogy, metrics, and experiment forking

A core design rule is:

> Anything that can reasonably evolve should be exposable as an evolvable parameter, and anything that can reasonably change during an experiment should be schedulable over generations.

---

# 1. Simulation Foundation and Execution Engine

The program should be split into two major layers.

## Modern UI / Viewer

- Godot 4.x or another modern desktop UI/3D frontend
- 3D creature viewer
- world editor
- live experiment dashboard
- graphing and metrics
- genome inspector
- brain inspector
- replay viewer
- experiment configuration

## High-Speed Simulation / Evolution Backend

- independent of rendering
- headless operation
- native C++ or Rust core for CPU execution
- optional CUDA/GPU backend
- same experiment file usable in GUI, CLI, local headless, and distributed modes

Rendering must never be required for evolution. A run may evaluate thousands or millions of creatures without drawing them, while the GUI renders only creatures selected for viewing.

## Physics Features

Support at minimum:

- rigid bodies
- boxes
- capsules
- spheres
- convex bodies
- mass and density
- friction
- restitution
- gravity
- hinge joints
- ball/socket joints
- slider joints
- configurable joint limits
- torque motors
- velocity motors
- position/servo motors
- spring/damper behavior
- muscle-like actuators
- self-collision
- collision layers/groups

## Determinism

Every experiment should support:

- explicit random seed
- deterministic fixed timestep
- deterministic replay where practical
- optional faster nondeterministic/asynchronous execution

## Replaceable Compute Backends

The simulation API should allow several backends.

### CPU General Physics

- supports arbitrary body graphs
- handles creatures with different numbers of limbs and joints
- multithreaded evaluation
- easiest development/debugging backend

### GPU Batch Physics

- evaluates hundreds or thousands of creatures/environments simultaneously
- CUDA/NVIDIA backend first
- future AMD/ROCm and Apple/Metal-compatible accelerator paths where practical
- controller inference and fitness calculation can remain on GPU

For efficient batching, use bounded morphology representations such as:

- up to 64 body-part slots
- up to 128 joint slots
- inactive elements masked out

This allows differently shaped creatures to occupy regular GPU-friendly data structures.

---

# 2. Evolvable Creature Genome and Morphology

Each creature owns a genome capable of describing its body, joints, sensors, actuators, and optionally its brain.

## Direct Body Genome

Basic structure:

`Creature -> Parts -> Connections -> Joints -> Actuators -> Sensors`

Each segment may contain evolvable properties such as:

- shape
- dimensions
- position
- orientation
- density
- mass
- friction
- restitution/material
- parent attachment point
- child attachment points
- symmetry/mirroring flags

## Evolvable Joint Properties

- joint type
- axis
- angular/linear limits
- motor strength
- maximum torque
- target velocity
- damping
- stiffness
- actuator type

## Structural Mutations

Support operations including:

- add segment
- remove segment
- duplicate branch
- mirror branch
- resize segment
- rotate segment
- move attachment
- mutate joint type
- mutate actuator
- mutate material
- add/remove sensor
- increase/decrease symmetry
- recursively duplicate structures

## Hard Constraints

Configurable examples:

- body parts: 1-64+
- body depth: 1-16+
- branches per node: 0-8+
- total mass limits
- minimum segment size
- maximum segment size
- maximum joint torque
- allow/disallow body-part interpenetration
- maximum creature bounding size
- symmetry requirements

## Multiple Genome Encodings

The engine should not be limited to one representation.

### Direct Genome

Closest to 3DVCE. The genome explicitly describes body parts and joints.

### Developmental Genome

Stores construction rules rather than every final body part directly, allowing repeated or recursive structures.

### CPPN / Generative Genome

Uses compact generative networks to create structured, symmetric, or repeated morphologies.

### Unified Body + Brain Genome

Allows morphology and control to co-evolve as one organism.

## Starting Population Modes

Users can begin from:

- fully random creatures
- clones of one hand-built creature
- mutations of one seed creature
- several hand-defined species
- imported genomes
- survivors/champions from previous experiments
- random bodies with fixed brains
- fixed bodies with random brains
- partly locked genomes

Individual properties should support a lock state so experiments can hold selected traits constant while evolving the rest.

---

# 3. Modular Creature Brain and Controller System

The application should support interchangeable controller families rather than forcing every experiment to use one neural architecture.

## 3DVCE-Style Expression Brain

Available operators may include:

- constants
- add/subtract/multiply/divide
- sine/cosine
- clamp
- absolute value
- comparisons
- conditionals
- timers
- oscillators
- sensor inputs
- joint/motor outputs
- expression-tree mutation
- expression-tree crossover

## Central Pattern Generator / Oscillator Controller

Useful for locomotion. Evolvable parameters include:

- phase
- amplitude
- frequency
- coupling strength
- limb synchronization

## Fixed Neural Network

- MLP
- selectable hidden layers
- selectable activation functions
- directly evolved weights/biases

## Recurrent Neural Controllers

- RNN
- GRU
- recurrent state / memory

## NEAT-Style Topology Evolution

- add neurons
- add/remove/disable connections
- evolve weights
- historical markings / innovation tracking
- structural crossover
- speciation

## Graph Neural Network Controller

- body segments represented as graph nodes
- joints represented as graph edges
- naturally supports variable morphology
- message passing allows reusable control logic across different body plans

## Sensor Inputs

Potential sensory channels include:

- joint angle
- joint velocity
- joint torque/load
- contact/touch
- segment orientation
- angular velocity
- root velocity
- root acceleration
- distance from ground
- raycast/range sensors
- target direction
- target distance
- energy state
- damage state
- global/relative clock
- sine-wave clock
- previous neural state

## Outputs

- target joint angle
- target joint velocity
- raw torque
- muscle contraction
- grip
- custom task-specific actions

The UI must allow users to control which sensors and outputs evolution is permitted to use. Creatures should not automatically receive perfect environmental information.

---

# 4. Evolution Engine and Algorithm Library

Evolution methods should be modular plugins that consume genomes and evaluation results through a shared API.

## Classic Genetic Algorithm

Support the familiar 3DVCE-style workflow:

- population
- tournament selection
- elitism
- crossover
- subtree crossover
- random-tree crossover
- body mutation
- brain mutation
- structural mutation
- configurable mutation schedules

## Speciation

- group structurally/behaviorally similar creatures
- protect new body plans from immediate extinction
- configurable compatibility thresholds

## NEAT / CPPN-NEAT

Suitable for:

- evolving brain topology
- evolving generative body encodings
- co-evolving body and control

## CMA-ES

For continuous optimization of controller and morphology parameters.

## Multi-Objective / Pareto Evolution

Support methods such as NSGA-II.

Example objectives:

- maximize speed
- minimize energy consumption
- minimize body mass

Do not require users to collapse every objective into one weighted scalar unless they choose to.

## Novelty Search

Reward behavioral difference rather than only objective performance to help escape local optima.

## Novelty Search + Local Competition

Combine behavioral exploration with performance pressure.

## MAP-Elites / Quality Diversity

Maintain the best creature found across behavioral or morphological niches.

Example archive dimensions:

- X axis: creature mass
- Y axis: number of limbs
- each cell: fastest creature in that niche

This can preserve many distinct successful designs instead of converging on one champion.

## CMA-ME / CMA-MAE and Future QD Methods

Quality-diversity algorithms should be implemented through modular strategy interfaces so newer methods can be added without altering the simulator.

## Hybrid Evolution + Learning

Support experiments such as:

`Evolution chooses body -> RL trains controller -> resulting score returned to evolution`

or:

`Evolution produces body + initial brain -> brain learns during lifetime`

This enables Baldwinian/Lamarckian-style experiments and modern morphology-control co-optimization.

---

# 5. World, Fitness, Starting Conditions, and Generation Scheduling

This is the main experiment-design layer.

## World Editor

Users should be able to configure:

- terrain dimensions
- flat ground
- slopes
- hills
- procedural terrain
- stairs
- pits
- gaps
- obstacles
- walls
- movable objects
- payloads
- targets
- food/resources
- hazards
- gravity
- friction
- atmosphere/drag
- wind
- lighting for visualization
- water/fluid environments later

## Fitness / Objective Editor

Fitness should be constructed with a node/expression system rather than hardcoded boxes.

Examples:

`Fitness = Distance`

`Fitness = Distance * 2 - Energy * 0.25 - Falling * 10`

Built-in measurements should include:

- total distance
- forward distance
- average speed
- maximum speed
- acceleration
- survival time
- average height
- maximum height
- jump height
- climb height
- stability
- time upright
- turning accuracy
- target distance
- objects moved
- payload displacement
- energy consumption
- actuator work
- torque consumption
- damage
- body-part count
- body mass
- collision count
- locomotion efficiency

## Multiple Evaluation Trials

Each creature can be tested across multiple seeds/world variants to prevent overfitting to one exact terrain layout.

Options:

- mean fitness
- median fitness
- worst-case fitness
- best-case fitness
- weighted aggregation

## Generation Timeline

Almost every experiment parameter should be keyframeable across generations.

Example:

| Generation | Terrain | Objective | Mutation | Trial Duration |
|---:|---|---|---:|---:|
| 0 | Flat | Distance | 5% | 5 s |
| 100 | Slight bumps | Distance | 4% | 7 s |
| 300 | Rough | Speed + efficiency | 3% | 10 s |
| 600 | Obstacles | Speed + efficiency | 2% | 15 s |

Schedulable values should include:

- gravity
- mutation rate
- crossover rate
- terrain roughness
- motor strength limits
- fitness weights
- maximum creature complexity
- allowed body parts
- population size
- evaluation duration
- number of trials
- novelty weight
- species thresholds
- environmental difficulty

## Conditional Schedule Events

Examples:

- when best speed > 5 m/s, enable rough terrain
- if no improvement for 50 generations, double mutation rate
- if diversity < 0.15, increase novelty pressure
- after archive coverage reaches 70%, introduce obstacles
- when average body complexity exceeds a threshold, increase energy penalty

This provides visual evolutionary curriculum design without requiring user code.

---

# 6. Modern Hardware Acceleration and Parallel Execution

Creature evaluation is highly parallel and should be designed around that fact from the beginning.

## CPU Execution

- worker pool
- configurable workers/threads
- one or more simulations per worker
- SIMD where beneficial
- asynchronous job dispatch
- work stealing
- CPU affinity options

## GPU Execution

Batch large numbers of environments and creatures.

Keep as much of the pipeline on GPU as possible:

- physics integration
- controller inference
- sensor calculation
- fitness accumulation
- termination checks

Only final metrics and selected genome data need to return to CPU frequently.

## Multi-GPU

Example:

- GPU 0 evaluates batch A
- GPU 1 evaluates batch B
- GPU 2 evaluates batch C
- coordinator merges results and performs reproduction/archive updates

Support:

- automatic load balancing
- per-GPU batch size
- VRAM-aware scheduling
- heterogeneous GPU workers

## Distributed Execution

Optional LAN/cluster system:

- one coordinator
- multiple worker machines
- workers advertise CPU/GPU resources
- jobs are assigned dynamically
- results/checkpoints returned to coordinator

Possible future cloud-worker support can use the same protocol.

## Execution Modes

### Deterministic Mode

- reproducibility prioritized
- fixed ordering/timestep
- appropriate for scientific comparisons

### Maximum Throughput Mode

- asynchronous evaluation
- relaxed ordering
- maximum device utilization

## Performance Dashboard

Display live:

- physics steps/s
- creatures/s
- trials/s
- generations/hour
- total evaluations
- CPU utilization
- GPU utilization
- VRAM use
- system RAM use
- active simulations
- queue depth
- average evaluation time
- estimated generation completion

## Checkpointing and Recovery

- automatic checkpoint interval
- generation checkpoints
- crash-safe experiment metadata
- resume after power loss
- optional rolling checkpoints
- preserve RNG state where possible

No long-running experiment should be lost because the application or machine stops unexpectedly.

---

# 7. Modern UI, Experiment Control Room, Analysis, and Reproducibility

The UI should feel like modern scientific/game-development software rather than a collection of legacy settings dialogs.

## Experiment Workspace

Actions:

- New Experiment
- Load Experiment
- Clone Experiment
- Resume Checkpoint
- Import Genome
- Fork From Creature

## Creature Editor

Interactive 3D editor showing:

- starting morphology
- allowable mutations
- joint properties
- actuator properties
- sensor placement
- parameter ranges
- locked/unlocked traits

Selecting a body part should expose its editable/evolvable properties.

## Brain Editor

Visual graph/tree display for:

- neural networks
- recurrent networks
- expression trees
- CPGs
- NEAT networks
- GNN topology

Users should be able to inspect individual nodes, edges, sensor inputs, and outputs.

## World Editor

Visual placement/configuration of:

- terrain
- obstacles
- targets
- payloads
- hazards
- environmental parameters

## Evolution Panel

Configure:

- algorithm
- population size
- selection
- elitism
- species
- mutation
- crossover
- novelty
- quality-diversity archive
- random seed
- stopping criteria

## Fitness Panel

Node/expression editor for objective construction, multi-objective configuration, normalization, and trial aggregation.

## Timeline Panel

Visual generation-by-generation scheduler with:

- keyframes
- curves
- staged curricula
- conditional rules
- event markers

## Run Dashboard

Example live display:

- Generation: 387
- Evaluations: 6,450,000
- Best speed: 8.31 m/s
- Median speed: 3.18 m/s
- Species: 14
- Diversity: 0.72
- Throughput: 18,400 creatures/s

Controls:

- Watch Champion
- Watch Random Creature
- Watch Parents
- Watch Species
- Watch Archive Cell
- Pause
- Resume
- Single Generation
- Turbo
- Headless

## Evolution History / Results

Every retained creature should expose:

- creature ID
- generation
- parents
- ancestry
- fitness/objectives
- species
- behavior descriptors
- mutations
- genome
- brain
- replay
- environment seed

Analysis views should include:

- family tree
- species tree
- fitness curves
- median fitness
- diversity metrics
- morphology complexity
- population histograms
- Pareto front
- MAP-Elites heatmap
- body complexity over time
- lineage browser
- compare creatures
- compare runs

Any historical creature should support:

**Fork Experiment From This Creature**

## Self-Contained Experiment Format

Example project layout:

```text
FastWalker/
  experiment.json
  world.json
  evolution.json
  schedule.json
  checkpoints/
  genomes/
  champions/
  replays/
  metrics/
  logs/
```

Configuration should be human-readable JSON, YAML, or equivalent and fully usable without the GUI.

Example CLI:

```text
EvoLab run FastWalker.evo --headless
EvoLab run FastWalker.evo --gpu 0,1 --workers 24
```

## Architecture Principle

Keep these major systems independent:

- Evolution Engine
- Creature Genome
- Brain Engine
- Physics Backends
- World / Fitness System
- Experiment Scheduler
- Compute Scheduler
- Storage / Replay
- Modern GUI

This separation allows future evolutionary algorithms, controller types, physics backends, and accelerators to be added without rewriting the entire application.

---

# Recommended Initial Implementation Order

The first working mode should resemble classic 3DVCE closely enough to validate the entire pipeline:

1. rigid-body creature genome
2. joints and motors
3. simple expression/oscillator brain
4. distance-traveled fitness
5. classic genetic algorithm
6. flat-world evaluation
7. multithreaded CPU execution
8. champion visualization and replay

Then expand immediately toward:

- NEAT / topology evolution
- MAP-Elites / quality diversity
- richer morphology mutation
- generation scheduling
- GPU batched simulation
- multi-GPU/distributed execution
- hybrid evolution + learning

The final product should be a general-purpose virtual evolution laboratory, not merely a prettier clone of 3DVCE.
