extends Node

const EVENT_PORT_START := 47821
const EVENT_PORT_TRIES := 32

var _batch_spin: SpinBox
var _workers_spin: SpinBox
var _seconds_spin: SpinBox
var _dt_spin: SpinBox
var _live_button: Button
var _creature_button: Button
var _batch_button: Button
var _stop_button: Button
var _status_label: Label
var _progress_bar: ProgressBar
var _metrics: RichTextLabel
var _capabilities_label: Label
var _probe_mesh: MeshInstance3D
var _creature_meshes: Dictionary = {}

var _udp: PacketPeerUDP
var _event_port := 0
var _job_pid := 0
var _job_kind := ""
var _last_state_time := 0.0


func _ready() -> void:
    _build_3d_preview()
    _build_ui()

    if not _backend_exists():
        _set_status("Rust backend missing. Run BOOTSTRAP_AND_RUN.bat.")
        _set_controls_enabled(false)
        return

    if not _open_event_socket():
        _set_status("Could not open a local event port for the simulator.")
        _set_controls_enabled(false)
        return

    _load_capabilities()
    _set_status("Ready • backend connected on localhost:%d" % _event_port)


func _process(_delta: float) -> void:
    if _udp == null:
        return

    while _udp.get_available_packet_count() > 0:
        var raw := _udp.get_packet().get_string_from_utf8()
        var parsed = JSON.parse_string(raw)
        if typeof(parsed) == TYPE_DICTIONARY:
            _handle_event(parsed)

    if _job_pid > 0 and not OS.is_process_running(_job_pid):
        _job_pid = 0
        if _job_kind != "":
            _set_status("Simulator process ended.")
            _finish_job_controls()


func _exit_tree() -> void:
    _stop_current_job()
    if _udp != null:
        _udp.close()


func _backend_path() -> String:
    var exe_name := "evolab.exe" if OS.get_name() == "Windows" else "evolab"
    var project_dir := ProjectSettings.globalize_path("res://")
    var repo_root := project_dir.path_join("../..").simplify_path()
    return repo_root.path_join("target").path_join("release").path_join(exe_name)


func _backend_exists() -> bool:
    return FileAccess.file_exists(_backend_path())


