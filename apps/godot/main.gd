extends Node

const EVENT_PORT_START := 47821
const EVENT_PORT_TRIES := 32

var _batch_spin: SpinBox
var _workers_spin: SpinBox
var _seconds_spin: SpinBox
var _dt_spin: SpinBox
var _seed_spin: SpinBox
var _mutation_spin: SpinBox
var _random_segments_spin: SpinBox
var _max_segments_spin: SpinBox
var _population_spin: SpinBox
var _generations_spin: SpinBox
var _tournament_spin: SpinBox
var _elite_spin: SpinBox
var _evolution_mutations_spin: SpinBox

var _seed_creature_button: Button
var _mutate_button: Button
var _random_button: Button
var _save_button: Button
var _load_button: Button
var _live_button: Button
var _batch_button: Button
var _evolve_button: Button
var _watch_champion_button: Button
var _stop_button: Button

var _status_label: Label
var _progress_bar: ProgressBar
var _metrics: RichTextLabel
var _capabilities_label: Label

var _probe_mesh: MeshInstance3D
var _creature_meshes: Dictionary = {}
var _current_genome: Dictionary = {}
var _current_genome_source := ""
var _current_mutation_count := 0
var _has_evolution_champion := false

var _save_dialog: FileDialog
var _load_dialog: FileDialog

var _udp: PacketPeerUDP
var _event_port := 0
var _job_pid := 0
var _job_kind := ""
var _last_state_time := 0.0


func _ready() -> void:
    _build_3d_preview()
    _build_ui()
    _build_file_dialogs()

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
    ground_mesh.size = Vector3(14.0, 0.2, 14.0)
    ground.mesh = ground_mesh
    ground.position = Vector3(2.2, 0.0, 0.0)
    var ground_material := StandardMaterial3D.new()
    ground_material.albedo_color = Color(0.15, 0.18, 0.22)
    ground.material_override = ground_material
    add_child(ground)

    _probe_mesh = MeshInstance3D.new()
    var probe_box := BoxMesh.new()
    probe_box.size = Vector3(0.5, 0.5, 0.5)
    _probe_mesh.mesh = probe_box
    _probe_mesh.position = Vector3(2.2, 3.0, 0.0)
    var probe_material := StandardMaterial3D.new()
    probe_material.albedo_color = Color(0.25, 0.75, 1.0)
    probe_material.metallic = 0.15
    probe_material.roughness = 0.35
    _probe_mesh.material_override = probe_material
    add_child(_probe_mesh)

    var camera := Camera3D.new()
    camera.position = Vector3(9.2, 5.8, 9.2)
    add_child(camera)
    camera.look_at(Vector3(2.2, 1.2, 0.0), Vector3.UP)


