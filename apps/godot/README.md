# Godot frontend

This directory is reserved for the modern desktop UI and 3D viewer.

Step 1 intentionally keeps physics and evaluation in `evolab-core` so the UI
never owns the simulation loop. The frontend will eventually communicate with
the core through a stable API/FFI boundary and will be optional for headless
runs.