func _open_event_socket() -> bool:
    _udp = PacketPeerUDP.new()

    for port in range(EVENT_PORT_START, EVENT_PORT_START + EVENT_PORT_TRIES):
        if _udp.bind(port, "127.0.0.1") == OK:
            _event_port = port
            return true

    _udp = null
    return false


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
    panel.size = Vector2(470, 800)
    layer.add_child(panel)

    var margin := MarginContainer.new()
    margin.add_theme_constant_override("margin_left", 18)
    margin.add_theme_constant_override("margin_right", 18)
    margin.add_theme_constant_override("margin_top", 16)
    margin.add_theme_constant_override("margin_bottom", 16)
    panel.add_child(margin)

    var column := VBoxContainer.new()
    column.add_theme_constant_override("separation", 8)
    margin.add_child(column)

    var title := Label.new()
    title.text = "Modern 3D Creature Evolution"
    title.add_theme_font_size_override("font_size", 24)
    column.add_child(title)

    var subtitle := Label.new()
    subtitle.text = "Step 2 • Creature Morphology"
    subtitle.modulate = Color(0.72, 0.78, 0.88)
    column.add_child(subtitle)

    column.add_child(HSeparator.new())

    _batch_spin = _add_number_row(column, "Parallel simulations", 1, 1000000, 1000, 1)
    _batch_spin.tooltip_text = "Independent physics simulations evaluated as a batch."
    _workers_spin = _add_number_row(column, "CPU workers (0 = auto)", 0, 256, 0, 1)
    _seconds_spin = _add_number_row(column, "Seconds / simulation", 0.1, 120.0, 8.0, 0.1)
    _dt_spin = _add_number_row(column, "Physics dt (seconds)", 0.0001, 0.05, 1.0 / 120.0, 0.0001)

    _creature_button = Button.new()
    _creature_button.text = "🧬 Watch 3-Segment Creature"
    _creature_button.custom_minimum_size = Vector2(0, 42)
    _creature_button.pressed.connect(_on_creature_pressed)
    column.add_child(_creature_button)

    _live_button = Button.new()
    _live_button.text = "Watch Single-Box Physics"
    _live_button.custom_minimum_size = Vector2(0, 38)
    _live_button.pressed.connect(_on_live_pressed)
    column.add_child(_live_button)

    _batch_button = Button.new()
    _batch_button.text = "Run Parallel Benchmark"
    _batch_button.custom_minimum_size = Vector2(0, 38)
    _batch_button.pressed.connect(_on_batch_pressed)
    column.add_child(_batch_button)

    _stop_button = Button.new()
    _stop_button.text = "■ Stop"
    _stop_button.custom_minimum_size = Vector2(0, 34)
    _stop_button.disabled = true
    _stop_button.pressed.connect(_on_stop_pressed)
    column.add_child(_stop_button)

    _status_label = Label.new()
    _status_label.text = "Starting..."
    _status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    column.add_child(_status_label)

    _progress_bar = ProgressBar.new()
    _progress_bar.min_value = 0
    _progress_bar.max_value = 100
    _progress_bar.value = 0
    _progress_bar.show_percentage = true
    column.add_child(_progress_bar)

    column.add_child(HSeparator.new())

    var heading := Label.new()
    heading.text = "Live state / latest result"
    heading.add_theme_font_size_override("font_size", 18)
    column.add_child(heading)

    _metrics = RichTextLabel.new()
    _metrics.bbcode_enabled = true
    _metrics.fit_content = false
    _metrics.custom_minimum_size = Vector2(0, 190)
    _metrics.text = "[color=#9aa7bd]Watch the creature or run a benchmark.[/color]"
    column.add_child(_metrics)

    _capabilities_label = Label.new()
    _capabilities_label.text = "Backend capabilities: loading..."
    _capabilities_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    _capabilities_label.modulate = Color(0.64, 0.7, 0.8)
    column.add_child(_capabilities_label)

    var footer := Label.new()
    footer.text = "The creature body and motorized joints are simulated in Rust. Godot only renders snapshots."
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


func _load_capabilities() -> void:
    var output: Array = []
    var exit_code := OS.execute(
        _backend_path(),
        PackedStringArray(["capabilities", "--json"]),
        output,
        true,
        false
    )

    if exit_code != 0 or output.is_empty():
        _capabilities_label.text = "Backend capabilities unavailable."
        return

    var parsed = JSON.parse_string(str(output[0]).strip_edges())
    if typeof(parsed) != TYPE_DICTIONARY:
        _capabilities_label.text = "Backend capabilities unavailable."
        return

    var backends: Array = parsed.get("backends", [])
    var backend: Dictionary = backends[0] if not backends.is_empty() else {}
    _capabilities_label.text = (
        "v%s • %s logical CPU threads • %s • deterministic: %s • live streaming: %s • GPU: %s"
        % [
            str(parsed.get("app_version", "?")),
            str(parsed.get("logical_cpu_threads", "?")),
            str(backend.get("name", "unknown")),
            _yes_no(backend.get("deterministic", false)),
            _yes_no(backend.get("state_streaming", false)),
            "yes" if backend.get("gpu_accelerated", false) else "not yet",
        ]
    )


func _on_creature_pressed() -> void:
    if _job_pid > 0:
        return

    _probe_mesh.visible = false
    _clear_creature_meshes()
    _progress_bar.value = 0
    _metrics.text = "[color=#9aa7bd]Starting three-segment creature...[/color]"

    var args := PackedStringArray([
        "creature-stream",
        "--event-port", str(_event_port),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--frame-hz", "60",
    ])

    if _start_job("creature", args):
        _set_status("Three-segment creature running...")


func _on_live_pressed() -> void:
    if _job_pid > 0:
        return

    _clear_creature_meshes()
    _probe_mesh.visible = true
    _reset_probe()
    _progress_bar.value = 0
    _metrics.text = "[color=#9aa7bd]Receiving live world-state snapshots...[/color]"

    var args := PackedStringArray([
        "stream",
        "--event-port", str(_event_port),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--frame-hz", "60",
    ])

    if _start_job("live", args):
        _set_status("Single-box physics running...")


