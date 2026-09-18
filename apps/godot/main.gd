extends Node

var _batch_spin: SpinBox
var _workers_spin: SpinBox
var _seconds_spin: SpinBox
var _dt_spin: SpinBox
var _run_button: Button
var _status_label: Label
var _metrics: RichTextLabel
var _probe_mesh: MeshInstance3D
var _thread: Thread
var _running := false


func _ready() -> void:
    _build_3d_preview()
    _build_ui()
    _set_status("Ready. Rust backend found: %s" % _backend_exists())


func _exit_tree() -> void:
    if _thread != null and _thread.is_started():
        _thread.wait_to_finish()


func _backend_path() -> String:
    var exe_name := "evolab.exe" if OS.get_name() == "Windows" else "evolab"
    return ProjectSettings.globalize_path("res://../../target/release/%s" % exe_name)


func _backend_exists() -> bool:
    return FileAccess.file_exists(_backend_path())


func _build_3d_preview() -> void:
    var world := WorldEnvironment.new()
    var environment := Environment.new()
    environment.background_mode = Environment.BG_COLOR
    environment.background_color = Color(0.035, 0.04, 0.055)
    environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
    environment.ambient_light_color = Color(0.65, 0.7, 0.8)
    environment.ambient_light_energy = 0.8
    world.environment = environment
    add_child(world)

    var light := DirectionalLight3D.new()
    light.rotation_degrees = Vector3(-55.0, -35.0, 0.0)
    light.light_energy = 1.4
    light.shadow_enabled = true
    add_child(light)

    var ground := MeshInstance3D.new()
    var ground_mesh := BoxMesh.new()
    ground_mesh.size = Vector3(12.0, 0.2, 12.0)
    ground.mesh = ground_mesh
    ground.position = Vector3(1.8, 0.0, 0.0)
    var ground_material := StandardMaterial3D.new()
    ground_material.albedo_color = Color(0.15, 0.18, 0.22)
    ground.material_override = ground_material
    add_child(ground)

    _probe_mesh = MeshInstance3D.new()
    var probe_box := BoxMesh.new()
    probe_box.size = Vector3(0.5, 0.5, 0.5)
    _probe_mesh.mesh = probe_box
    _probe_mesh.position = Vector3(1.8, 3.0, 0.0)
    var probe_material := StandardMaterial3D.new()
    probe_material.albedo_color = Color(0.25, 0.75, 1.0)
    probe_material.metallic = 0.15
    probe_material.roughness = 0.35
    _probe_mesh.material_override = probe_material
    add_child(_probe_mesh)

    var camera := Camera3D.new()
    camera.position = Vector3(8.5, 5.5, 8.5)
    add_child(camera)
    camera.look_at(Vector3(1.8, 1.0, 0.0), Vector3.UP)


func _build_ui() -> void:
    var layer := CanvasLayer.new()
    add_child(layer)

    var panel := PanelContainer.new()
    panel.position = Vector2(18, 18)
    panel.size = Vector2(430, 724)
    layer.add_child(panel)

    var margin := MarginContainer.new()
    margin.add_theme_constant_override("margin_left", 18)
    margin.add_theme_constant_override("margin_right", 18)
    margin.add_theme_constant_override("margin_top", 16)
    margin.add_theme_constant_override("margin_bottom", 16)
    panel.add_child(margin)

    var column := VBoxContainer.new()
    column.add_theme_constant_override("separation", 10)
    margin.add_child(column)

    var title := Label.new()
    title.text = "Modern 3D Creature Evolution"
    title.add_theme_font_size_override("font_size", 24)
    column.add_child(title)

    var subtitle := Label.new()
    subtitle.text = "Step 1 • Simulation Foundation"
    subtitle.modulate = Color(0.72, 0.78, 0.88)
    column.add_child(subtitle)

    column.add_child(HSeparator.new())

    _batch_spin = _add_number_row(column, "Worlds", 1, 1000000, 1000, 1)
    _workers_spin = _add_number_row(column, "CPU workers (0 = auto)", 0, 256, 0, 1)
    _seconds_spin = _add_number_row(column, "Seconds / world", 0.1, 120.0, 5.0, 0.1)
    _dt_spin = _add_number_row(column, "Physics dt (seconds)", 0.0001, 0.05, 1.0 / 120.0, 0.0001)
    _dt_spin.custom_arrow_step = 0.0001

    _run_button = Button.new()
    _run_button.text = "Run Parallel Physics Test"
    _run_button.custom_minimum_size = Vector2(0, 42)
    _run_button.pressed.connect(_on_run_pressed)
    column.add_child(_run_button)

    _status_label = Label.new()
    _status_label.text = "Ready"
    _status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    column.add_child(_status_label)

    column.add_child(HSeparator.new())

    var heading := Label.new()
    heading.text = "Latest result"
    heading.add_theme_font_size_override("font_size", 18)
    column.add_child(heading)

    _metrics = RichTextLabel.new()
    _metrics.bbcode_enabled = true
    _metrics.fit_content = false
    _metrics.custom_minimum_size = Vector2(0, 300)
    _metrics.text = "[color=#9aa7bd]Run a test to populate metrics.[/color]"
    column.add_child(_metrics)

    var footer := Label.new()
    footer.text = "The 3D box is the Step 1 probe. Simulation runs in Rust; Godot is only the UI/viewer."
    footer.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    footer.modulate = Color(0.64, 0.7, 0.8)
    column.add_child(footer)