func _build_ui() -> void:
    var layer := CanvasLayer.new()
    add_child(layer)

    var panel := PanelContainer.new()
    panel.position = Vector2(18, 18)
    panel.size = Vector2(510, 884)
    layer.add_child(panel)

    var margin := MarginContainer.new()
    margin.add_theme_constant_override("margin_left", 18)
    margin.add_theme_constant_override("margin_right", 18)
    margin.add_theme_constant_override("margin_top", 14)
    margin.add_theme_constant_override("margin_bottom", 14)
    var scroll := ScrollContainer.new()
    scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
    scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
    panel.add_child(scroll)

    margin.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    scroll.add_child(margin)

    var column := VBoxContainer.new()
    column.add_theme_constant_override("separation", 6)
    margin.add_child(column)

    var title := Label.new()
    title.text = "Modern 3D Creature Evolution"
    title.add_theme_font_size_override("font_size", 24)
    column.add_child(title)

    var subtitle := Label.new()
    subtitle.text = "Step 3 • Evolvable Expression-Tree Brains"
    subtitle.modulate = Color(0.72, 0.78, 0.88)
    column.add_child(subtitle)

    column.add_child(HSeparator.new())

    var editor_heading := Label.new()
    editor_heading.text = "Creature generation"
    editor_heading.add_theme_font_size_override("font_size", 17)
    column.add_child(editor_heading)

    _seed_spin = _add_number_row(column, "Seed", 1, 999999999, 1, 1)
    _mutation_spin = _add_number_row(column, "Mutation operations", 0, 500, 12, 1)
    _random_segments_spin = _add_number_row(column, "Random creature segments", 2, 40, 5, 1)
    _max_segments_spin = _add_number_row(column, "Maximum segments", 2, 40, 12, 1)

    var creature_row := HBoxContainer.new()
    creature_row.add_theme_constant_override("separation", 6)
    column.add_child(creature_row)

    _seed_creature_button = Button.new()
    _seed_creature_button.text = "Seed"
    _seed_creature_button.tooltip_text = "Run the known three-segment seed creature."
    _seed_creature_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _seed_creature_button.pressed.connect(_on_seed_creature_pressed)
    creature_row.add_child(_seed_creature_button)

    _mutate_button = Button.new()
    _mutate_button.text = "Mutate Current"
    _mutate_button.tooltip_text = "Mutate the current creature. If none exists, mutate the seed creature."
    _mutate_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _mutate_button.pressed.connect(_on_mutate_pressed)
    creature_row.add_child(_mutate_button)

    _random_button = Button.new()
    _random_button.text = "Random"
    _random_button.tooltip_text = "Generate a new random morphology using the selected seed."
    _random_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _random_button.pressed.connect(_on_random_pressed)
    creature_row.add_child(_random_button)

    var file_row := HBoxContainer.new()
    file_row.add_theme_constant_override("separation", 6)
    column.add_child(file_row)

    _save_button = Button.new()
    _save_button.text = "Save Genome..."
    _save_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _save_button.disabled = true
    _save_button.pressed.connect(_on_save_pressed)
    file_row.add_child(_save_button)

    _load_button = Button.new()
    _load_button.text = "Load Genome..."
    _load_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _load_button.pressed.connect(_on_load_pressed)
    file_row.add_child(_load_button)

    column.add_child(HSeparator.new())

    var evolution_heading := Label.new()
    evolution_heading.text = "Evolution"
    evolution_heading.add_theme_font_size_override("font_size", 17)
    column.add_child(evolution_heading)

    _population_spin = _add_number_row(column, "Population", 2, 10000, 50, 1)
    _generations_spin = _add_number_row(column, "Generations", 1, 10000, 100, 1)
    _tournament_spin = _add_number_row(column, "Tournament size", 1, 1000, 7, 1)
    _elite_spin = _add_number_row(column, "Elite kept", 1, 999, 2, 1)
    _evolution_mutations_spin = _add_number_row(column, "Mutations / child", 1, 500, 8, 1)

    var evolution_row := HBoxContainer.new()
    evolution_row.add_theme_constant_override("separation", 6)
    column.add_child(evolution_row)

    _evolve_button = Button.new()
    _evolve_button.text = "🧬 Start Evolution"
    _evolve_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _evolve_button.pressed.connect(_on_evolve_pressed)
    evolution_row.add_child(_evolve_button)

    _watch_champion_button = Button.new()
    _watch_champion_button.text = "Watch Champion"
    _watch_champion_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _watch_champion_button.disabled = true
    _watch_champion_button.pressed.connect(_on_watch_champion_pressed)
    evolution_row.add_child(_watch_champion_button)

    column.add_child(HSeparator.new())

    var sim_heading := Label.new()
    sim_heading.text = "Simulation / benchmark"
    sim_heading.add_theme_font_size_override("font_size", 17)
    column.add_child(sim_heading)

    _seconds_spin = _add_number_row(column, "Seconds / simulation", 0.1, 120.0, 8.0, 0.1)
    _dt_spin = _add_number_row(column, "Physics dt (seconds)", 0.0001, 0.05, 1.0 / 120.0, 0.0001)
    _batch_spin = _add_number_row(column, "Parallel simulations", 1, 1000000, 1000, 1)
    _workers_spin = _add_number_row(column, "CPU workers (0 = auto)", 0, 256, 0, 1)

    var utility_row := HBoxContainer.new()
    utility_row.add_theme_constant_override("separation", 6)
    column.add_child(utility_row)

    _live_button = Button.new()
    _live_button.text = "Single Box"
    _live_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _live_button.pressed.connect(_on_live_pressed)
    utility_row.add_child(_live_button)

    _batch_button = Button.new()
    _batch_button.text = "Benchmark"
    _batch_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _batch_button.pressed.connect(_on_batch_pressed)
    utility_row.add_child(_batch_button)

    _stop_button = Button.new()
    _stop_button.text = "■ Stop"
    _stop_button.custom_minimum_size = Vector2(0, 32)
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

    _metrics = RichTextLabel.new()
    _metrics.bbcode_enabled = true
    _metrics.fit_content = false
    _metrics.custom_minimum_size = Vector2(0, 155)
    _metrics.text = "[color=#9aa7bd]Generate, mutate, load, or watch a creature.[/color]"
    column.add_child(_metrics)

    _capabilities_label = Label.new()
    _capabilities_label.text = "Backend capabilities: loading..."
    _capabilities_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    _capabilities_label.modulate = Color(0.64, 0.7, 0.8)
    column.add_child(_capabilities_label)