func _on_batch_pressed() -> void:
    if _job_pid > 0:
        return

    _clear_creature_meshes()
    _probe_mesh.visible = true
    _reset_probe()
    _progress_bar.value = 0
    _metrics.text = "[color=#9aa7bd]Evaluating independent simulations in parallel...[/color]"

    var args := PackedStringArray([
        "probe",
        "--batch", str(int(_batch_spin.value)),
        "--workers", str(int(_workers_spin.value)),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--event-port", str(_event_port),
    ])

    if _start_job("batch", args):
        _set_status("Parallel benchmark running...")


func _start_job(kind: String, args: PackedStringArray) -> bool:
    _job_kind = kind
    _job_pid = OS.create_process(_backend_path(), args, false)

    if _job_pid <= 0:
        _job_pid = 0
        _job_kind = ""
        _set_status("Failed to start Rust simulator.")
        return false

    _set_run_buttons_disabled(true)
    _stop_button.disabled = false
    return true


func _on_stop_pressed() -> void:
    _stop_current_job()
    _set_status("Stopped.")
    _metrics.text = "[color=#9aa7bd]Simulation cancelled.[/color]"


func _stop_current_job() -> void:
    if _job_pid > 0 and OS.is_process_running(_job_pid):
        OS.kill(_job_pid)
    _job_pid = 0
    _job_kind = ""
    _finish_job_controls()


func _finish_job_controls() -> void:
    _set_run_buttons_disabled(false)
    _stop_button.disabled = true


func _set_run_buttons_disabled(disabled: bool) -> void:
    _creature_button.disabled = disabled
    _live_button.disabled = disabled
    _batch_button.disabled = disabled


func _handle_event(event: Dictionary) -> void:
    var kind := str(event.get("kind", ""))

    match kind:
        "batch_started":
            _progress_bar.value = 0
            _set_status("Benchmark started • %s simulations" % str(event.get("total", 0)))

        "batch_progress":
            var fraction := float(event.get("fraction", 0.0))
            _progress_bar.value = fraction * 100.0
            _set_status(
                "Benchmark • %s / %s simulations"
                % [str(event.get("completed", 0)), str(event.get("total", 0))]
            )

        "probe_result":
            _show_probe_result(event)
            _progress_bar.value = 100
            _job_pid = 0
            _job_kind = ""
            _finish_job_controls()

        "stream_started":
            _progress_bar.value = 0
            _last_state_time = 0.0
            _set_status("Single-box stream • %s Hz" % str(event.get("frame_hz", 60)))

        "world_state":
            _show_world_state(event.get("state", {}))

        "stream_complete":
            _progress_bar.value = 100
            _set_status(
                "Single-box simulation complete • %s s"
                % _format_float(event.get("simulated_seconds", 0.0), 3)
            )
            _job_pid = 0
            _job_kind = ""
            _finish_job_controls()

        "creature_stream_started":
            _progress_bar.value = 0
            _last_state_time = 0.0
            _probe_mesh.visible = false
            _build_creature_from_genome(event.get("genome", {}))
            _set_status(
                "%s • %s segments • %s motorized joints"
                % [
                    str(event.get("creature_name", "Creature")),
                    str(event.get("segment_count", 0)),
                    str(event.get("joint_count", 0)),
                ]
            )

        "creature_state":
            _show_creature_state(event.get("state", {}))

        "creature_stream_complete":
            _progress_bar.value = 100
            _set_status(
                "Creature simulation complete • %s s"
                % _format_float(event.get("simulated_seconds", 0.0), 3)
            )
            _job_pid = 0
            _job_kind = ""
            _finish_job_controls()