func _add_number_row(
    parent: VBoxContainer,
    label_text: String,
    min_value: float,
    max_value: float,
    default_value: float,
    step_value: float
) -> SpinBox:
    var row := HBoxContainer.new()
    parent.add_child(row)

    var label := Label.new()
    label.text = label_text
    label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    row.add_child(label)

    var spin := SpinBox.new()
    spin.min_value = min_value
    spin.max_value = max_value
    spin.step = step_value
    spin.value = default_value
    spin.custom_minimum_size = Vector2(145, 32)
    row.add_child(spin)
    return spin


func _on_run_pressed() -> void:
    if _running:
        return

    if not _backend_exists():
        _set_status("Rust backend is missing. Run BOOTSTRAP_AND_RUN.bat to build it.")
        return

    _running = true
    _run_button.disabled = true
    _run_button.text = "Running..."
    _probe_mesh.position = Vector3(1.8, 3.0, 0.0)
    _metrics.text = "[color=#9aa7bd]Evaluating independent physics worlds...[/color]"
    _set_status("Running Rust simulation backend...")

    var request := {
        "batch": int(_batch_spin.value),
        "workers": int(_workers_spin.value),
        "seconds": float(_seconds_spin.value),
        "dt": float(_dt_spin.value),
    }

    _thread = Thread.new()
    _thread.start(_run_probe_worker.bind(request))


func _run_probe_worker(request: Dictionary) -> void:
    var output: Array = []
    var args := PackedStringArray([
        "probe",
        "--batch", str(request.batch),
        "--workers", str(request.workers),
        "--seconds", str(request.seconds),
        "--dt", str(request.dt),
        "--json",
    ])

    var exit_code := OS.execute(_backend_path(), args, output, true, false)
    var raw := ""
    for line in output:
        raw += str(line)

    call_deferred("_probe_finished", exit_code, raw)


func _probe_finished(exit_code: int, raw: String) -> void:
    if _thread != null:
        _thread.wait_to_finish()
        _thread = null

    _running = false
    _run_button.disabled = false
    _run_button.text = "Run Parallel Physics Test"

    if exit_code != 0:
        _set_status("Backend failed with exit code %d" % exit_code)
        _metrics.text = "[color=#ff7b7b]%s[/color]" % raw.strip_edges()
        return

    var parsed = JSON.parse_string(raw.strip_edges())
    if typeof(parsed) != TYPE_DICTIONARY:
        _set_status("Backend returned invalid JSON.")
        _metrics.text = "[color=#ff7b7b]%s[/color]" % raw.strip_edges()
        return

    var result: Dictionary = parsed
    var position: Array = result.get("final_probe_position", [0.0, 0.35, 0.0])
    if position.size() >= 3:
        _probe_mesh.position = Vector3(1.8 + float(position[0]), float(position[1]), float(position[2]))

    _set_status("Complete • %s backend" % result.get("backend", "unknown"))

    _metrics.text = (
        "[table=2]"
        + "[cell]Worlds evaluated[/cell][cell][b]%s[/b][/cell]" % _format_int(result.get("worlds_evaluated", 0))
        + "[cell]Wall time[/cell][cell][b]%.4f s[/b][/cell]" % float(result.get("wall_seconds", 0.0))
        + "[cell]World throughput[/cell][cell][b]%s / s[/b][/cell]" % _format_float(result.get("worlds_per_second", 0.0), 1)
        + "[cell]Physics throughput[/cell][cell][b]%s steps / s[/b][/cell]" % _format_float(result.get("physics_steps_per_second", 0.0), 0)
        + "[cell]Steps / world[/cell][cell]%s[/cell]" % _format_int(result.get("steps_per_world", 0))
        + "[cell]Physics dt[/cell][cell]%.8f s[/cell]" % float(result.get("physics_dt_seconds", 0.0))
        + "[cell]Simulated time / world[/cell][cell]%.3f s[/cell]" % float(result.get("simulated_seconds_per_world", 0.0))
        + "[cell]Deterministic[/cell][cell]%s[/cell]" % str(result.get("deterministic", false))
        + "[/table]"
    )


func _set_status(text: String) -> void:
    if _status_label != null:
        _status_label.text = text


func _format_int(value) -> String:
    return "%d" % int(value)


func _format_float(value, decimals: int) -> String:
    var pattern := "%." + str(decimals) + "f"
    return pattern % float(value)