func _build_file_dialogs() -> void:
    _save_dialog = FileDialog.new()
    _save_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
    _save_dialog.filters = PackedStringArray(["*.json ; Creature Genome JSON"])
    _save_dialog.current_file = "creature_genome.json"
    _save_dialog.file_selected.connect(_on_save_file_selected)
    add_child(_save_dialog)

    _load_dialog = FileDialog.new()
    _load_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _load_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
    _load_dialog.filters = PackedStringArray(["*.json ; Creature Genome JSON"])
    _load_dialog.file_selected.connect(_on_load_file_selected)
    add_child(_load_dialog)


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
    spin.custom_minimum_size = Vector2(145, 29)
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
        "v%s • %s logical CPU threads • %s • deterministic: %s • streaming: %s • GPU: %s"
        % [
            str(parsed.get("app_version", "?")),
            str(parsed.get("logical_cpu_threads", "?")),
            str(backend.get("name", "unknown")),
            _yes_no(backend.get("deterministic", false)),
            _yes_no(backend.get("state_streaming", false)),
            "yes" if backend.get("gpu_accelerated", false) else "not yet",
        ]
    )


func _base_creature_args() -> PackedStringArray:
    return PackedStringArray([
        "creature-stream",
        "--event-port", str(_event_port),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--frame-hz", "60",
        "--seed", str(int(_seed_spin.value)),
        "--max-segments", str(int(_max_segments_spin.value)),
    ])


func _prepare_creature_run() -> void:
    _probe_mesh.visible = false
    _clear_creature_meshes()
    _progress_bar.value = 0
    _metrics.text = "[color=#9aa7bd]Starting creature simulation...[/color]"


func _on_seed_creature_pressed() -> void:
    if _job_pid > 0:
        return

    _prepare_creature_run()
    var args := _base_creature_args()
    if _start_job("creature", args):
        _set_status("Running three-segment seed creature...")


func _on_mutate_pressed() -> void:
    if _job_pid > 0:
        return

    _prepare_creature_run()
    var args := _base_creature_args()
    args.append_array(PackedStringArray([
        "--mutations", str(int(_mutation_spin.value)),
    ]))

    if not _current_genome.is_empty():
        var temp_path := ProjectSettings.globalize_path("user://mutation_parent.json")
        if not _write_genome_file(temp_path, _current_genome):
            _set_status("Could not prepare current genome for mutation.")
            return
        args.append_array(PackedStringArray(["--genome", temp_path]))

    if _start_job("creature", args):
        _set_status("Mutating creature and running it...")


func _on_random_pressed() -> void:
    if _job_pid > 0:
        return

    _prepare_creature_run()
    var args := _base_creature_args()
    args.append_array(PackedStringArray([
        "--random-segments", str(int(_random_segments_spin.value)),
        "--mutations", str(int(_mutation_spin.value)),
    ]))

    if _start_job("creature", args):
        _set_status("Generating random creature and running it...")