func _build_creature_from_genome(genome_value) -> void:
    _clear_creature_meshes()
    if typeof(genome_value) != TYPE_DICTIONARY:
        return

    var genome: Dictionary = genome_value
    var segments: Array = genome.get("segments", [])

    for segment_value in segments:
        if typeof(segment_value) != TYPE_DICTIONARY:
            continue

        var segment: Dictionary = segment_value
        var id_key := str(segment.get("id", 0))
        var half: Array = segment.get("half_extents", [0.25, 0.25, 0.25])

        var instance := MeshInstance3D.new()
        var box := BoxMesh.new()
        if half.size() >= 3:
            box.size = Vector3(
                float(half[0]) * 2.0,
                float(half[1]) * 2.0,
                float(half[2]) * 2.0
            )
        instance.mesh = box

        var material := StandardMaterial3D.new()
        material.albedo_color = _segment_color(int(segment.get("id", 0)))
        material.metallic = 0.08
        material.roughness = 0.45
        instance.material_override = material
        add_child(instance)
        _creature_meshes[id_key] = instance


func _clear_creature_meshes() -> void:
    for mesh_value in _creature_meshes.values():
        var mesh := mesh_value as MeshInstance3D
        if is_instance_valid(mesh):
            mesh.queue_free()
    _creature_meshes.clear()


func _show_creature_state(state_value) -> void:
    if typeof(state_value) != TYPE_DICTIONARY:
        return

    var state: Dictionary = state_value
    var bodies: Array = state.get("bodies", [])

    for body_value in bodies:
        if typeof(body_value) != TYPE_DICTIONARY:
            continue

        var body: Dictionary = body_value
        var id_key := str(body.get("id", 0))
        if not _creature_meshes.has(id_key):
            continue

        var mesh := _creature_meshes[id_key] as MeshInstance3D
        var position: Array = body.get("position", [0.0, 0.0, 0.0])
        var rotation: Array = body.get("rotation_xyzw", [0.0, 0.0, 0.0, 1.0])

        if position.size() >= 3:
            mesh.position = Vector3(
                1.8 + float(position[0]),
                float(position[1]),
                float(position[2])
            )

        if rotation.size() >= 4:
            mesh.quaternion = Quaternion(
                float(rotation[0]),
                float(rotation[1]),
                float(rotation[2]),
                float(rotation[3])
            )

    _last_state_time = float(state.get("simulated_seconds", 0.0))
    if _seconds_spin.value > 0:
        _progress_bar.value = clamp(
            _last_state_time / _seconds_spin.value * 100.0,
            0.0,
            100.0
        )

    var root_height := 0.0
    var root_speed := 0.0
    if not bodies.is_empty() and typeof(bodies[0]) == TYPE_DICTIONARY:
        var root: Dictionary = bodies[0]
        var root_pos: Array = root.get("position", [0.0, 0.0, 0.0])
        var root_vel: Array = root.get("linear_velocity", [0.0, 0.0, 0.0])
        if root_pos.size() >= 2:
            root_height = float(root_pos[1])
        if root_vel.size() >= 3:
            root_speed = Vector3(
                float(root_vel[0]),
                float(root_vel[1]),
                float(root_vel[2])
            ).length()

    _metrics.text = (
        "[table=2]"
        + "[cell]Mode[/cell][cell][b]3-segment creature[/b][/cell]"
        + "[cell]Segments[/cell][cell]%s[/cell]" % str(bodies.size())
        + "[cell]Step[/cell][cell]%s[/cell]" % str(state.get("step", 0))
        + "[cell]Simulated time[/cell][cell][b]%s s[/b][/cell]" % _format_float(_last_state_time, 3)
        + "[cell]Torso height[/cell][cell]%s m[/cell]" % _format_float(root_height, 3)
        + "[cell]Torso speed[/cell][cell]%s m/s[/cell]" % _format_float(root_speed, 3)
        + "[/table]"
    )


