# Godot frontend

The Step 1 GUI is now functional.

It provides:

- a modern desktop window
- settings for batch size, CPU worker count, simulation duration, and fixed timestep
- a basic 3D probe preview
- asynchronous execution so the UI stays responsive
- machine-readable JSON communication with the Rust `evolab` backend
- live result display for worlds/s, physics steps/s, wall time, and simulation settings

The Rust simulation remains completely independent of Godot. Godot launches the
compiled backend as a child process and consumes its versioned JSON result.

This is intentionally a simple IPC boundary for Step 1. It can later be upgraded
to a persistent local IPC server/FFI layer without putting physics into the UI.