func _on_evolve_pressed() -> void:
    if _job_pid > 0:
        return

    if int(_tournament_spin.value) > int(_population_spin.value):
        _set_status("Tournament size cannot exceed population.")
        return
    if int(_elite_spin.value) >= int(_population_spin.value):
        _set_status("Elite kept must be smaller than population.")
        return

    _probe_mesh.visible = false
    _progress_bar.value = 0
    _has_evolution_champion = false
    _watch_champion_button.disabled = true
    _metrics.text = "[color=#9aa7bd]Creating and evaluating generation 1...[/color]"

    var args := PackedStringArray([
        "evolve",
        "--population", str(int(_population_spin.value)),
        "--generations", str(int(_generations_spin.value)),
        "--tournament", str(int(_tournament_spin.value)),
        "--elite", str(int(_elite_spin.value)),
        "--mutations", str(int(_evolution_mutations_spin.value)),
        "--max-segments", str(int(_max_segments_spin.value)),
        "--seed", str(int(_seed_spin.value)),
        "--workers", str(int(_workers_spin.value)),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--event-port", str(_event_port),
    ])

    if not _current_genome.is_empty():
        var parent_path := ProjectSettings.globalize_path("user://evolution_parent.json")
        if not _write_genome_file(parent_path, _current_genome):
            _set_status("Could not write evolution parent genome.")
            return
        args.append_array(PackedStringArray(["--genome", parent_path]))

    if _start_job("evolution", args):
        _set_status("Evolution running • fitness = horizontal distance traveled")


func _on_watch_champion_pressed() -> void:
    if _job_pid > 0 or _current_genome.is_empty():
        return

    var champion_path := ProjectSettings.globalize_path("user://evolution_champion.json")
    if not _write_genome_file(champion_path, _current_genome):
        _set_status("Could not prepare champion genome.")
        return

    _prepare_creature_run()
    var args := _base_creature_args()
    args.append_array(PackedStringArray(["--genome", champion_path]))

    if _start_job("creature", args):
        _set_status("Watching evolution champion in real time...")


func _on_save_pressed() -> void:
    if _current_genome.is_empty():
        _set_status("There is no creature genome to save yet.")
        return
    _save_dialog.popup_centered_ratio(0.70)


func _on_load_pressed() -> void:
    if _job_pid > 0:
        return
    _load_dialog.popup_centered_ratio(0.70)


func _on_save_file_selected(path: String) -> void:
    if _write_genome_file(path, _current_genome):
        _set_status("Saved genome: %s" % path)
    else:
        _set_status("Failed to save genome: %s" % path)


func _on_load_file_selected(path: String) -> void:
    var file := FileAccess.open(path, FileAccess.READ)
    if file == null:
        _set_status("Could not open genome: %s" % path)
        return

    var parsed = JSON.parse_string(file.get_as_text())
    file.close()

    if typeof(parsed) != TYPE_DICTIONARY:
        _set_status("Genome file is not valid JSON.")
        return

    _current_genome = parsed
    _save_button.disabled = false
    _prepare_creature_run()

    var args := _base_creature_args()
    args.append_array(PackedStringArray(["--genome", path]))

    if _start_job("creature", args):
        _set_status("Loaded genome. Rust is validating and simulating it...")


func _write_genome_file(path: String, genome: Dictionary) -> bool:
    var file := FileAccess.open(path, FileAccess.WRITE)
    if file == null:
        return false
    file.store_string(JSON.stringify(genome, "\t"))
    file.close()
    return true


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
    _save_button.disabled = _current_genome.is_empty()


func _set_run_buttons_disabled(disabled: bool) -> void:
    _seed_creature_button.disabled = disabled
    _mutate_button.disabled = disabled
    _random_button.disabled = disabled
    _load_button.disabled = disabled
    _live_button.disabled = disabled
    _batch_button.disabled = disabled
    _evolve_button.disabled = disabled
    _watch_champion_button.disabled = disabled or not _has_evolution_champion
    _save_button.disabled = disabled or _current_genome.is_empty()