func _show_world_state(state_value) -> void:
    if typeof(state_value) != TYPE_DICTIONARY:
        return

    var state: Dictionary = state_value
    var position: Array = state.get("position", [0.0, 3.0, 0.0])
    var rotation: Array = state.get("rotation_xyzw", [0.0, 0.0, 0.0, 1.0])

    if position.size() >= 3:
        _probe_mesh.position = Vector3(
            1.8 + float(position[0]),
            float(position[1]),
            float(position[2])
        )

    if rotation.size() >= 4:
        _probe_mesh.quaternion = Quaternion(
            float(rotation[0]),
            float(rotation[1]),
            float(rotation[2]),
            float(rotation[3])
        )

    _last_state_time = float(state.get("simulated_seconds", 0.0))
    if _seconds_spin.value > 0:
        _progress_bar.value = clamp(
            _last_state_time / _seconds_spin.value * 100.0,
            0.0,
            100.0
        )

    var velocity: Array = state.get("linear_velocity", [0.0, 0.0, 0.0])
    var speed := 0.0
    if velocity.size() >= 3:
        speed = Vector3(
            float(velocity[0]),
            float(velocity[1]),
            float(velocity[2])
        ).length()

    _metrics.text = (
        "[table=2]"
        + "[cell]Mode[/cell][cell][b]Single-box physics[/b][/cell]"
        + "[cell]Step[/cell][cell]%s[/cell]" % str(state.get("step", 0))
        + "[cell]Simulated time[/cell][cell][b]%s s[/b][/cell]" % _format_float(_last_state_time, 3)
        + "[cell]Height[/cell][cell]%s m[/cell]" % _format_float(position[1] if position.size() >= 2 else 0.0, 3)
        + "[cell]Speed[/cell][cell]%s m/s[/cell]" % _format_float(speed, 3)
        + "[cell]Sleeping[/cell][cell]%s[/cell]" % str(state.get("sleeping", false))
        + "[/table]"
    )


func _show_probe_result(result: Dictionary) -> void:
    var position: Array = result.get("final_probe_position", [0.0, 0.35, 0.0])
    if position.size() >= 3:
        _probe_mesh.position = Vector3(
            1.8 + float(position[0]),
            float(position[1]),
            float(position[2])
        )

    _set_status("Benchmark complete • %s backend" % result.get("backend", "unknown"))

    _metrics.text = (
        "[table=2]"
        + "[cell]Simulations evaluated[/cell][cell][b]%s[/b][/cell]" % _format_int(result.get("worlds_evaluated", 0))
        + "[cell]Wall time[/cell][cell][b]%.4f s[/b][/cell]" % float(result.get("wall_seconds", 0.0))
        + "[cell]Simulation throughput[/cell][cell][b]%s / s[/b][/cell]" % _format_float(result.get("worlds_per_second", 0.0), 1)
        + "[cell]Physics throughput[/cell][cell][b]%s steps / s[/b][/cell]" % _format_float(result.get("physics_steps_per_second", 0.0), 0)
        + "[cell]Steps / simulation[/cell][cell]%s[/cell]" % _format_int(result.get("steps_per_world", 0))
        + "[cell]Physics dt[/cell][cell]%.8f s[/cell]" % float(result.get("physics_dt_seconds", 0.0))
        + "[cell]Simulated time[/cell][cell]%.3f s[/cell]" % float(result.get("simulated_seconds_per_world", 0.0))
        + "[cell]Deterministic[/cell][cell]%s[/cell]" % str(result.get("deterministic", false))
        + "[/table]"
    )


func _segment_color(id: int) -> Color:
    match id:
        0:
            return Color(0.25, 0.78, 1.0)
        1:
            return Color(0.95, 0.55, 0.22)
        2:
            return Color(0.45, 0.92, 0.42)
        _:
            return Color(0.75, 0.75, 0.85)


func _reset_probe() -> void:
    _probe_mesh.position = Vector3(1.8, 3.0, 0.0)
    _probe_mesh.quaternion = Quaternion.IDENTITY


func _set_controls_enabled(enabled: bool) -> void:
    if _creature_button != null:
        _creature_button.disabled = not enabled
    if _live_button != null:
        _live_button.disabled = not enabled
    if _batch_button != null:
        _batch_button.disabled = not enabled


func _set_status(text: String) -> void:
    if _status_label != null:
        _status_label.text = text


func _yes_no(value) -> String:
    return "yes" if bool(value) else "no"


func _format_int(value) -> String:
    return "%d" % int(value)


func _format_float(value, decimals: int) -> String:
    var pattern := "%." + str(decimals) + "f"
    return pattern % float(value)