func _handle_event(event: Dictionary) -> void:
    var kind := str(event.get("kind", ""))

    match kind:
        "evolution_started":
            _progress_bar.value = 0
            _set_status(
                "Evolution started • %s creatures × %s generations"
                % [str(event.get("population", 0)), str(event.get("generations", 0))]
            )

        "generation_complete":
            var generation := int(event.get("generation", 0))
            var generations := int(event.get("generations", 1))
            _progress_bar.value = float(event.get("fraction", 0.0)) * 100.0

            var champion_value = event.get("champion", {})
            if typeof(champion_value) == TYPE_DICTIONARY:
                _current_genome = champion_value
                _current_genome_source = "generation %d champion" % generation
                _probe_mesh.visible = false
                _build_creature_from_genome(_current_genome)

            _set_status(
                "Generation %d / %d • best %.4f m • average %.4f m"
                % [
                    generation,
                    generations,
                    float(event.get("best_fitness", 0.0)),
                    float(event.get("average_fitness", 0.0)),
                ]
            )
            _metrics.text = (
                "[table=2]"
                + "[cell]Generation[/cell][cell][b]%d / %d[/b][/cell]" % [generation, generations]
                + "[cell]Best distance[/cell][cell][b]%.4f m[/b][/cell]" % float(event.get("best_distance", 0.0))
                + "[cell]Average distance[/cell][cell]%.4f m[/cell]" % float(event.get("average_fitness", 0.0))
                + "[cell]Worst distance[/cell][cell]%.4f m[/cell]" % float(event.get("worst_fitness", 0.0))
                + "[cell]Champion segments[/cell][cell]%s[/cell]" % str(event.get("best_segments", 0))
                + "[cell]Brain nodes[/cell][cell]%s[/cell]" % str(event.get("best_brain_nodes", 0))
                + "[cell]Evaluations[/cell][cell]%s[/cell]" % str(event.get("evaluations_completed", 0))
                + "[/table]"
            )

        "evolution_complete":
            var final_champion = event.get("champion", {})
            if typeof(final_champion) == TYPE_DICTIONARY:
                _current_genome = final_champion
                _current_genome_source = "evolution champion"
                _build_creature_from_genome(_current_genome)

            _has_evolution_champion = true
            _progress_bar.value = 100
            _set_status(
                "Evolution complete • champion %.4f m • %s evaluations"
                % [
                    float(event.get("champion_fitness", 0.0)),
                    str(event.get("evaluations_completed", 0)),
                ]
            )
            _metrics.text = (
                "[table=2]"
                + "[cell]Champion distance[/cell][cell][b]%.4f m[/b][/cell]" % float(event.get("champion_distance", 0.0))
                + "[cell]Generations[/cell][cell]%s[/cell]" % str(event.get("generations_completed", 0))
                + "[cell]Evaluations[/cell][cell]%s[/cell]" % str(event.get("evaluations_completed", 0))
                + "[cell]Wall time[/cell][cell]%.3f s[/cell]" % float(event.get("wall_seconds", 0.0))
                + "[cell]Next[/cell][cell]Click Watch Champion[/cell]"
                + "[/table]"
            )
            _job_pid = 0
            _job_kind = ""
            _finish_job_controls()

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

            var genome_value = event.get("genome", {})
            if typeof(genome_value) == TYPE_DICTIONARY:
                _current_genome = genome_value
                _save_button.disabled = false

            _current_genome_source = str(event.get("genome_source", "unknown"))
            var mutation_log: Array = event.get("mutation_log", [])
            _current_mutation_count = mutation_log.size()

            _build_creature_from_genome(_current_genome)
            _set_status(
                "%s • %s segments • %s joints • %s brain nodes • %s mutations"
                % [
                    str(event.get("creature_name", "Creature")),
                    str(event.get("segment_count", 0)),
                    str(event.get("joint_count", 0)),
                    str(event.get("brain_nodes", 0)),
                    str(_current_mutation_count),
                ]
            )

        "creature_state":
            _show_creature_state(event.get("state", {}))

        "creature_stream_complete":
            _progress_bar.value = 100
            _set_status(
                "Creature simulation complete • %s s • genome ready to save/mutate"
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

        var initial: Array = segment.get("initial_position", [0.0, 0.0, 0.0])
        if initial.size() >= 3:
            instance.position = Vector3(
                2.2 + float(initial[0]),
                float(initial[1]),
                float(initial[2])
            )

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
                2.2 + float(position[0]),
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
        + "[cell]Genome[/cell][cell][b]%s[/b][/cell]" % _current_genome_source
        + "[cell]Segments[/cell][cell]%s[/cell]" % str(bodies.size())
        + "[cell]Mutations[/cell][cell]%s[/cell]" % str(_current_mutation_count)
        + "[cell]Brain nodes[/cell][cell]%s[/cell]" % str(_current_brain_node_count())
        + "[cell]Step[/cell][cell]%s[/cell]" % str(state.get("step", 0))
        + "[cell]Time[/cell][cell][b]%s s[/b][/cell]" % _format_float(_last_state_time, 3)
        + "[cell]Root height[/cell][cell]%s m[/cell]" % _format_float(root_height, 3)
        + "[cell]Root speed[/cell][cell]%s m/s[/cell]" % _format_float(root_speed, 3)
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
            2.2 + float(position[0]),
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
        + "[cell]Time[/cell][cell][b]%s s[/b][/cell]" % _format_float(_last_state_time, 3)
        + "[cell]Height[/cell][cell]%s m[/cell]" % _format_float(position[1] if position.size() >= 2 else 0.0, 3)
        + "[cell]Speed[/cell][cell]%s m/s[/cell]" % _format_float(speed, 3)
        + "[/table]"
    )


func _show_probe_result(result: Dictionary) -> void:
    var position: Array = result.get("final_probe_position", [0.0, 0.35, 0.0])
    if position.size() >= 3:
        _probe_mesh.position = Vector3(
            2.2 + float(position[0]),
            float(position[1]),
            float(position[2])
        )

    _set_status("Benchmark complete • %s backend" % result.get("backend", "unknown"))

    _metrics.text = (
        "[table=2]"
        + "[cell]Simulations[/cell][cell][b]%s[/b][/cell]" % _format_int(result.get("worlds_evaluated", 0))
        + "[cell]Wall time[/cell][cell][b]%.4f s[/b][/cell]" % float(result.get("wall_seconds", 0.0))
        + "[cell]Throughput[/cell][cell][b]%s / s[/b][/cell]" % _format_float(result.get("worlds_per_second", 0.0), 1)
        + "[cell]Physics[/cell][cell][b]%s steps / s[/b][/cell]" % _format_float(result.get("physics_steps_per_second", 0.0), 0)
        + "[cell]Steps / simulation[/cell][cell]%s[/cell]" % _format_int(result.get("steps_per_world", 0))
        + "[/table]"
    )


func _current_brain_node_count() -> int:
    if _current_genome.is_empty():
        return 0

    var brain_value = _current_genome.get("brain", {})
    if typeof(brain_value) != TYPE_DICTIONARY:
        return 0

    var brain: Dictionary = brain_value
    var outputs: Array = brain.get("outputs", [])
    var total := 0
    for output_value in outputs:
        if typeof(output_value) != TYPE_DICTIONARY:
            continue
        var output: Dictionary = output_value
        total += _expression_node_count(output.get("expression", null))
    return total


func _expression_node_count(expression_value) -> int:
    if expression_value == null:
        return 0
    if typeof(expression_value) != TYPE_DICTIONARY:
        return 1

    var expression: Dictionary = expression_value
    if expression.has("Constant") or expression.has("Sensor"):
        return 1

    for key in ["Negate", "Sin", "Cos"]:
        if expression.has(key):
            return 1 + _expression_node_count(expression[key])

    for key in ["Add", "Subtract", "Multiply"]:
        if expression.has(key):
            var pair = expression[key]
            if typeof(pair) == TYPE_ARRAY and pair.size() >= 2:
                return 1 + _expression_node_count(pair[0]) + _expression_node_count(pair[1])

    if expression.has("Clamp"):
        var clamp_value = expression["Clamp"]
        if typeof(clamp_value) == TYPE_DICTIONARY:
            return 1 + _expression_node_count(clamp_value.get("value", null))

    return 1


func _segment_color(id: int) -> Color:
    var hue := fmod(float(id) * 0.173 + 0.54, 1.0)
    return Color.from_hsv(hue, 0.62, 0.95)


func _reset_probe() -> void:
    _probe_mesh.position = Vector3(2.2, 3.0, 0.0)
    _probe_mesh.quaternion = Quaternion.IDENTITY


func _set_controls_enabled(enabled: bool) -> void:
    if _seed_creature_button != null:
        _seed_creature_button.disabled = not enabled
    if _mutate_button != null:
        _mutate_button.disabled = not enabled
    if _random_button != null:
        _random_button.disabled = not enabled
    if _load_button != null:
        _load_button.disabled = not enabled
    if _live_button != null:
        _live_button.disabled = not enabled
    if _batch_button != null:
        _batch_button.disabled = not enabled
    if _evolve_button != null:
        _evolve_button.disabled = not enabled
    if _watch_champion_button != null:
        _watch_champion_button.disabled = not enabled or not _has_evolution_champion


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
