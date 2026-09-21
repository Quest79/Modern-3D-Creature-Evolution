extends Node

const EVENT_PORT_START := 47821
const EVENT_PORT_TRIES := 32
const PORTABLE_DATA_DIR_NAME := "portable_data"
const PORTABLE_RUNTIME_DIR_NAME := "runtime"
const PORTABLE_SAVES_DIR_NAME := "saves"
const PORTABLE_CHAMPIONS_DIR_NAME := "champions"
const PORTABLE_EXPERIMENTS_DIR_NAME := "experiments"
const DEFAULT_HUD_WIDTH := 400.0
const MIN_HUD_WIDTH := 160.0
const MAX_HUD_WIDTH := 400.0
const HUD_RESIZE_HANDLE_WIDTH := 8.0

var _batch_spin: SpinBox
var _workers_spin: SpinBox
var _accelerator_option: OptionButton
var _gpu_ids_edit: LineEdit
var _gpu_batch_spin: SpinBox
var _gpu_max_parts_spin: SpinBox
var _gpu_max_joints_spin: SpinBox
var _cpu_fallback_check: CheckBox
var _throughput_option: OptionButton
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
var _crossover_spin: SpinBox
var _evolution_mutations_spin: SpinBox
var _structural_mutation_spin: SpinBox
var _motor_strength_spin: SpinBox
var _trials_spin: SpinBox
var _trial_aggregation_option: OptionButton
var _fitness_distance_spin: SpinBox
var _fitness_speed_spin: SpinBox
var _fitness_upright_spin: SpinBox
var _fitness_stability_spin: SpinBox
var _fitness_energy_spin: SpinBox
var _terrain_option: OptionButton
var _gravity_x_spin: SpinBox
var _gravity_y_spin: SpinBox
var _gravity_z_spin: SpinBox
var _ground_friction_spin: SpinBox
var _world_seed_spin: SpinBox
var _slope_spin: SpinBox
var _hill_height_spin: SpinBox
var _hill_wavelength_spin: SpinBox
var _stair_height_spin: SpinBox
var _stair_depth_spin: SpinBox
var _walls_check: CheckBox
var _blocks_check: CheckBox
var _gaps_check: CheckBox
var _pits_check: CheckBox
var _obstacle_count_spin: SpinBox
var _obstacle_spacing_spin: SpinBox
var _obstacle_size_spin: SpinBox
var _gap_width_spin: SpinBox
var _pit_depth_spin: SpinBox
var _timeline_generation_spin: SpinBox
var _timeline_condition_option: OptionButton
var _timeline_condition_value_spin: SpinBox
var _timeline_list: ItemList
var _add_timeline_button: Button
var _remove_timeline_button: Button
var _clear_timeline_button: Button

var _seed_creature_button: Button
var _mutate_button: Button
var _random_button: Button
var _save_button: Button
var _load_button: Button
var _live_button: Button
var _batch_button: Button
var _evolve_button: Button
var _resume_evolution_button: Button
var _watch_champion_button: Button
var _stop_button: Button
var _settings_button: Button
var _results_button: Button
var _save_experiment_button: Button
var _load_experiment_button: Button
var _fork_experiment_button: Button

var _status_label: Label
var _progress_bar: ProgressBar
var _metrics: RichTextLabel
var _capabilities_label: Label
var _hud_panel: PanelContainer
var _title_label: Label
var _section_headings: Array[Label] = []
var _ui_theme: Theme
var _settings_window: Window
var _results_window: Window
var _results_history_list: ItemList
var _results_champion_list: ItemList
var _results_lineage_list: ItemList
var _results_lineage_detail: RichTextLabel
var _results_species_list: ItemList
var _results_detail: RichTextLabel
var _results_analysis: RichTextLabel
var _results_graph: Control
var _results_best_line: Line2D
var _results_average_line: Line2D
var _results_median_line: Line2D
var _results_diversity_graph: Control
var _results_morphology_line: Line2D
var _results_class_line: Line2D
var _font_size_spin: SpinBox
var _camera_speed_spin: SpinBox
var _mouse_sensitivity_spin: SpinBox
var _playback_speed_spin: SpinBox
var _hud_resize_handle: ColorRect
var _walkthrough_window: Window
var _walkthrough_text: RichTextLabel
var _walkthrough_back_button: Button
var _walkthrough_next_button: Button
var _walkthrough_run_button: Button

var _probe_mesh: MeshInstance3D
var _camera: Camera3D
var _world_meshes: Array[MeshInstance3D] = []
var _rendered_world_json := ""
var _creature_meshes: Dictionary = {}
var _current_genome: Dictionary = {}
var _current_genome_source := ""
var _current_mutation_count := 0
var _has_evolution_champion := false
var _champion_world: Dictionary = {}
var _champion_motor_strength := 1.0
var _results_data: Dictionary = {}
var _cuda_devices: Array = []

var _font_size := 16
var _accelerator_mode := "cpu"
var _gpu_ids := ""
var _gpu_batch_size := 4096
var _gpu_max_parts := 64
var _gpu_max_joints := 128
var _cpu_fallback := true
var _throughput_mode := "deterministic"
var _camera_move_speed := 6.0
var _mouse_sensitivity_degrees := 0.15
var _playback_speed := 1.0
var _fitness_distance_weight := 1.0
var _fitness_speed_weight := 0.0
var _fitness_upright_weight := 0.0
var _fitness_stability_weight := 0.0
var _fitness_energy_weight := 0.0
var _terrain_kind := "flat"
var _gravity_x := 0.0
var _gravity_y := -9.81
var _gravity_z := 0.0
var _ground_friction := 1.0
var _world_seed := 1
var _slope_degrees := 8.0
var _hill_height := 0.75
var _hill_wavelength := 8.0
var _stair_height := 0.25
var _stair_depth := 1.25
var _walls_enabled := false
var _blocks_enabled := false
var _gaps_enabled := false
var _pits_enabled := false
var _obstacle_count := 6
var _obstacle_spacing := 5.0
var _obstacle_size := 1.0
var _gap_width := 1.5
var _pit_depth := 1.5
var _motor_strength := 1.0
var _trials_per_creature := 1
var _trial_aggregation := "mean"
var _structural_mutation_chance := 0.30
var _timeline_entries: Array = []
var _timeline_next_id := 1
var _experiment_name := "Experiment"
var _experiment_fork_pending := false
var _hud_width := DEFAULT_HUD_WIDTH
var _visible_hud_width := DEFAULT_HUD_WIDTH
var _hud_dragging := false
var _mouse_looking := false
var _camera_yaw := 0.0
var _camera_pitch := 0.0
var _walkthrough_step := 0
var _walkthrough_completed := false
var _walkthrough_version := 0
var _walkthrough_guided_run_active := false

var _save_dialog: FileDialog
var _load_dialog: FileDialog
var _experiment_save_dialog: FileDialog
var _experiment_load_dialog: FileDialog
var _results_load_dialog: FileDialog

var _udp: PacketPeerUDP
var _event_port := 0
var _job_pid := 0
var _job_kind := ""
var _dead_process_since_ms := -1
var _job_started_ms := -1
var _job_received_event := false
var _last_state_time := 0.0

var _replay_frames: Array = []
var _replay_active := false
var _replay_source_complete := false
var _replay_started := false
var _replay_kind := ""
var _replay_clock := 0.0
var _replay_final_time := 0.0


func _ready() -> void:
    _ensure_portable_directories()
    _load_settings()
    _ui_theme = Theme.new()
    _ui_theme.default_font_size = _font_size

    _build_3d_preview()
    _build_ui()
    _build_settings_window()
    _build_results_window()
    _build_file_dialogs()
    _build_walkthrough_window()
    get_viewport().size_changed.connect(_update_layout)
    _update_layout()

    if not _backend_exists():
        _set_status("Rust backend missing. Run BOOTSTRAP_AND_RUN.bat.")
        _set_controls_enabled(false)
        return

    if not _open_event_socket():
        _set_status("Could not open a local event port for the simulator.")
        _set_controls_enabled(false)
        return

    _load_capabilities()
    _refresh_world_preview()
    _set_status("Ready • backend connected on localhost:%d" % _event_port)
    call_deferred("_maybe_show_first_run_walkthrough")


func _process(delta: float) -> void:
    _update_camera_movement(delta)

    if _udp == null:
        return

    while _udp.get_available_packet_count() > 0:
        var raw := _udp.get_packet().get_string_from_utf8()
        var parsed = JSON.parse_string(raw)
        if typeof(parsed) == TYPE_DICTIONARY:
            _handle_event(parsed)

    _update_replay(delta)

    if _job_pid > 0:
        if OS.is_process_running(_job_pid):
            _dead_process_since_ms = -1
            if (
                not _job_received_event
                and _job_started_ms >= 0
                and Time.get_ticks_msec() - _job_started_ms >= 5000
            ):
                OS.kill(_job_pid)
                var timed_out_kind := _job_kind
                _job_pid = 0
                _job_kind = ""
                _job_started_ms = -1
                _reset_replay()
                _set_status(
                    "%s started but sent no events for 5 seconds."
                    % (timed_out_kind if not timed_out_kind.is_empty() else "Simulator")
                )
                _metrics.text = (
                    "[color=#ff8a8a][b]Run failed:[/b] "
                    + "the backend sent no data back to the GUI.[/color]"
                )
                _finish_job_controls()
        elif _dead_process_since_ms < 0:
            # Give final UDP packets a moment to arrive before declaring the
            # process dead. Fast benchmark jobs can otherwise race the GUI.
            _dead_process_since_ms = Time.get_ticks_msec()
        elif Time.get_ticks_msec() - _dead_process_since_ms >= 300:
            var ended_kind := _job_kind
            _job_pid = 0
            _dead_process_since_ms = -1
            _job_started_ms = -1

            if (
                (ended_kind == "live" or ended_kind == "creature")
                and _replay_active
            ):
                if _replay_frames.is_empty():
                    _job_kind = ""
                    _reset_replay()
                    _set_status(
                        "Simulator exited before sending any %s state."
                        % ("creature" if ended_kind == "creature" else "world")
                    )
                    _metrics.text = (
                        "[color=#ff8a8a][b]Test failed:[/b] "
                        + "the simulator process exited before any state arrived.[/color]"
                    )
                    _finish_job_controls()
                elif not _replay_source_complete:
                    # The producer is expected to finish before slow-motion
                    # playback. If its tiny completion packet was dropped, use
                    # the newest buffered frame as the replay end.
                    _replay_source_complete = true
                    _replay_final_time = float(
                        _replay_frames.back().get("simulated_seconds", 0.0)
                    )
            elif ended_kind == "evolution" and _load_genome_file(_evolution_champion_path()):
                _job_kind = ""
                _has_evolution_champion = true
                _load_results_file(_evolution_results_path())
                var recovered_fitness := 0.0
                if not _results_data.is_empty():
                    var recovered_result = _results_data.get("result", {})
                    if typeof(recovered_result) == TYPE_DICTIONARY:
                        recovered_fitness = float(
                            recovered_result.get("champion_fitness", 0.0)
                        )
                var recovered_archive := _archive_current_champion(recovered_fitness)
                _current_genome_source = (
                    recovered_archive
                    if not recovered_archive.is_empty()
                    else _evolution_champion_path()
                )
                _build_creature_from_genome(_current_genome)
                _set_status(
                    "Evolution process ended • latest champion/results recovered"
                    + (" • checkpoint available to Resume"
                        if FileAccess.file_exists(_evolution_checkpoint_path())
                        else "")
                )
                _finish_job_controls()
            else:
                _job_kind = ""
                _reset_replay()
                _set_status("Simulator process ended unexpectedly.")
                _finish_job_controls()


func _exit_tree() -> void:
    _set_mouse_look(false)
    _stop_current_job()
    if _udp != null:
        _udp.close()


func _repo_root_path() -> String:
    var project_dir := ProjectSettings.globalize_path("res://")
    return project_dir.path_join("../..").simplify_path()


func _portable_data_root() -> String:
    return _repo_root_path().path_join(PORTABLE_DATA_DIR_NAME)


func _portable_runtime_dir() -> String:
    return _portable_data_root().path_join(PORTABLE_RUNTIME_DIR_NAME)


func _portable_saves_dir() -> String:
    return _portable_data_root().path_join(PORTABLE_SAVES_DIR_NAME)


func _portable_champions_dir() -> String:
    return _portable_saves_dir().path_join(PORTABLE_CHAMPIONS_DIR_NAME)


func _portable_experiments_dir() -> String:
    return _portable_saves_dir().path_join(PORTABLE_EXPERIMENTS_DIR_NAME)


func _runtime_path(file_name: String) -> String:
    return _portable_runtime_dir().path_join(file_name)


func _settings_path() -> String:
    return _portable_data_root().path_join("settings.cfg")


func _ensure_portable_directories() -> bool:
    var ok := true
    for path in [
        _portable_data_root(),
        _portable_runtime_dir(),
        _portable_saves_dir(),
        _portable_champions_dir(),
        _portable_experiments_dir(),
    ]:
        if DirAccess.make_dir_recursive_absolute(path) != OK:
            if not DirAccess.dir_exists_absolute(path):
                ok = false
                push_error("Could not create portable data directory: %s" % path)
    return ok


func _backend_path() -> String:
    var exe_name := "evolab.exe" if OS.get_name() == "Windows" else "evolab"
    return _repo_root_path().path_join("target").path_join("release").path_join(exe_name)


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

    # Always show a ground plane immediately. The Rust backend replaces this
    # fallback with authoritative world geometry as soon as it is available.
    _build_world_fallback(_world_config_dictionary())

    _camera = Camera3D.new()
    _camera.position = Vector3(9.2, 5.8, 9.2)
    add_child(_camera)
    _camera.look_at(Vector3(2.2, 1.2, 0.0), Vector3.UP)
    _camera_yaw = _camera.rotation.y
    _camera_pitch = _camera.rotation.x


func _build_ui() -> void:
    var layer := CanvasLayer.new()
    add_child(layer)

    _hud_panel = PanelContainer.new()
    _hud_panel.position = Vector2.ZERO
    _hud_panel.size = Vector2(_hud_width, get_viewport().get_visible_rect().size.y)
    _hud_panel.theme = _ui_theme
    layer.add_child(_hud_panel)

    _hud_resize_handle = ColorRect.new()
    _hud_resize_handle.color = Color(0.52, 0.57, 0.66, 0.55)
    _hud_resize_handle.mouse_filter = Control.MOUSE_FILTER_STOP
    _hud_resize_handle.mouse_default_cursor_shape = Control.CURSOR_HSIZE
    _hud_resize_handle.tooltip_text = "Drag to resize the left HUD."
    _hud_resize_handle.gui_input.connect(_on_hud_resize_input)
    layer.add_child(_hud_resize_handle)

    var margin := MarginContainer.new()
    margin.add_theme_constant_override("margin_left", 18)
    margin.add_theme_constant_override("margin_right", 18)
    margin.add_theme_constant_override("margin_top", 14)
    margin.add_theme_constant_override("margin_bottom", 14)
    var scroll := ScrollContainer.new()
    scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
    scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
    _hud_panel.add_child(scroll)

    margin.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    scroll.add_child(margin)

    var column := VBoxContainer.new()
    column.add_theme_constant_override("separation", 6)
    margin.add_child(column)

    var header_row := HBoxContainer.new()
    header_row.add_theme_constant_override("separation", 8)
    column.add_child(header_row)

    _title_label = Label.new()
    _title_label.text = "Modern 3D Creature Evolution"
    _title_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    header_row.add_child(_title_label)

    _results_button = Button.new()
    _results_button.text = "Results"
    _results_button.disabled = true
    _results_button.pressed.connect(_on_results_pressed)
    header_row.add_child(_results_button)

    _settings_button = Button.new()
    _settings_button.text = "⚙ Settings"
    _settings_button.pressed.connect(_on_settings_pressed)
    header_row.add_child(_settings_button)

    var subtitle := Label.new()
    subtitle.text = "Step 8 • GPU / Multi-GPU Acceleration"
    subtitle.modulate = Color(0.72, 0.78, 0.88)
    column.add_child(subtitle)

    column.add_child(HSeparator.new())

    var quick_section := _add_collapsible_section(column, "Quick Test / Main Controls", true)

    var guide_button := Button.new()
    guide_button.text = "▶ Guided First Test"
    guide_button.tooltip_text = "Walk through a safe first creature simulation and explain what each part does."
    guide_button.pressed.connect(_show_walkthrough)
    quick_section.add_child(guide_button)

    var quick_test_row := HBoxContainer.new()
    quick_test_row.add_theme_constant_override("separation", 6)
    quick_section.add_child(quick_test_row)

    _seed_creature_button = Button.new()
    _seed_creature_button.text = "Run Seed Creature"
    _seed_creature_button.tooltip_text = "Run the known three-segment creature in the current world."
    _seed_creature_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _seed_creature_button.pressed.connect(_on_seed_creature_pressed)
    quick_test_row.add_child(_seed_creature_button)

    _live_button = Button.new()
    _live_button.text = "Single Box"
    _live_button.tooltip_text = "Run the simplest physics sanity check."
    _live_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _live_button.pressed.connect(_on_live_pressed)
    quick_test_row.add_child(_live_button)

    _batch_button = Button.new()
    _batch_button.text = "Benchmark"
    _batch_button.tooltip_text = "Run many independent simulations and measure throughput."
    _batch_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _batch_button.pressed.connect(_on_batch_pressed)
    quick_test_row.add_child(_batch_button)

    var quick_evolution_row := HBoxContainer.new()
    quick_evolution_row.add_theme_constant_override("separation", 6)
    quick_section.add_child(quick_evolution_row)

    _evolve_button = Button.new()
    _evolve_button.text = "🧬 Start Evolution"
    _evolve_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _evolve_button.pressed.connect(_on_evolve_pressed)
    quick_evolution_row.add_child(_evolve_button)

    _resume_evolution_button = Button.new()
    _resume_evolution_button.text = "Resume"
    _resume_evolution_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _resume_evolution_button.disabled = not FileAccess.file_exists(
        _evolution_checkpoint_path()
    )
    _resume_evolution_button.pressed.connect(_on_resume_evolution_pressed)
    quick_evolution_row.add_child(_resume_evolution_button)

    _watch_champion_button = Button.new()
    _watch_champion_button.text = "Watch Champion"
    _watch_champion_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _watch_champion_button.disabled = true
    _watch_champion_button.pressed.connect(_on_watch_champion_pressed)
    quick_evolution_row.add_child(_watch_champion_button)

    _stop_button = Button.new()
    _stop_button.text = "■ Stop Current Run"
    _stop_button.custom_minimum_size = Vector2(0, 32)
    _stop_button.disabled = true
    _stop_button.pressed.connect(_on_stop_pressed)
    quick_section.add_child(_stop_button)

    var creature_section := _add_collapsible_section(column, "Creature Generation & Files", false)

    _seed_spin = _add_number_row(creature_section, "Seed", 1, 999999999, 1, 1)
    _mutation_spin = _add_number_row(creature_section, "Mutation operations", 0, 500, 12, 1)
    _random_segments_spin = _add_number_row(creature_section, "Random creature segments", 2, 40, 5, 1)
    _max_segments_spin = _add_number_row(creature_section, "Maximum segments", 2, 40, 12, 1)

    var creature_row := HBoxContainer.new()
    creature_row.add_theme_constant_override("separation", 6)
    creature_section.add_child(creature_row)

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
    creature_section.add_child(file_row)

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

    var experiment_row := HBoxContainer.new()
    experiment_row.add_theme_constant_override("separation", 6)
    creature_section.add_child(experiment_row)

    _save_experiment_button = Button.new()
    _save_experiment_button.text = "Save Experiment..."
    _save_experiment_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _save_experiment_button.pressed.connect(_on_save_experiment_pressed)
    experiment_row.add_child(_save_experiment_button)

    _load_experiment_button = Button.new()
    _load_experiment_button.text = "Load Experiment..."
    _load_experiment_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _load_experiment_button.pressed.connect(_on_load_experiment_pressed)
    experiment_row.add_child(_load_experiment_button)

    _fork_experiment_button = Button.new()
    _fork_experiment_button.text = "Fork..."
    _fork_experiment_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _fork_experiment_button.pressed.connect(_on_fork_experiment_pressed)
    experiment_row.add_child(_fork_experiment_button)

    var evolution_section := _add_collapsible_section(column, "Evolution Settings", false)

    _population_spin = _add_number_row(evolution_section, "Population", 2, 10000, 50, 1)
    _generations_spin = _add_number_row(evolution_section, "Generations", 1, 10000, 100, 1)
    _tournament_spin = _add_number_row(evolution_section, "Tournament size", 1, 1000, 7, 1)
    _elite_spin = _add_number_row(evolution_section, "Elite kept", 1, 999, 2, 1)
    _crossover_spin = _add_number_row(evolution_section, "Brain crossover", 0.0, 1.0, 0.5, 0.05)
    _crossover_spin.tooltip_text = "Chance that a child receives a brain subtree from a second selected parent."
    _evolution_mutations_spin = _add_number_row(evolution_section, "Mutations / child", 1, 500, 8, 1)
    _structural_mutation_spin = _add_number_row(
        evolution_section,
        "Structural mutation chance",
        0.0,
        1.0,
        _structural_mutation_chance,
        0.01
    )
    _motor_strength_spin = _add_number_row(
        evolution_section, "Motor activation", 0.0, 1.0, _motor_strength, 0.01
    )
    _motor_strength_spin.suffix = "× biological max"
    _trials_spin = _add_number_row(
        evolution_section, "Trials / creature", 1, 100, _trials_per_creature, 1
    )

    var aggregation_row := HBoxContainer.new()
    evolution_section.add_child(aggregation_row)
    var aggregation_label := Label.new()
    aggregation_label.text = "Trial aggregation"
    aggregation_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    aggregation_row.add_child(aggregation_label)
    _trial_aggregation_option = OptionButton.new()
    _trial_aggregation_option.custom_minimum_size = Vector2(145, 29)
    for aggregation_name in ["Mean", "Median", "Worst", "Best"]:
        _trial_aggregation_option.add_item(aggregation_name)
    var aggregation_index := ["mean", "median", "worst", "best"].find(
        _trial_aggregation
    )
    _trial_aggregation_option.select(maxi(aggregation_index, 0))
    _trial_aggregation_option.item_selected.connect(_on_trial_aggregation_selected)
    aggregation_row.add_child(_trial_aggregation_option)

    _structural_mutation_spin.value_changed.connect(_on_schedule_base_changed)
    _motor_strength_spin.value_changed.connect(_on_schedule_base_changed)
    _trials_spin.value_changed.connect(_on_schedule_base_changed)

    var fitness_section := _add_collapsible_section(column, "Fitness Weights", false)

    _fitness_distance_spin = _add_number_row(
        fitness_section, "Distance", -100.0, 100.0, _fitness_distance_weight, 0.05
    )
    _fitness_speed_spin = _add_number_row(
        fitness_section, "Average speed", -100.0, 100.0, _fitness_speed_weight, 0.05
    )
    _fitness_upright_spin = _add_number_row(
        fitness_section, "Upright", -100.0, 100.0, _fitness_upright_weight, 0.05
    )
    _fitness_stability_spin = _add_number_row(
        fitness_section, "Stability", -100.0, 100.0, _fitness_stability_weight, 0.05
    )
    _fitness_energy_spin = _add_number_row(
        fitness_section, "Energy / effort", -100.0, 100.0, _fitness_energy_weight, 0.01
    )
    _fitness_energy_spin.tooltip_text = "Use a negative weight to penalize actuator effort."

    for spin in [
        _fitness_distance_spin,
        _fitness_speed_spin,
        _fitness_upright_spin,
        _fitness_stability_spin,
        _fitness_energy_spin,
    ]:
        spin.value_changed.connect(_on_fitness_weights_changed)

    var world_section := _add_collapsible_section(column, "World / Terrain", false)

    var terrain_row := HBoxContainer.new()
    world_section.add_child(terrain_row)
    var terrain_label := Label.new()
    terrain_label.text = "Terrain"
    terrain_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    terrain_row.add_child(terrain_label)
    _terrain_option = OptionButton.new()
    _terrain_option.custom_minimum_size = Vector2(145, 29)
    for terrain_name in ["Flat", "Slope", "Hills", "Stairs"]:
        _terrain_option.add_item(terrain_name)
    var terrain_index := ["flat", "slope", "hills", "stairs"].find(_terrain_kind)
    _terrain_option.select(maxi(terrain_index, 0))
    _terrain_option.item_selected.connect(_on_terrain_selected)
    terrain_row.add_child(_terrain_option)

    _world_seed_spin = _add_number_row(world_section, "World seed", 0, 999999999, _world_seed, 1)
    _gravity_x_spin = _add_number_row(world_section, "Gravity X", -30.0, 30.0, _gravity_x, 0.1)
    _gravity_y_spin = _add_number_row(world_section, "Gravity Y", -30.0, 30.0, _gravity_y, 0.1)
    _gravity_z_spin = _add_number_row(world_section, "Gravity Z", -30.0, 30.0, _gravity_z, 0.1)
    _ground_friction_spin = _add_number_row(
        world_section, "Ground friction", 0.0, 5.0, _ground_friction, 0.05
    )
    _slope_spin = _add_number_row(world_section, "Slope angle (deg)", -35.0, 35.0, _slope_degrees, 0.5)
    _hill_height_spin = _add_number_row(world_section, "Hill height", 0.0, 10.0, _hill_height, 0.05)
    _hill_wavelength_spin = _add_number_row(
        world_section, "Hill wavelength", 1.0, 100.0, _hill_wavelength, 0.25
    )
    _stair_height_spin = _add_number_row(
        world_section, "Stair height", 0.01, 5.0, _stair_height, 0.05
    )
    _stair_depth_spin = _add_number_row(
        world_section, "Stair depth", 0.1, 20.0, _stair_depth, 0.05
    )

    var obstacle_label := Label.new()
    obstacle_label.text = "Obstacle types"
    world_section.add_child(obstacle_label)

    var obstacle_row_a := HBoxContainer.new()
    obstacle_row_a.add_theme_constant_override("separation", 10)
    world_section.add_child(obstacle_row_a)
    _walls_check = CheckBox.new()
    _walls_check.text = "Walls"
    _walls_check.button_pressed = _walls_enabled
    obstacle_row_a.add_child(_walls_check)
    _blocks_check = CheckBox.new()
    _blocks_check.text = "Blocks"
    _blocks_check.button_pressed = _blocks_enabled
    obstacle_row_a.add_child(_blocks_check)

    var obstacle_row_b := HBoxContainer.new()
    obstacle_row_b.add_theme_constant_override("separation", 10)
    world_section.add_child(obstacle_row_b)
    _gaps_check = CheckBox.new()
    _gaps_check.text = "Gaps"
    _gaps_check.button_pressed = _gaps_enabled
    obstacle_row_b.add_child(_gaps_check)
    _pits_check = CheckBox.new()
    _pits_check.text = "Pits"
    _pits_check.button_pressed = _pits_enabled
    obstacle_row_b.add_child(_pits_check)

    _obstacle_count_spin = _add_number_row(
        world_section, "Obstacle count", 0, 100, _obstacle_count, 1
    )
    _obstacle_spacing_spin = _add_number_row(
        world_section, "Obstacle spacing", 1.0, 50.0, _obstacle_spacing, 0.25
    )
    _obstacle_size_spin = _add_number_row(
        world_section, "Obstacle size", 0.1, 10.0, _obstacle_size, 0.05
    )
    _gap_width_spin = _add_number_row(
        world_section, "Gap width", 0.1, 10.0, _gap_width, 0.05
    )
    _pit_depth_spin = _add_number_row(
        world_section, "Pit depth", 0.1, 20.0, _pit_depth, 0.05
    )

    for world_spin in [
        _world_seed_spin,
        _gravity_x_spin,
        _gravity_y_spin,
        _gravity_z_spin,
        _ground_friction_spin,
        _slope_spin,
        _hill_height_spin,
        _hill_wavelength_spin,
        _stair_height_spin,
        _stair_depth_spin,
        _obstacle_count_spin,
        _obstacle_spacing_spin,
        _obstacle_size_spin,
        _gap_width_spin,
        _pit_depth_spin,
    ]:
        world_spin.value_changed.connect(_on_world_numeric_changed)

    for world_check in [_walls_check, _blocks_check, _gaps_check, _pits_check]:
        world_check.toggled.connect(_on_world_toggle_changed)

    var timeline_section := _add_collapsible_section(column, "Experiment Timeline", false)

    var timeline_help := Label.new()
    timeline_help.text = (
        "Keyframes snapshot the current world, fitness, population, mutation, "
        + "motor, duration, and trial settings."
    )
    timeline_help.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    timeline_help.modulate = Color(0.70, 0.76, 0.86)
    timeline_section.add_child(timeline_help)

    _timeline_generation_spin = _add_number_row(
        timeline_section, "Keyframe generation", 1, 10000, 1, 1
    )

    var condition_row := HBoxContainer.new()
    timeline_section.add_child(condition_row)
    var condition_label := Label.new()
    condition_label.text = "Activation"
    condition_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    condition_row.add_child(condition_label)
    _timeline_condition_option = OptionButton.new()
    _timeline_condition_option.custom_minimum_size = Vector2(190, 29)
    for condition_name in [
        "At generation",
        "Best fitness ≥",
        "Average fitness ≥",
        "Best distance ≥",
    ]:
        _timeline_condition_option.add_item(condition_name)
    _timeline_condition_option.item_selected.connect(_on_timeline_condition_selected)
    condition_row.add_child(_timeline_condition_option)

    _timeline_condition_value_spin = _add_number_row(
        timeline_section, "Condition threshold", -100000.0, 100000.0, 1.0, 0.1
    )
    _timeline_condition_value_spin.editable = false

    _timeline_list = ItemList.new()
    _timeline_list.custom_minimum_size = Vector2(0, 115)
    _timeline_list.select_mode = ItemList.SELECT_SINGLE
    timeline_section.add_child(_timeline_list)

    var timeline_buttons := HBoxContainer.new()
    timeline_buttons.add_theme_constant_override("separation", 6)
    timeline_section.add_child(timeline_buttons)

    _add_timeline_button = Button.new()
    _add_timeline_button.text = "Add Snapshot"
    _add_timeline_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _add_timeline_button.pressed.connect(_on_add_timeline_keyframe)
    timeline_buttons.add_child(_add_timeline_button)

    _remove_timeline_button = Button.new()
    _remove_timeline_button.text = "Remove"
    _remove_timeline_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _remove_timeline_button.pressed.connect(_on_remove_timeline_keyframe)
    timeline_buttons.add_child(_remove_timeline_button)

    _clear_timeline_button = Button.new()
    _clear_timeline_button.text = "Clear"
    _clear_timeline_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _clear_timeline_button.pressed.connect(_on_clear_timeline)
    timeline_buttons.add_child(_clear_timeline_button)

    _refresh_timeline_list()

    var sim_section := _add_collapsible_section(column, "Simulation / Benchmark Details", false)

    _seconds_spin = _add_number_row(sim_section, "Seconds / simulation", 0.1, 120.0, 8.0, 0.1)
    _dt_spin = _add_number_row(sim_section, "Physics dt (seconds)", 0.0001, 0.05, 1.0 / 120.0, 0.0001)
    _playback_speed_spin = _add_number_row(
        sim_section,
        "Playback speed",
        0.01,
        2.0,
        _playback_speed,
        0.01
    )
    _playback_speed_spin.suffix = "x"
    _playback_speed_spin.tooltip_text = "Live viewer speed. 0.01x = 100× slower, 2.00x = 2× faster."
    _playback_speed_spin.value_changed.connect(_on_playback_speed_changed)
    _batch_spin = _add_number_row(
        sim_section, "Parallel simulations / generation", 1, 1000000, 1000, 1
    )
    _batch_spin.tooltip_text = (
        "Benchmark world count. During CUDA evolution this is also the candidate "
        + "evaluation pool per generation; Population remains the survivor/parent pool."
    )
    _workers_spin = _add_number_row(sim_section, "CPU workers (0 = auto)", 0, 256, 0, 1)

    var accelerator_row := HBoxContainer.new()
    sim_section.add_child(accelerator_row)
    var accelerator_label := Label.new()
    accelerator_label.text = "Execution backend"
    accelerator_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    accelerator_row.add_child(accelerator_label)
    _accelerator_option = OptionButton.new()
    _accelerator_option.custom_minimum_size = Vector2(145, 29)
    for accelerator_name in ["CPU", "Auto", "CUDA"]:
        _accelerator_option.add_item(accelerator_name)
    var accelerator_index := ["cpu", "auto", "cuda"].find(_accelerator_mode)
    _accelerator_option.select(maxi(accelerator_index, 0))
    _accelerator_option.item_selected.connect(_on_accelerator_selected)
    accelerator_row.add_child(_accelerator_option)

    var gpu_ids_row := HBoxContainer.new()
    sim_section.add_child(gpu_ids_row)
    var gpu_ids_label := Label.new()
    gpu_ids_label.text = "CUDA GPU IDs"
    gpu_ids_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    gpu_ids_row.add_child(gpu_ids_label)
    _gpu_ids_edit = LineEdit.new()
    _gpu_ids_edit.custom_minimum_size = Vector2(145, 29)
    _gpu_ids_edit.placeholder_text = "all (or 0,1)"
    _gpu_ids_edit.text = _gpu_ids
    _gpu_ids_edit.text_changed.connect(_on_gpu_ids_changed)
    gpu_ids_row.add_child(_gpu_ids_edit)

    _gpu_batch_spin = _add_number_row(
        sim_section, "GPU batch size", 1, 1000000, _gpu_batch_size, 1
    )
    _gpu_max_parts_spin = _add_number_row(
        sim_section, "GPU max parts", 1, 1024, _gpu_max_parts, 1
    )
    _gpu_max_joints_spin = _add_number_row(
        sim_section, "GPU max joints", 1, 2048, _gpu_max_joints, 1
    )
    for accelerator_spin in [_gpu_batch_spin, _gpu_max_parts_spin, _gpu_max_joints_spin]:
        accelerator_spin.value_changed.connect(_on_accelerator_number_changed)

    _cpu_fallback_check = CheckBox.new()
    _cpu_fallback_check.text = "Allow CPU fallback for unsupported accelerator work"
    _cpu_fallback_check.button_pressed = _cpu_fallback
    _cpu_fallback_check.toggled.connect(_on_cpu_fallback_toggled)
    sim_section.add_child(_cpu_fallback_check)

    var throughput_row := HBoxContainer.new()
    sim_section.add_child(throughput_row)
    var throughput_label := Label.new()
    throughput_label.text = "Throughput policy"
    throughput_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    throughput_row.add_child(throughput_label)
    _throughput_option = OptionButton.new()
    _throughput_option.custom_minimum_size = Vector2(145, 29)
    _throughput_option.add_item("Deterministic")
    _throughput_option.add_item("Max throughput")
    _throughput_option.select(0 if _throughput_mode == "deterministic" else 1)
    _throughput_option.item_selected.connect(_on_throughput_selected)
    throughput_row.add_child(_throughput_option)

    var status_section := _add_collapsible_section(column, "Status / Results", true)

    _status_label = Label.new()
    _status_label.text = "Starting..."
    _status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    status_section.add_child(_status_label)

    _progress_bar = ProgressBar.new()
    _progress_bar.min_value = 0
    _progress_bar.max_value = 100
    _progress_bar.value = 0
    _progress_bar.show_percentage = true
    status_section.add_child(_progress_bar)

    status_section.add_child(HSeparator.new())

    _metrics = RichTextLabel.new()
    _metrics.bbcode_enabled = true
    _metrics.fit_content = false
    _metrics.custom_minimum_size = Vector2(0, 155)
    _metrics.text = "[color=#9aa7bd]Generate, mutate, load, or watch a creature.[/color]"
    status_section.add_child(_metrics)

    _capabilities_label = Label.new()
    _capabilities_label.text = "Backend capabilities: loading..."
    _capabilities_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    _capabilities_label.modulate = Color(0.64, 0.7, 0.8)
    status_section.add_child(_capabilities_label)

    _apply_font_size()


func _build_results_window() -> void:
    _results_window = Window.new()
    _results_window.title = "Evolution Results / History"
    _results_window.size = Vector2i(980, 720)
    _results_window.min_size = Vector2i(760, 560)
    _results_window.visible = false
    _results_window.theme = _ui_theme
    _results_window.close_requested.connect(_results_window.hide)
    add_child(_results_window)

    var margin := MarginContainer.new()
    margin.add_theme_constant_override("margin_left", 14)
    margin.add_theme_constant_override("margin_right", 14)
    margin.add_theme_constant_override("margin_top", 14)
    margin.add_theme_constant_override("margin_bottom", 14)
    margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
    _results_window.add_child(margin)

    var root_column := VBoxContainer.new()
    root_column.add_theme_constant_override("separation", 8)
    margin.add_child(root_column)

    var results_toolbar := HBoxContainer.new()
    root_column.add_child(results_toolbar)

    var load_results_button := Button.new()
    load_results_button.text = "Load Results..."
    load_results_button.pressed.connect(_on_load_results_pressed)
    results_toolbar.add_child(load_results_button)

    var results_title := Label.new()
    results_title.text = "Persisted evolution analysis"
    results_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    results_title.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
    results_title.modulate = Color(0.70, 0.76, 0.86)
    results_toolbar.add_child(results_title)

    var tabs := TabContainer.new()
    tabs.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
    root_column.add_child(tabs)

    var history_tab := VBoxContainer.new()
    history_tab.name = "History"
    history_tab.add_theme_constant_override("separation", 8)
    tabs.add_child(history_tab)

    _results_graph = Control.new()
    _results_graph.custom_minimum_size = Vector2(0, 190)
    _results_graph.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    var graph_bg := ColorRect.new()
    graph_bg.color = Color(0.05, 0.06, 0.08, 0.92)
    graph_bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
    graph_bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
    _results_graph.add_child(graph_bg)

    _results_best_line = Line2D.new()
    _results_best_line.width = 2.5
    _results_best_line.default_color = Color(0.35, 0.85, 1.0)
    _results_graph.add_child(_results_best_line)

    _results_average_line = Line2D.new()
    _results_average_line.width = 2.0
    _results_average_line.default_color = Color(1.0, 0.72, 0.32)
    _results_graph.add_child(_results_average_line)

    _results_median_line = Line2D.new()
    _results_median_line.width = 2.0
    _results_median_line.default_color = Color(0.55, 1.0, 0.72)
    _results_graph.add_child(_results_median_line)

    history_tab.add_child(_results_graph)

    var graph_key := Label.new()
    graph_key.text = (
        "Best fitness (blue)   Average fitness (orange)   Median fitness (green)"
    )
    graph_key.modulate = Color(0.72, 0.78, 0.88)
    history_tab.add_child(graph_key)

    var history_split := HSplitContainer.new()
    history_split.size_flags_vertical = Control.SIZE_EXPAND_FILL
    history_tab.add_child(history_split)

    _results_history_list = ItemList.new()
    _results_history_list.custom_minimum_size = Vector2(360, 260)
    _results_history_list.item_selected.connect(_on_result_history_selected)
    history_split.add_child(_results_history_list)

    _results_detail = RichTextLabel.new()
    _results_detail.bbcode_enabled = true
    _results_detail.fit_content = false
    _results_detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _results_detail.size_flags_vertical = Control.SIZE_EXPAND_FILL
    history_split.add_child(_results_detail)

    var champions_tab := VBoxContainer.new()
    champions_tab.name = "Champions"
    champions_tab.add_theme_constant_override("separation", 8)
    tabs.add_child(champions_tab)

    _results_champion_list = ItemList.new()
    _results_champion_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
    _results_champion_list.item_selected.connect(_on_result_champion_selected)
    champions_tab.add_child(_results_champion_list)

    var champion_buttons := HBoxContainer.new()
    champions_tab.add_child(champion_buttons)

    var load_champion_button := Button.new()
    load_champion_button.text = "Load Selected Champion"
    load_champion_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    load_champion_button.pressed.connect(_on_load_archived_champion)
    champion_buttons.add_child(load_champion_button)

    var compare_champion_button := Button.new()
    compare_champion_button.text = "Compare Selected vs Final"
    compare_champion_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    compare_champion_button.pressed.connect(_on_compare_archived_champion)
    champion_buttons.add_child(compare_champion_button)

    var lineage_tab := VBoxContainer.new()
    lineage_tab.name = "Lineage"
    lineage_tab.add_theme_constant_override("separation", 8)
    tabs.add_child(lineage_tab)

    var lineage_split := HSplitContainer.new()
    lineage_split.size_flags_vertical = Control.SIZE_EXPAND_FILL
    lineage_tab.add_child(lineage_split)

    _results_lineage_list = ItemList.new()
    _results_lineage_list.custom_minimum_size = Vector2(520, 260)
    _results_lineage_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
    _results_lineage_list.item_selected.connect(_on_result_lineage_selected)
    lineage_split.add_child(_results_lineage_list)

    _results_lineage_detail = RichTextLabel.new()
    _results_lineage_detail.bbcode_enabled = true
    _results_lineage_detail.fit_content = false
    _results_lineage_detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    _results_lineage_detail.size_flags_vertical = Control.SIZE_EXPAND_FILL
    lineage_split.add_child(_results_lineage_detail)

    var lineage_buttons := HBoxContainer.new()
    lineage_tab.add_child(lineage_buttons)

    var replay_lineage_button := Button.new()
    replay_lineage_button.text = "Replay Selected"
    replay_lineage_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    replay_lineage_button.pressed.connect(_on_replay_lineage_creature)
    lineage_buttons.add_child(replay_lineage_button)

    var load_lineage_button := Button.new()
    load_lineage_button.text = "Load Selected"
    load_lineage_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    load_lineage_button.pressed.connect(_on_load_lineage_creature)
    lineage_buttons.add_child(load_lineage_button)

    var compare_lineage_button := Button.new()
    compare_lineage_button.text = "Compare Selected vs Final"
    compare_lineage_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    compare_lineage_button.pressed.connect(_on_compare_lineage_creature)
    lineage_buttons.add_child(compare_lineage_button)

    var fork_lineage_button := Button.new()
    fork_lineage_button.text = "Fork Experiment From Selected..."
    fork_lineage_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    fork_lineage_button.pressed.connect(_on_fork_lineage_creature)
    lineage_buttons.add_child(fork_lineage_button)

    var classes_tab := VBoxContainer.new()
    classes_tab.name = "Classes"
    classes_tab.add_theme_constant_override("separation", 8)
    tabs.add_child(classes_tab)

    var classes_help := Label.new()
    classes_help.text = (
        "Descriptive analysis classes: segment count × brain-size bucket. "
        + "These are not yet selection/speciation groups."
    )
    classes_help.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    classes_help.modulate = Color(0.70, 0.76, 0.86)
    classes_tab.add_child(classes_help)

    _results_species_list = ItemList.new()
    _results_species_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
    classes_tab.add_child(_results_species_list)

    var analysis_tab := VBoxContainer.new()
    analysis_tab.name = "Analysis"
    analysis_tab.add_theme_constant_override("separation", 8)
    tabs.add_child(analysis_tab)

    _results_diversity_graph = Control.new()
    _results_diversity_graph.custom_minimum_size = Vector2(0, 180)
    _results_diversity_graph.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    var diversity_bg := ColorRect.new()
    diversity_bg.color = Color(0.05, 0.06, 0.08, 0.92)
    diversity_bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
    diversity_bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
    _results_diversity_graph.add_child(diversity_bg)

    _results_morphology_line = Line2D.new()
    _results_morphology_line.width = 2.5
    _results_morphology_line.default_color = Color(0.45, 0.90, 0.55)
    _results_diversity_graph.add_child(_results_morphology_line)

    _results_class_line = Line2D.new()
    _results_class_line.width = 2.0
    _results_class_line.default_color = Color(0.85, 0.55, 1.0)
    _results_diversity_graph.add_child(_results_class_line)

    analysis_tab.add_child(_results_diversity_graph)

    var diversity_key := Label.new()
    diversity_key.text = "Unique morphologies (green)   Analysis classes (purple)"
    diversity_key.modulate = Color(0.72, 0.78, 0.88)
    analysis_tab.add_child(diversity_key)

    _results_analysis = RichTextLabel.new()
    _results_analysis.bbcode_enabled = true
    _results_analysis.fit_content = false
    _results_analysis.size_flags_vertical = Control.SIZE_EXPAND_FILL
    analysis_tab.add_child(_results_analysis)


func _on_results_pressed() -> void:
    if _results_data.is_empty():
        return
    _set_mouse_look(false)
    _refresh_results_window()
    _results_window.popup_centered()


func _on_load_results_pressed() -> void:
    if _job_pid > 0:
        return
    _results_load_dialog.popup_centered_ratio(0.72)


func _on_results_load_file_selected(path: String) -> void:
    if _load_results_file(path):
        _set_status("Loaded evolution results: %s" % path)
        _results_window.popup_centered()
    else:
        _set_status("Could not load evolution results: %s" % path)


func _refresh_results_window() -> void:
    if _results_history_list == null:
        return

    _results_history_list.clear()
    _results_champion_list.clear()
    _results_lineage_list.clear()
    _results_species_list.clear()
    _results_best_line.clear_points()
    _results_average_line.clear_points()
    _results_median_line.clear_points()
    _results_morphology_line.clear_points()
    _results_class_line.clear_points()

    if _results_data.is_empty():
        _results_detail.text = "[color=#9aa7bd]No evolution results loaded.[/color]"
        _results_lineage_detail.text = ""
        _results_analysis.text = ""
        return

    var result_value = _results_data.get("result", {})
    if typeof(result_value) != TYPE_DICTIONARY:
        return
    var result: Dictionary = result_value
    var history: Array = result.get("history", [])
    var archive: Array = result.get("champion_archive", [])

    for summary_value in history:
        if typeof(summary_value) != TYPE_DICTIONARY:
            continue
        var summary: Dictionary = summary_value
        _results_history_list.add_item(
            "Gen %d   best %.4f   avg %.4f   distance %.3f m"
            % [
                int(summary.get("generation", 0)),
                float(summary.get("best_fitness", 0.0)),
                float(summary.get("average_fitness", 0.0)),
                float(summary.get("best_distance", 0.0)),
            ]
        )

        var lineage: Array = summary.get("lineage", [])
        for record_value in lineage:
            if typeof(record_value) != TYPE_DICTIONARY:
                continue
            var record: Dictionary = record_value
            var record_mutations = record.get("mutations", [])
            var record_mutation_count := 0
            if typeof(record_mutations) == TYPE_ARRAY:
                record_mutation_count = record_mutations.size()
            _results_lineage_list.add_item(
                "G%d  #%s  parents [%s]  class %s  fit %.4f  seg %s  brain %s  mut %d"
                % [
                    int(record.get("generation", 0)),
                    str(record.get("individual_id", 0)),
                    _format_parent_ids(record.get("parent_ids", [])),
                    str(record.get("species_id", 0)),
                    float(record.get("fitness", 0.0)),
                    str(record.get("segments", 0)),
                    str(record.get("brain_nodes", 0)),
                    record_mutation_count,
                ]
            )

    for entry_value in archive:
        if typeof(entry_value) != TYPE_DICTIONARY:
            continue
        var entry: Dictionary = entry_value
        _results_champion_list.add_item(
            "Gen %d   #%s   fitness %.4f   distance %.3f m   parents [%s]"
            % [
                int(entry.get("generation", 0)),
                str(entry.get("individual_id", 0)),
                float(entry.get("fitness", 0.0)),
                float(entry.get("metrics", {}).get("distance", 0.0)),
                _format_parent_ids(entry.get("parent_ids", [])),
            ]
        )

    if not history.is_empty():
        _results_history_list.select(history.size() - 1)
        _show_result_generation(history.size() - 1)
    if _results_lineage_list.item_count > 0:
        _results_lineage_list.select(0)
        _on_result_lineage_selected(0)

    _refresh_results_analysis()
    call_deferred("_refresh_results_graph")
    call_deferred("_refresh_diversity_graph")


func _refresh_results_graph() -> void:
    if _results_data.is_empty() or _results_graph == null:
        return

    var result: Dictionary = _results_data.get("result", {})
    var history: Array = result.get("history", [])
    if history.is_empty():
        return

    var best_values: Array[float] = []
    var average_values: Array[float] = []
    var median_values: Array[float] = []
    var minimum := INF
    var maximum := -INF

    for summary_value in history:
        if typeof(summary_value) != TYPE_DICTIONARY:
            continue
        var summary: Dictionary = summary_value
        var best := float(summary.get("best_fitness", 0.0))
        var average := float(summary.get("average_fitness", 0.0))
        var median_value := float(summary.get("median_fitness", average))
        best_values.append(best)
        average_values.append(average)
        median_values.append(median_value)
        minimum = minf(minimum, minf(best, minf(average, median_value)))
        maximum = maxf(maximum, maxf(best, maxf(average, median_value)))

    if best_values.is_empty():
        return

    if absf(maximum - minimum) < 0.000001:
        maximum = minimum + 1.0

    var graph_size := _results_graph.size
    if graph_size.x < 100.0:
        graph_size.x = 900.0
    if graph_size.y < 100.0:
        graph_size.y = 190.0

    var left := 12.0
    var right := graph_size.x - 12.0
    var top := 12.0
    var bottom := graph_size.y - 12.0
    var denominator := maxf(float(best_values.size() - 1), 1.0)

    _results_best_line.clear_points()
    _results_average_line.clear_points()
    _results_median_line.clear_points()

    for index in range(best_values.size()):
        var x := lerpf(left, right, float(index) / denominator)
        var best_y := lerpf(
            bottom,
            top,
            (best_values[index] - minimum) / (maximum - minimum)
        )
        var average_y := lerpf(
            bottom,
            top,
            (average_values[index] - minimum) / (maximum - minimum)
        )
        var median_y := lerpf(
            bottom,
            top,
            (median_values[index] - minimum) / (maximum - minimum)
        )
        _results_best_line.add_point(Vector2(x, best_y))
        _results_average_line.add_point(Vector2(x, average_y))
        _results_median_line.add_point(Vector2(x, median_y))


func _on_result_history_selected(index: int) -> void:
    _show_result_generation(index)


func _show_result_generation(index: int) -> void:
    if _results_data.is_empty():
        return

    var result: Dictionary = _results_data.get("result", {})
    var history: Array = result.get("history", [])
    if index < 0 or index >= history.size():
        return
    if typeof(history[index]) != TYPE_DICTIONARY:
        return

    var summary: Dictionary = history[index]
    var metrics: Dictionary = summary.get("best_metrics", {})
    var diversity: Dictionary = summary.get("diversity", {})
    var species: Array = summary.get("species", [])
    var pareto: Array = summary.get("pareto_front", [])
    var map_cells: Array = summary.get("map_elites", [])

    _results_species_list.clear()
    for species_value in species:
        if typeof(species_value) != TYPE_DICTIONARY:
            continue
        var species_entry: Dictionary = species_value
        _results_species_list.add_item(
            "Class %s   seg %s   brain bucket %s   members %s   best %.4f   avg %.4f"
            % [
                str(species_entry.get("species_id", 0)),
                str(species_entry.get("segment_count", 0)),
                str(species_entry.get("brain_node_bucket", 0)),
                str(species_entry.get("members", 0)),
                float(species_entry.get("best_fitness", 0.0)),
                float(species_entry.get("average_fitness", 0.0)),
            ]
        )

    _results_detail.text = (
        "[b]Generation %d[/b]

"
        + "[table=2]"
        + "[cell]Champion ID[/cell][cell]#%s[/cell]"
        + "[cell]Parents[/cell][cell]%s[/cell]"
        + "[cell]Analysis class[/cell][cell]%s[/cell]"
        + "[cell]Best fitness[/cell][cell][b]%.5f[/b][/cell]"
        + "[cell]Average / median / worst[/cell][cell]%.5f / %.5f / %.5f[/cell]"
        + "[cell]Distance[/cell][cell]%.4f m[/cell]"
        + "[cell]Speed[/cell][cell]%.4f m/s[/cell]"
        + "[cell]Upright[/cell][cell]%.3f[/cell]"
        + "[cell]Stability[/cell][cell]%.3f[/cell]"
        + "[cell]Energy[/cell][cell]%.4f[/cell]"
        + "[cell]Fitness σ[/cell][cell]%.4f[/cell]"
        + "[cell]Unique morphologies[/cell][cell]%s[/cell]"
        + "[cell]Analysis classes[/cell][cell]%s[/cell]"
        + "[cell]Pareto front[/cell][cell]%s creatures[/cell]"
        + "[cell]MAP-Elites occupied[/cell][cell]%s cells[/cell]"
        + "[cell]Segments mean ± σ[/cell][cell]%.2f ± %.2f[/cell]"
        + "[cell]Brain nodes mean ± σ[/cell][cell]%.2f ± %.2f[/cell]"
        + "[/table]"
        % [
            int(summary.get("generation", 0)),
            str(summary.get("champion_id", 0)),
            _format_parent_ids(summary.get("champion_parent_ids", [])),
            str(summary.get("champion_species_id", 0)),
            float(summary.get("best_fitness", 0.0)),
            float(summary.get("average_fitness", 0.0)),
            float(summary.get(
                "median_fitness",
                summary.get("average_fitness", 0.0)
            )),
            float(summary.get("worst_fitness", 0.0)),
            float(metrics.get("distance", 0.0)),
            float(metrics.get("average_speed", 0.0)),
            float(metrics.get("upright", 0.0)),
            float(metrics.get("stability", 0.0)),
            float(metrics.get("energy", 0.0)),
            float(diversity.get("fitness_stddev", 0.0)),
            str(diversity.get("unique_morphologies", 0)),
            str(species.size()),
            str(pareto.size()),
            str(map_cells.size()),
            float(diversity.get("segment_count_mean", 0.0)),
            float(diversity.get("segment_count_stddev", 0.0)),
            float(diversity.get("brain_nodes_mean", 0.0)),
            float(diversity.get("brain_nodes_stddev", 0.0)),
        ]
    )


func _refresh_diversity_graph() -> void:
    if _results_data.is_empty() or _results_diversity_graph == null:
        return

    var result: Dictionary = _results_data.get("result", {})
    var history: Array = result.get("history", [])
    if history.is_empty():
        return

    var morphology_values: Array[float] = []
    var class_values: Array[float] = []
    var maximum := 1.0

    for summary_value in history:
        if typeof(summary_value) != TYPE_DICTIONARY:
            continue
        var summary: Dictionary = summary_value
        var diversity: Dictionary = summary.get("diversity", {})
        var morphology_count := float(
            diversity.get("unique_morphologies", 0)
        )
        var class_count := float(diversity.get("analysis_species_count", 0))
        morphology_values.append(morphology_count)
        class_values.append(class_count)
        maximum = maxf(maximum, maxf(morphology_count, class_count))

    if morphology_values.is_empty():
        return

    var graph_size := _results_diversity_graph.size
    if graph_size.x < 100.0:
        graph_size.x = 900.0
    if graph_size.y < 100.0:
        graph_size.y = 180.0

    var left := 12.0
    var right := graph_size.x - 12.0
    var top := 12.0
    var bottom := graph_size.y - 12.0
    var denominator := maxf(float(morphology_values.size() - 1), 1.0)

    _results_morphology_line.clear_points()
    _results_class_line.clear_points()

    for index in range(morphology_values.size()):
        var x := lerpf(left, right, float(index) / denominator)
        var morphology_y := lerpf(
            bottom,
            top,
            morphology_values[index] / maximum
        )
        var class_y := lerpf(bottom, top, class_values[index] / maximum)
        _results_morphology_line.add_point(Vector2(x, morphology_y))
        _results_class_line.add_point(Vector2(x, class_y))


func _result_lineage_record_at(flat_index: int) -> Dictionary:
    if flat_index < 0 or _results_data.is_empty():
        return {}

    var result_value = _results_data.get("result", {})
    if typeof(result_value) != TYPE_DICTIONARY:
        return {}
    var result: Dictionary = result_value
    var history: Array = result.get("history", [])
    var remaining := flat_index

    for summary_value in history:
        if typeof(summary_value) != TYPE_DICTIONARY:
            continue
        var summary: Dictionary = summary_value
        var lineage: Array = summary.get("lineage", [])
        for record_value in lineage:
            if typeof(record_value) != TYPE_DICTIONARY:
                continue
            if remaining == 0:
                return record_value
            remaining -= 1

    return {}


func _result_lineage_record_by_id(individual_id: int) -> Dictionary:
    if _results_data.is_empty():
        return {}

    var result: Dictionary = _results_data.get("result", {})
    var history: Array = result.get("history", [])
    for summary_value in history:
        if typeof(summary_value) != TYPE_DICTIONARY:
            continue
        var summary: Dictionary = summary_value
        var lineage: Array = summary.get("lineage", [])
        for record_value in lineage:
            if (
                typeof(record_value) == TYPE_DICTIONARY
                and int(record_value.get("individual_id", -1)) == individual_id
            ):
                return record_value

    return {}


func _result_generation_summary(generation: int) -> Dictionary:
    if _results_data.is_empty():
        return {}

    var result: Dictionary = _results_data.get("result", {})
    var history: Array = result.get("history", [])
    for summary_value in history:
        if (
            typeof(summary_value) == TYPE_DICTIONARY
            and int(summary_value.get("generation", -1)) == generation
        ):
            return summary_value

    return {}


func _format_ancestry(record: Dictionary) -> String:
    if record.is_empty():
        return "-"

    var lines: PackedStringArray = []
    var frontier: Array = [{
        "id": int(record.get("individual_id", 0)),
        "depth": 0,
    }]
    var seen: Dictionary = {}

    while not frontier.is_empty() and lines.size() < 64:
        var item: Dictionary = frontier.pop_front()
        var individual_id := int(item.get("id", 0))
        var depth := int(item.get("depth", 0))
        if seen.has(individual_id):
            continue
        seen[individual_id] = true

        var ancestor := _result_lineage_record_by_id(individual_id)
        if ancestor.is_empty():
            lines.append("%s#%s" % ["  ".repeat(depth), str(individual_id)])
            continue

        lines.append(
            "%s#%s  G%s  fit %.4f"
            % [
                "  ".repeat(depth),
                str(individual_id),
                str(ancestor.get("generation", 0)),
                float(ancestor.get("fitness", 0.0)),
            ]
        )

        if depth >= 6:
            continue

        var parents = ancestor.get("parent_ids", [])
        if typeof(parents) != TYPE_ARRAY:
            continue
        for parent_id in parents:
            frontier.append({
                "id": int(parent_id),
                "depth": depth + 1,
            })

    return "\n".join(lines)


func _format_mutation_records(value) -> String:
    if typeof(value) != TYPE_ARRAY:
        return "None recorded"
    var mutations: Array = value
    if mutations.is_empty():
        return "None (elite copy / original ancestor)"

    var lines: PackedStringArray = []
    for mutation_value in mutations:
        if typeof(mutation_value) != TYPE_DICTIONARY:
            continue
        var mutation: Dictionary = mutation_value
        var description := str(mutation.get("description", ""))
        if description.is_empty():
            description = str(mutation.get("kind", "Mutation"))
        lines.append("• " + description)

    return "\n".join(lines) if not lines.is_empty() else "None recorded"


func _format_trial_seeds(value) -> String:
    if typeof(value) != TYPE_ARRAY:
        return "-"
    var seeds: Array = value
    if seeds.is_empty():
        return "-"
    var parts: PackedStringArray = []
    for seed_value in seeds:
        parts.append(str(seed_value))
    return ", ".join(parts)


func _on_result_lineage_selected(index: int) -> void:
    var record := _result_lineage_record_at(index)
    if record.is_empty():
        _results_lineage_detail.text = ""
        return

    var metrics: Dictionary = record.get("metrics", {})
    _results_lineage_detail.text = (
        "[b]Creature #%s[/b]\n"
        + "Generation %s • parents [%s]\n\n"
        + "[table=2]"
        + "[cell]Fitness[/cell][cell][b]%.5f[/b][/cell]"
        + "[cell]Analysis class[/cell][cell]%s[/cell]"
        + "[cell]Distance[/cell][cell]%.4f m[/cell]"
        + "[cell]Speed[/cell][cell]%.4f m/s[/cell]"
        + "[cell]Upright[/cell][cell]%.3f[/cell]"
        + "[cell]Stability[/cell][cell]%.3f[/cell]"
        + "[cell]Energy[/cell][cell]%.4f[/cell]"
        + "[cell]Segments / joints[/cell][cell]%s / %s[/cell]"
        + "[cell]Brain nodes[/cell][cell]%s[/cell]"
        + "[cell]Trial seeds[/cell][cell]%s[/cell]"
        + "[/table]\n\n"
        + "[b]Mutations from parent[/b]\n%s\n\n"
        + "[b]Ancestry (up to 6 generations)[/b]\n%s"
        % [
            str(record.get("individual_id", 0)),
            str(record.get("generation", 0)),
            _format_parent_ids(record.get("parent_ids", [])),
            float(record.get("fitness", 0.0)),
            str(record.get("species_id", 0)),
            float(metrics.get("distance", 0.0)),
            float(metrics.get("average_speed", 0.0)),
            float(metrics.get("upright", 0.0)),
            float(metrics.get("stability", 0.0)),
            float(metrics.get("energy", 0.0)),
            str(record.get("segments", 0)),
            str(record.get("joints", 0)),
            str(record.get("brain_nodes", 0)),
            _format_trial_seeds(record.get("trial_seeds", [])),
            _format_mutation_records(record.get("mutations", [])),
            _format_ancestry(record),
        ]
    )


func _selected_lineage_record() -> Dictionary:
    var selected := _results_lineage_list.get_selected_items()
    if selected.is_empty():
        return {}
    return _result_lineage_record_at(int(selected[0]))


func _on_replay_lineage_creature() -> void:
    if _job_pid > 0:
        return

    var record := _selected_lineage_record()
    if record.is_empty():
        return

    var genome_value = record.get("genome", {})
    if typeof(genome_value) != TYPE_DICTIONARY:
        _set_status("This results file does not retain that creature genome.")
        return

    var summary := _result_generation_summary(int(record.get("generation", 0)))
    var settings: Dictionary = summary.get("effective_settings", {})
    var simulation: Dictionary = settings.get("simulation", {})
    var world: Dictionary = simulation.get("world", {})

    var temp_path := _runtime_path("historical_replay_creature.json")
    if not _write_genome_file(temp_path, genome_value):
        _set_status("Could not prepare historical creature replay.")
        return

    _current_genome = genome_value.duplicate(true)
    _current_genome_source = "generation %s creature #%s" % [
        str(record.get("generation", 0)),
        str(record.get("individual_id", 0)),
    ]

    _prepare_creature_run()
    var args := PackedStringArray([
        "creature-stream",
        "--event-port", str(_event_port),
        "--seconds", str(float(
            simulation.get("duration_seconds", _seconds_spin.value)
        )),
        "--dt", str(float(simulation.get("dt", _dt_spin.value))),
        "--frame-hz", "60",
        "--playback-speed", "%.2f" % _playback_speed,
        "--seed", "1",
        "--max-segments", str(maxi(
            int(_max_segments_spin.value),
            int(record.get("segments", 2))
        )),
        "--world-file", _write_runtime_json("runtime_replay_world.json", world),
        "--motor-strength", str(float(
            simulation.get("motor_strength_multiplier", 1.0)
        )),
        "--genome", temp_path,
    ])

    _results_window.hide()
    if _start_job("creature", args):
        _set_status(
            "Replaying generation %s creature #%s in its recorded environment..."
            % [
                str(record.get("generation", 0)),
                str(record.get("individual_id", 0)),
            ]
        )


func _on_load_lineage_creature() -> void:
    var record := _selected_lineage_record()
    if record.is_empty():
        return

    var genome_value = record.get("genome", {})
    if typeof(genome_value) != TYPE_DICTIONARY:
        _set_status("This results file does not retain that creature genome.")
        return

    _current_genome = genome_value.duplicate(true)
    _current_genome_source = "generation %s creature #%s" % [
        str(record.get("generation", 0)),
        str(record.get("individual_id", 0)),
    ]
    _save_button.disabled = false
    _probe_mesh.visible = false
    _build_creature_from_genome(_current_genome)
    _results_window.hide()
    _set_status("Loaded %s." % _current_genome_source)


func _on_compare_lineage_creature() -> void:
    var record := _selected_lineage_record()
    if record.is_empty():
        return

    var result: Dictionary = _results_data.get("result", {})
    var final_metrics: Dictionary = result.get("champion_metrics", {})
    var selected_metrics: Dictionary = record.get("metrics", {})

    _results_lineage_detail.text = (
        "[b]Creature #%s vs final champion[/b]\n\n"
        + "[table=3]"
        + "[cell][/cell][cell][b]Selected[/b][/cell][cell][b]Final[/b][/cell]"
        + "[cell]Fitness[/cell][cell]%.5f[/cell][cell]%.5f[/cell]"
        + "[cell]Distance[/cell][cell]%.4f[/cell][cell]%.4f[/cell]"
        + "[cell]Speed[/cell][cell]%.4f[/cell][cell]%.4f[/cell]"
        + "[cell]Upright[/cell][cell]%.3f[/cell][cell]%.3f[/cell]"
        + "[cell]Stability[/cell][cell]%.3f[/cell][cell]%.3f[/cell]"
        + "[cell]Energy[/cell][cell]%.4f[/cell][cell]%.4f[/cell]"
        + "[/table]\n\n"
        + "[b]Selected mutations[/b]\n%s"
        % [
            str(record.get("individual_id", 0)),
            float(record.get("fitness", 0.0)),
            float(result.get("champion_fitness", 0.0)),
            float(selected_metrics.get("distance", 0.0)),
            float(final_metrics.get("distance", 0.0)),
            float(selected_metrics.get("average_speed", 0.0)),
            float(final_metrics.get("average_speed", 0.0)),
            float(selected_metrics.get("upright", 0.0)),
            float(final_metrics.get("upright", 0.0)),
            float(selected_metrics.get("stability", 0.0)),
            float(final_metrics.get("stability", 0.0)),
            float(selected_metrics.get("energy", 0.0)),
            float(final_metrics.get("energy", 0.0)),
            _format_mutation_records(record.get("mutations", [])),
        ]
    )


func _on_fork_lineage_creature() -> void:
    var record := _selected_lineage_record()
    if record.is_empty():
        return

    var genome_value = record.get("genome", {})
    if typeof(genome_value) != TYPE_DICTIONARY:
        _set_status("This results file does not retain that creature genome.")
        return

    _current_genome = genome_value.duplicate(true)
    _current_genome_source = "generation %s creature #%s" % [
        str(record.get("generation", 0)),
        str(record.get("individual_id", 0)),
    ]
    _experiment_name = str(
        _results_data.get("experiment_name", _experiment_name)
    )
    _results_window.hide()
    _on_fork_experiment_pressed()


func _on_result_champion_selected(_index: int) -> void:
    pass


func _on_load_archived_champion() -> void:
    var selected := _results_champion_list.get_selected_items()
    if selected.is_empty() or _results_data.is_empty():
        return

    var result: Dictionary = _results_data.get("result", {})
    var archive: Array = result.get("champion_archive", [])
    var index := int(selected[0])
    if index < 0 or index >= archive.size():
        return
    if typeof(archive[index]) != TYPE_DICTIONARY:
        return

    var entry: Dictionary = archive[index]
    var genome_value = entry.get("genome", {})
    if typeof(genome_value) != TYPE_DICTIONARY:
        return

    _current_genome = genome_value.duplicate(true)
    _current_genome_source = "generation %d archived champion" % int(
        entry.get("generation", 0)
    )
    _save_button.disabled = false
    _probe_mesh.visible = false
    _build_creature_from_genome(_current_genome)
    _results_window.hide()
    _set_status(
        "Loaded generation %d champion (#%s)."
        % [int(entry.get("generation", 0)), str(entry.get("individual_id", 0))]
    )


func _on_compare_archived_champion() -> void:
    var selected := _results_champion_list.get_selected_items()
    if selected.is_empty() or _results_data.is_empty():
        return

    var result: Dictionary = _results_data.get("result", {})
    var archive: Array = result.get("champion_archive", [])
    var index := int(selected[0])
    if index < 0 or index >= archive.size():
        return

    var selected_entry: Dictionary = archive[index]
    var final_fitness := float(result.get("champion_fitness", 0.0))
    var final_metrics: Dictionary = result.get("champion_metrics", {})
    var selected_metrics: Dictionary = selected_entry.get("metrics", {})

    _results_analysis.text = (
        "[b]Selected vs final champion[/b]

"
        + "[table=3]"
        + "[cell][/cell][cell][b]Selected G%d[/b][/cell][cell][b]Final[/b][/cell]"
        + "[cell]Fitness[/cell][cell]%.5f[/cell][cell]%.5f[/cell]"
        + "[cell]Distance[/cell][cell]%.4f[/cell][cell]%.4f[/cell]"
        + "[cell]Speed[/cell][cell]%.4f[/cell][cell]%.4f[/cell]"
        + "[cell]Upright[/cell][cell]%.3f[/cell][cell]%.3f[/cell]"
        + "[cell]Stability[/cell][cell]%.3f[/cell][cell]%.3f[/cell]"
        + "[cell]Energy[/cell][cell]%.4f[/cell][cell]%.4f[/cell]"
        + "[/table]

"
        + "[color=#9aa7bd]Analysis classes below are descriptive morphology/brain buckets, "
        + "not yet evolutionary speciation.[/color]"
        % [
            int(selected_entry.get("generation", 0)),
            float(selected_entry.get("fitness", 0.0)),
            final_fitness,
            float(selected_metrics.get("distance", 0.0)),
            float(final_metrics.get("distance", 0.0)),
            float(selected_metrics.get("average_speed", 0.0)),
            float(final_metrics.get("average_speed", 0.0)),
            float(selected_metrics.get("upright", 0.0)),
            float(final_metrics.get("upright", 0.0)),
            float(selected_metrics.get("stability", 0.0)),
            float(final_metrics.get("stability", 0.0)),
            float(selected_metrics.get("energy", 0.0)),
            float(final_metrics.get("energy", 0.0)),
        ]
    )


func _refresh_results_analysis() -> void:
    if _results_data.is_empty():
        return

    var result: Dictionary = _results_data.get("result", {})
    var history: Array = result.get("history", [])
    if history.is_empty():
        return

    var final_summary: Dictionary = history.back()
    var diversity: Dictionary = final_summary.get("diversity", {})
    var pareto: Array = final_summary.get("pareto_front", [])
    var map_cells: Array = final_summary.get("map_elites", [])
    var species: Array = final_summary.get("species", [])

    var best_species_text := "none"
    if not species.is_empty() and typeof(species[0]) == TYPE_DICTIONARY:
        var best_species: Dictionary = species[0]
        for species_value in species:
            if (
                typeof(species_value) == TYPE_DICTIONARY
                and float(species_value.get("best_fitness", -INF))
                    > float(best_species.get("best_fitness", -INF))
            ):
                best_species = species_value
        best_species_text = (
            "%s (%s members, best %.4f)"
            % [
                str(best_species.get("species_id", 0)),
                str(best_species.get("members", 0)),
                float(best_species.get("best_fitness", 0.0)),
            ]
        )

    _results_analysis.text = (
        "[b]%s[/b]
"
        + "Generations: %s   Evaluations: %s   Wall time: %.3f s

"
        + "[b]Final-generation diversity[/b]
"
        + "[table=2]"
        + "[cell]Fitness σ[/cell][cell]%.5f[/cell]"
        + "[cell]Unique morphologies[/cell][cell]%s[/cell]"
        + "[cell]Analysis classes[/cell][cell]%s[/cell]"
        + "[cell]Best class[/cell][cell]%s[/cell]"
        + "[cell]Segment mean ± σ[/cell][cell]%.2f ± %.2f[/cell]"
        + "[cell]Brain-node mean ± σ[/cell][cell]%.2f ± %.2f[/cell]"
        + "[cell]Distance/energy Pareto front[/cell][cell]%s[/cell]"
        + "[cell]MAP-Elites occupied cells[/cell][cell]%s[/cell]"
        + "[/table]

"
        + "[color=#9aa7bd]Analysis classes group creatures by segment count and "
        + "brain-node bucket. They provide species-style browsing groundwork; "
        + "selection is not yet using speciation. MAP-Elites here is analysis-only "
        + "occupancy, not yet the evolution algorithm.[/color]"
        % [
            str(_results_data.get("experiment_name", "Evolution Results")),
            str(result.get("generations_completed", 0)),
            str(result.get("evaluations_completed", 0)),
            float(_results_data.get("wall_seconds", 0.0)),
            float(diversity.get("fitness_stddev", 0.0)),
            str(diversity.get("unique_morphologies", 0)),
            str(species.size()),
            best_species_text,
            float(diversity.get("segment_count_mean", 0.0)),
            float(diversity.get("segment_count_stddev", 0.0)),
            float(diversity.get("brain_nodes_mean", 0.0)),
            float(diversity.get("brain_nodes_stddev", 0.0)),
            str(pareto.size()),
            str(map_cells.size()),
        ]
    )


func _format_parent_ids(value) -> String:
    if typeof(value) != TYPE_ARRAY:
        return "-"
    var ids: Array = value
    if ids.is_empty():
        return "-"
    var parts: PackedStringArray = []
    for id_value in ids:
        parts.append(str(id_value))
    return ", ".join(parts)


func _build_walkthrough_window() -> void:
    _walkthrough_window = Window.new()
    _walkthrough_window.title = "Guided First Test"
    _walkthrough_window.size = Vector2i(620, 470)
    _walkthrough_window.min_size = Vector2i(520, 400)
    _walkthrough_window.visible = false
    _walkthrough_window.theme = _ui_theme
    _walkthrough_window.close_requested.connect(_walkthrough_window.hide)
    add_child(_walkthrough_window)

    var margin := MarginContainer.new()
    margin.add_theme_constant_override("margin_left", 22)
    margin.add_theme_constant_override("margin_right", 22)
    margin.add_theme_constant_override("margin_top", 20)
    margin.add_theme_constant_override("margin_bottom", 18)
    margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
    _walkthrough_window.add_child(margin)

    var column := VBoxContainer.new()
    column.add_theme_constant_override("separation", 12)
    margin.add_child(column)

    var title := Label.new()
    title.text = "First Test Walkthrough"
    title.add_theme_font_size_override("font_size", _font_size + 5)
    column.add_child(title)

    _walkthrough_text = RichTextLabel.new()
    _walkthrough_text.bbcode_enabled = true
    _walkthrough_text.fit_content = false
    _walkthrough_text.size_flags_vertical = Control.SIZE_EXPAND_FILL
    column.add_child(_walkthrough_text)

    _walkthrough_run_button = Button.new()
    _walkthrough_run_button.text = "▶ Run This First Test"
    _walkthrough_run_button.pressed.connect(_on_walkthrough_run_test)
    column.add_child(_walkthrough_run_button)

    var buttons := HBoxContainer.new()
    buttons.add_theme_constant_override("separation", 8)
    column.add_child(buttons)

    _walkthrough_back_button = Button.new()
    _walkthrough_back_button.text = "Back"
    _walkthrough_back_button.pressed.connect(_on_walkthrough_back)
    buttons.add_child(_walkthrough_back_button)

    var spacer := Control.new()
    spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    buttons.add_child(spacer)

    _walkthrough_next_button = Button.new()
    _walkthrough_next_button.text = "Next"
    _walkthrough_next_button.pressed.connect(_on_walkthrough_next)
    buttons.add_child(_walkthrough_next_button)

    _refresh_walkthrough()


func _maybe_show_first_run_walkthrough() -> void:
    # Versioned so an already-completed older walkthrough is shown once again
    # when the walkthrough itself changes materially.
    if _walkthrough_version < 2:
        _walkthrough_completed = false
        _show_walkthrough()


func _show_walkthrough() -> void:
    _walkthrough_step = 0
    _refresh_walkthrough()
    _walkthrough_window.popup_centered()


func _refresh_walkthrough() -> void:
    if _walkthrough_text == null:
        return

    match _walkthrough_step:
        0:
            _walkthrough_text.text = (
                "[b]What this app is doing[/b]\n\n"
                + "The left HUD controls the experiment. The 3D area shows the exact world and "
                + "the creature being simulated. The floor should always be visible.\n\n"
                + "For the first run we will use the built-in three-segment seed creature so "
                + "there are no random-generation variables to confuse the test."
            )
        1:
            _walkthrough_text.text = (
                "[b]The safe first-test settings[/b]\n\n"
                + "The guide will set: [b]Seed 1[/b], [b]Flat terrain[/b], "
                + "[b]8 seconds[/b], [b]1.00x playback[/b], and [b]CPU[/b].\n\n"
                + "CPU is chosen only for this sanity check because every supported machine has it. "
                + "After this works, Benchmark can be used to compare CPU/CUDA throughput."
            )
        2:
            _walkthrough_text.text = (
                "[b]Run the first creature test[/b]\n\n"
                + "Click [b]Run This First Test[/b] below. The app will apply the simple settings "
                + "and start the known seed creature.\n\n"
                + "Expected result: you should see the floor, a multi-part creature, and smooth motion "
                + "in the 3D view. The progress/status area will update while it runs."
            )
        3:
            _walkthrough_text.text = (
                "[b]While it runs[/b]\n\n"
                + "Watch the 3D view first. The floor is the world collision surface. The colored boxes "
                + "are creature body segments connected by joints.\n\n"
                + "Right-drag looks around. W/S move along the camera aim and A/D strafe. "
                + "Playback speed changes how quickly you watch the buffered simulation, not the physics rules."
            )
        _:
            _walkthrough_text.text = (
                "[b]Reading the result[/b]\n\n"
                + "Open [b]Status / Results[/b] in the HUD. It shows progress and metrics from the run. "
                + "The [b]Quick Test / Main Controls[/b] section is where you can rerun the seed creature, "
                + "try the Single Box physics check, run a Benchmark, or start Evolution.\n\n"
                + "The detailed sections stay collapsed until you need to change advanced settings."
            )

    _walkthrough_back_button.disabled = _walkthrough_step == 0
    _walkthrough_run_button.visible = _walkthrough_step == 2
    _walkthrough_next_button.text = "Finish" if _walkthrough_step >= 4 else "Next"


func _on_walkthrough_back() -> void:
    _walkthrough_step = maxi(0, _walkthrough_step - 1)
    _refresh_walkthrough()


func _on_walkthrough_next() -> void:
    if _walkthrough_step >= 4:
        _walkthrough_completed = true
        _walkthrough_version = 2
        _save_settings()
        _walkthrough_window.hide()
        return
    _walkthrough_step += 1
    _refresh_walkthrough()


func _on_walkthrough_run_test() -> void:
    if _job_pid > 0:
        _set_status("Stop the current run before starting the guided first test.")
        return

    _seed_spin.value = 1
    _seconds_spin.value = 8.0
    _playback_speed_spin.value = 1.0
    _playback_speed = 1.0
    _terrain_kind = "flat"
    _terrain_option.select(0)
    _walls_check.button_pressed = false
    _blocks_check.button_pressed = false
    _gaps_check.button_pressed = false
    _pits_check.button_pressed = false
    _walls_enabled = false
    _blocks_enabled = false
    _gaps_enabled = false
    _pits_enabled = false
    _accelerator_mode = "cpu"
    _accelerator_option.select(0)
    _save_settings()
    _refresh_world_preview()

    _walkthrough_step = 3
    _refresh_walkthrough()
    _walkthrough_guided_run_active = true
    _walkthrough_window.hide()
    _set_status("Guided first test starting • watch the 3D view and Status / Results.")
    _on_seed_creature_pressed()


func _build_settings_window() -> void:
    _settings_window = Window.new()
    _settings_window.title = "Settings"
    _settings_window.size = Vector2i(430, 310)
    _settings_window.min_size = Vector2i(360, 260)
    _settings_window.visible = false
    _settings_window.theme = _ui_theme
    _settings_window.close_requested.connect(_settings_window.hide)
    add_child(_settings_window)

    var margin := MarginContainer.new()
    margin.add_theme_constant_override("margin_left", 18)
    margin.add_theme_constant_override("margin_right", 18)
    margin.add_theme_constant_override("margin_top", 18)
    margin.add_theme_constant_override("margin_bottom", 18)
    margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
    _settings_window.add_child(margin)

    var column := VBoxContainer.new()
    column.add_theme_constant_override("separation", 12)
    margin.add_child(column)

    var heading := Label.new()
    heading.text = "Interface & Camera"
    column.add_child(heading)

    _font_size_spin = _add_number_row(column, "Font size", 7, 20, _font_size, 1)
    _font_size_spin.value_changed.connect(_on_font_size_changed)

    _camera_speed_spin = _add_number_row(
        column,
        "WASD move speed",
        0.5,
        50.0,
        _camera_move_speed,
        0.5
    )
    _camera_speed_spin.value_changed.connect(_on_camera_speed_changed)

    _mouse_sensitivity_spin = _add_number_row(
        column,
        "Mouse sensitivity",
        0.03,
        1.0,
        _mouse_sensitivity_degrees,
        0.01
    )
    _mouse_sensitivity_spin.value_changed.connect(_on_mouse_sensitivity_changed)

    var help := Label.new()
    help.text = "Hold right mouse and move to look. W/S move along your aim; A/D strafe. Drag the thin bar on the HUD's right edge to resize it; the width is saved automatically."
    help.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    help.modulate = Color(0.70, 0.76, 0.86)
    column.add_child(help)


func _on_settings_pressed() -> void:
    _set_mouse_look(false)
    _settings_window.popup_centered()


func _on_font_size_changed(value: float) -> void:
    _font_size = int(value)
    _apply_font_size()
    _save_settings()


func _on_camera_speed_changed(value: float) -> void:
    _camera_move_speed = float(value)
    _save_settings()


func _on_mouse_sensitivity_changed(value: float) -> void:
    _mouse_sensitivity_degrees = float(value)
    _save_settings()


func _on_playback_speed_changed(value: float) -> void:
    _playback_speed = clampf(float(value), 0.01, 2.0)
    _save_settings()


func _on_fitness_weights_changed(_value: float) -> void:
    _fitness_distance_weight = float(_fitness_distance_spin.value)
    _fitness_speed_weight = float(_fitness_speed_spin.value)
    _fitness_upright_weight = float(_fitness_upright_spin.value)
    _fitness_stability_weight = float(_fitness_stability_spin.value)
    _fitness_energy_weight = float(_fitness_energy_spin.value)
    _save_settings()


func _on_schedule_base_changed(_value: float) -> void:
    _structural_mutation_chance = float(_structural_mutation_spin.value)
    _motor_strength = float(_motor_strength_spin.value)
    _trials_per_creature = int(_trials_spin.value)
    _save_settings()


func _on_trial_aggregation_selected(index: int) -> void:
    var values := ["mean", "median", "worst", "best"]
    if index >= 0 and index < values.size():
        _trial_aggregation = values[index]
    _save_settings()


func _on_timeline_condition_selected(index: int) -> void:
    _timeline_condition_value_spin.editable = index != 0


func _fitness_config_dictionary() -> Dictionary:
    return {
        "weights": {
            "distance": float(_fitness_distance_spin.value),
            "average_speed": float(_fitness_speed_spin.value),
            "upright": float(_fitness_upright_spin.value),
            "stability": float(_fitness_stability_spin.value),
            "energy": float(_fitness_energy_spin.value),
        }
    }


func _trial_aggregation_value() -> String:
    if _trial_aggregation_option == null:
        return _trial_aggregation
    var values := ["mean", "median", "worst", "best"]
    var index := _trial_aggregation_option.selected
    if index >= 0 and index < values.size():
        return values[index]
    return "mean"


func _timeline_snapshot_changes() -> Dictionary:
    _sync_world_state_from_controls()
    return {
        "world": _world_config_dictionary(),
        "fitness": _fitness_config_dictionary(),
        "population_size": int(_population_spin.value),
        "mutations_per_child": int(_evolution_mutations_spin.value),
        "structural_mutation_chance": float(_structural_mutation_spin.value),
        "motor_strength_multiplier": float(_motor_strength_spin.value),
        "trial_duration_seconds": float(_seconds_spin.value),
        "trials_per_creature": int(_trials_spin.value),
        "trial_aggregation": _trial_aggregation_value(),
    }


func _timeline_condition_dictionary() -> Variant:
    var index := _timeline_condition_option.selected
    if index == 0:
        return null

    var kinds := [
        "",
        "best_fitness_at_least",
        "average_fitness_at_least",
        "best_distance_at_least",
    ]
    return {
        "kind": kinds[index],
        "value": float(_timeline_condition_value_spin.value),
    }


func _on_add_timeline_keyframe() -> void:
    var entry := {
        "id": "timeline_%d" % _timeline_next_id,
        "generation": int(_timeline_generation_spin.value),
        "condition": _timeline_condition_dictionary(),
        "changes": _timeline_snapshot_changes(),
    }
    _timeline_next_id += 1
    _timeline_entries.append(entry)
    _sort_timeline_entries()
    _refresh_timeline_list()
    _save_settings()
    _set_status(
        "Timeline keyframe added at generation %d."
        % int(entry.get("generation", 1))
    )


func _on_remove_timeline_keyframe() -> void:
    var selected := _timeline_list.get_selected_items()
    if selected.is_empty():
        return
    var index := int(selected[0])
    if index >= 0 and index < _timeline_entries.size():
        _timeline_entries.remove_at(index)
        _refresh_timeline_list()
        _save_settings()


func _on_clear_timeline() -> void:
    _timeline_entries.clear()
    _refresh_timeline_list()
    _save_settings()


func _sort_timeline_entries() -> void:
    _timeline_entries.sort_custom(
        func(a, b):
            return int(a.get("generation", 1)) < int(b.get("generation", 1))
    )


func _refresh_timeline_list() -> void:
    if _timeline_list == null:
        return

    _timeline_list.clear()
    for entry_value in _timeline_entries:
        if typeof(entry_value) != TYPE_DICTIONARY:
            continue
        var entry: Dictionary = entry_value
        var generation := int(entry.get("generation", 1))
        var condition_value = entry.get("condition", null)
        var activation := "Gen %d" % generation
        if typeof(condition_value) == TYPE_DICTIONARY:
            var condition: Dictionary = condition_value
            var condition_name := str(condition.get("kind", ""))
            var threshold := float(condition.get("value", 0.0))
            match condition_name:
                "best_fitness_at_least":
                    activation = "Gen %d+ • best fitness ≥ %.3f" % [generation, threshold]
                "average_fitness_at_least":
                    activation = "Gen %d+ • average fitness ≥ %.3f" % [generation, threshold]
                "best_distance_at_least":
                    activation = "Gen %d+ • best distance ≥ %.3f" % [generation, threshold]

        var changes: Dictionary = entry.get("changes", {})
        var world: Dictionary = changes.get("world", {})
        _timeline_list.add_item(
            "%s  |  %s  |  pop %s  |  trials %s"
            % [
                activation,
                str(world.get("terrain", "flat")),
                str(changes.get("population_size", "?")),
                str(changes.get("trials_per_creature", "?")),
            ]
        )


func _timeline_config_dictionary() -> Dictionary:
    return {"keyframes": _timeline_entries.duplicate(true)}


func _timeline_json() -> String:
    return JSON.stringify(_timeline_config_dictionary())


func _on_accelerator_selected(index: int) -> void:
    var values := ["cpu", "auto", "cuda"]
    if index >= 0 and index < values.size():
        _accelerator_mode = values[index]
    _save_settings()


func _on_gpu_ids_changed(value: String) -> void:
    _gpu_ids = value.strip_edges()
    _save_settings()


func _on_accelerator_number_changed(_value: float) -> void:
    _gpu_batch_size = int(_gpu_batch_spin.value)
    _gpu_max_parts = int(_gpu_max_parts_spin.value)
    _gpu_max_joints = int(_gpu_max_joints_spin.value)
    _save_settings()


func _on_cpu_fallback_toggled(value: bool) -> void:
    _cpu_fallback = value
    _save_settings()


func _on_throughput_selected(index: int) -> void:
    _throughput_mode = "deterministic" if index == 0 else "max"
    _save_settings()


func _gpu_ids_for_cli() -> String:
    var value := _gpu_ids_edit.text.strip_edges() if _gpu_ids_edit != null else _gpu_ids.strip_edges()
    return "all" if value.is_empty() else value


func _accelerator_cli_args() -> PackedStringArray:
    var args := PackedStringArray([
        "--accelerator", _accelerator_mode,
        "--gpus", _gpu_ids_for_cli(),
        "--gpu-batch-size", str(int(_gpu_batch_spin.value)),
        "--gpu-max-parts", str(int(_gpu_max_parts_spin.value)),
        "--gpu-max-joints", str(int(_gpu_max_joints_spin.value)),
        "--throughput-mode", _throughput_mode,
    ])
    if not _cpu_fallback_check.button_pressed:
        args.append("--no-cpu-fallback")
    return args


func _on_terrain_selected(index: int) -> void:
    var terrain_names := ["flat", "slope", "hills", "stairs"]
    if index >= 0 and index < terrain_names.size():
        _terrain_kind = terrain_names[index]
    _save_settings()
    _refresh_world_preview()


func _on_world_numeric_changed(_value: float) -> void:
    _sync_world_state_from_controls()
    _save_settings()
    _refresh_world_preview()


func _on_world_toggle_changed(_value: bool) -> void:
    _sync_world_state_from_controls()
    _save_settings()
    _refresh_world_preview()


func _sync_world_state_from_controls() -> void:
    if _world_seed_spin == null:
        return
    _world_seed = int(_world_seed_spin.value)
    _gravity_x = float(_gravity_x_spin.value)
    _gravity_y = float(_gravity_y_spin.value)
    _gravity_z = float(_gravity_z_spin.value)
    _ground_friction = float(_ground_friction_spin.value)
    _slope_degrees = float(_slope_spin.value)
    _hill_height = float(_hill_height_spin.value)
    _hill_wavelength = float(_hill_wavelength_spin.value)
    _stair_height = float(_stair_height_spin.value)
    _stair_depth = float(_stair_depth_spin.value)
    _walls_enabled = _walls_check.button_pressed
    _blocks_enabled = _blocks_check.button_pressed
    _gaps_enabled = _gaps_check.button_pressed
    _pits_enabled = _pits_check.button_pressed
    _obstacle_count = int(_obstacle_count_spin.value)
    _obstacle_spacing = float(_obstacle_spacing_spin.value)
    _obstacle_size = float(_obstacle_size_spin.value)
    _gap_width = float(_gap_width_spin.value)
    _pit_depth = float(_pit_depth_spin.value)


func _world_config_dictionary() -> Dictionary:
    return {
        "gravity": [_gravity_x, _gravity_y, _gravity_z],
        "ground_half_extents": [50.0, 0.1, 12.0],
        "ground_friction": _ground_friction,
        "terrain": _terrain_kind,
        "seed": _world_seed,
        "slope_degrees": _slope_degrees,
        "hill_height": _hill_height,
        "hill_wavelength": _hill_wavelength,
        "stair_height": _stair_height,
        "stair_depth": _stair_depth,
        "walls_enabled": _walls_enabled,
        "blocks_enabled": _blocks_enabled,
        "gaps_enabled": _gaps_enabled,
        "pits_enabled": _pits_enabled,
        "obstacle_count": _obstacle_count,
        "obstacle_spacing": _obstacle_spacing,
        "obstacle_size": _obstacle_size,
        "gap_width": _gap_width,
        "pit_depth": _pit_depth,
    }


func _world_json() -> String:
    _sync_world_state_from_controls()
    return JSON.stringify(_world_config_dictionary())


func _write_runtime_json(file_name: String, value) -> String:
    _ensure_portable_directories()
    var path := _runtime_path(file_name)
    var file := FileAccess.open(path, FileAccess.WRITE)
    if file == null:
        return ""
    var normalized = _normalize_integral_json_numbers(value)
    file.store_string(JSON.stringify(normalized))
    file.close()
    return path


func _world_file_for_cli(world_override = null) -> String:
    _sync_world_state_from_controls()
    var world := _world_config_dictionary()
    if typeof(world_override) == TYPE_DICTIONARY and not world_override.is_empty():
        world = world_override
    return _write_runtime_json("runtime_world.json", world)


func _timeline_file_for_cli() -> String:
    return _write_runtime_json("runtime_timeline.json", _timeline_config_dictionary())


func _refresh_world_preview() -> void:
    _render_world_config(_world_config_dictionary())


func _render_world_config(world_config: Dictionary) -> void:
    var world_json := JSON.stringify(world_config)
    if world_json == _rendered_world_json and not _world_meshes.is_empty():
        return

    # Never leave the viewer as an empty void if the backend is missing,
    # starting, or returns invalid geometry.
    if _world_meshes.is_empty():
        _build_world_fallback(world_config)

    if not _backend_exists():
        return

    var world_file := _write_runtime_json("runtime_world_preview.json", world_config)
    if world_file.is_empty():
        return

    var output: Array = []
    var exit_code := OS.execute(
        _backend_path(),
        PackedStringArray([
            "world-geometry",
            "--world-file",
            world_file,
        ]),
        output,
        true,
        false
    )
    if exit_code != 0 or output.is_empty():
        return

    var parsed = JSON.parse_string(str(output[0]).strip_edges())
    if typeof(parsed) != TYPE_DICTIONARY:
        return

    var geometry_value = parsed.get("geometry", [])
    if typeof(geometry_value) != TYPE_ARRAY or geometry_value.is_empty():
        return

    _build_world_from_geometry(geometry_value)
    _rendered_world_json = world_json


func _build_world_fallback(world_config: Dictionary) -> void:
    var half_value = world_config.get("ground_half_extents", [50.0, 0.1, 12.0])
    var half := [50.0, 0.1, 12.0]
    if typeof(half_value) == TYPE_ARRAY and half_value.size() >= 3:
        half = [
            float(half_value[0]),
            float(half_value[1]),
            float(half_value[2]),
        ]

    var angle := 0.0
    if str(world_config.get("terrain", "flat")) == "slope":
        angle = deg_to_rad(float(world_config.get("slope_degrees", 0.0)))

    _build_world_from_geometry([
        {
            "kind": "ground",
            "center": [0.0, -float(half[1]), 0.0],
            "half_extents": half,
            "rotation_radians": [0.0, 0.0, angle],
        }
    ])


func _build_world_from_geometry(geometry_value) -> void:
    for mesh in _world_meshes:
        if is_instance_valid(mesh):
            mesh.queue_free()
    _world_meshes.clear()

    if typeof(geometry_value) != TYPE_ARRAY:
        return

    for shape_value in geometry_value:
        if typeof(shape_value) != TYPE_DICTIONARY:
            continue

        var shape: Dictionary = shape_value
        var center: Array = shape.get("center", [0.0, 0.0, 0.0])
        var half: Array = shape.get("half_extents", [1.0, 0.1, 1.0])
        var rotation: Array = shape.get("rotation_radians", [0.0, 0.0, 0.0])
        if center.size() < 3 or half.size() < 3 or rotation.size() < 3:
            continue

        var instance := MeshInstance3D.new()
        var box := BoxMesh.new()
        box.size = Vector3(
            float(half[0]) * 2.0,
            float(half[1]) * 2.0,
            float(half[2]) * 2.0
        )
        instance.mesh = box
        instance.position = Vector3(
            2.2 + float(center[0]),
            float(center[1]),
            float(center[2])
        )
        instance.rotation = Vector3(
            float(rotation[0]),
            float(rotation[1]),
            float(rotation[2])
        )

        var material := StandardMaterial3D.new()
        var kind := str(shape.get("kind", "ground"))
        match kind:
            "wall":
                material.albedo_color = Color(0.58, 0.30, 0.20)
            "block":
                material.albedo_color = Color(0.42, 0.46, 0.54)
            "pit_floor":
                material.albedo_color = Color(0.10, 0.12, 0.15)
            _:
                # Ground needs clear contrast from the near-black sky/background.
                material.albedo_color = Color(0.24, 0.28, 0.34)
        material.roughness = 0.8
        instance.material_override = material
        add_child(instance)
        _world_meshes.append(instance)


func _set_world_controls_enabled(enabled: bool) -> void:
    if _terrain_option != null:
        _terrain_option.disabled = not enabled

    for spin in [
        _world_seed_spin,
        _gravity_x_spin,
        _gravity_y_spin,
        _gravity_z_spin,
        _ground_friction_spin,
        _slope_spin,
        _hill_height_spin,
        _hill_wavelength_spin,
        _stair_height_spin,
        _stair_depth_spin,
        _obstacle_count_spin,
        _obstacle_spacing_spin,
        _obstacle_size_spin,
        _gap_width_spin,
        _pit_depth_spin,
    ]:
        if spin != null:
            spin.editable = enabled

    for check in [_walls_check, _blocks_check, _gaps_check, _pits_check]:
        if check != null:
            check.disabled = not enabled


func _apply_font_size() -> void:
    if _ui_theme == null:
        return

    _ui_theme.default_font_size = _font_size

    if is_instance_valid(_title_label):
        _title_label.add_theme_font_size_override("font_size", _font_size + 8)

    for heading in _section_headings:
        if is_instance_valid(heading):
            heading.add_theme_font_size_override("font_size", _font_size + 1)

    if is_instance_valid(_settings_window):
        _settings_window.theme = _ui_theme


func _load_settings() -> void:
    var config := ConfigFile.new()
    if config.load(_settings_path()) != OK:
        return

    _font_size = int(config.get_value("ui", "font_size", _font_size))
    _font_size = clampi(_font_size, 7, 20)
    _walkthrough_completed = bool(
        config.get_value("ui", "walkthrough_completed", _walkthrough_completed)
    )
    _walkthrough_version = int(
        config.get_value("ui", "walkthrough_version", _walkthrough_version)
    )
    _hud_width = float(config.get_value("ui", "hud_width", _hud_width))
    _hud_width = clampf(_hud_width, MIN_HUD_WIDTH, MAX_HUD_WIDTH)
    _playback_speed = float(
        config.get_value("viewer", "playback_speed", _playback_speed)
    )
    _playback_speed = clampf(_playback_speed, 0.01, 2.0)
    _fitness_distance_weight = float(
        config.get_value("fitness", "distance", _fitness_distance_weight)
    )
    _fitness_speed_weight = float(
        config.get_value("fitness", "average_speed", _fitness_speed_weight)
    )
    _fitness_upright_weight = float(
        config.get_value("fitness", "upright", _fitness_upright_weight)
    )
    _fitness_stability_weight = float(
        config.get_value("fitness", "stability", _fitness_stability_weight)
    )
    _fitness_energy_weight = float(
        config.get_value("fitness", "energy", _fitness_energy_weight)
    )
    _terrain_kind = str(config.get_value("world", "terrain", _terrain_kind))
    _world_seed = int(config.get_value("world", "seed", _world_seed))
    _gravity_x = float(config.get_value("world", "gravity_x", _gravity_x))
    _gravity_y = float(config.get_value("world", "gravity_y", _gravity_y))
    _gravity_z = float(config.get_value("world", "gravity_z", _gravity_z))
    _ground_friction = float(
        config.get_value("world", "ground_friction", _ground_friction)
    )
    _slope_degrees = float(config.get_value("world", "slope_degrees", _slope_degrees))
    _hill_height = float(config.get_value("world", "hill_height", _hill_height))
    _hill_wavelength = float(
        config.get_value("world", "hill_wavelength", _hill_wavelength)
    )
    _stair_height = float(config.get_value("world", "stair_height", _stair_height))
    _stair_depth = float(config.get_value("world", "stair_depth", _stair_depth))
    _walls_enabled = bool(config.get_value("world", "walls_enabled", _walls_enabled))
    _blocks_enabled = bool(config.get_value("world", "blocks_enabled", _blocks_enabled))
    _gaps_enabled = bool(config.get_value("world", "gaps_enabled", _gaps_enabled))
    _pits_enabled = bool(config.get_value("world", "pits_enabled", _pits_enabled))
    _obstacle_count = int(
        config.get_value("world", "obstacle_count", _obstacle_count)
    )
    _obstacle_spacing = float(
        config.get_value("world", "obstacle_spacing", _obstacle_spacing)
    )
    _obstacle_size = float(
        config.get_value("world", "obstacle_size", _obstacle_size)
    )
    _gap_width = float(config.get_value("world", "gap_width", _gap_width))
    _pit_depth = float(config.get_value("world", "pit_depth", _pit_depth))
    _motor_strength = float(
        config.get_value("evolution", "motor_strength", _motor_strength)
    )
    _motor_strength = clampf(_motor_strength, 0.0, 1.0)
    _trials_per_creature = int(
        config.get_value("evolution", "trials_per_creature", _trials_per_creature)
    )
    _trials_per_creature = clampi(_trials_per_creature, 1, 100)
    _trial_aggregation = str(
        config.get_value("evolution", "trial_aggregation", _trial_aggregation)
    )
    if _trial_aggregation not in ["mean", "median", "worst", "best"]:
        _trial_aggregation = "mean"
    _structural_mutation_chance = float(
        config.get_value(
            "evolution",
            "structural_mutation_chance",
            _structural_mutation_chance
        )
    )
    _structural_mutation_chance = clampf(_structural_mutation_chance, 0.0, 1.0)

    _accelerator_mode = str(
        config.get_value("accelerator", "mode", _accelerator_mode)
    )
    if _accelerator_mode not in ["cpu", "auto", "cuda"]:
        _accelerator_mode = "cpu"
    _gpu_ids = str(config.get_value("accelerator", "gpu_ids", _gpu_ids))
    _gpu_batch_size = clampi(
        int(config.get_value("accelerator", "batch_size", _gpu_batch_size)),
        1,
        1000000
    )
    _gpu_max_parts = clampi(
        int(config.get_value("accelerator", "max_parts", _gpu_max_parts)),
        1,
        1024
    )
    _gpu_max_joints = clampi(
        int(config.get_value("accelerator", "max_joints", _gpu_max_joints)),
        1,
        2048
    )
    _cpu_fallback = bool(
        config.get_value("accelerator", "cpu_fallback", _cpu_fallback)
    )
    _throughput_mode = str(
        config.get_value("accelerator", "throughput_mode", _throughput_mode)
    )
    if _throughput_mode not in ["deterministic", "max"]:
        _throughput_mode = "deterministic"

    var timeline_raw := str(config.get_value("timeline", "json", ""))
    if not timeline_raw.is_empty():
        var parsed_timeline = JSON.parse_string(timeline_raw)
        if typeof(parsed_timeline) == TYPE_DICTIONARY:
            var keyframes = parsed_timeline.get("keyframes", [])
            if typeof(keyframes) == TYPE_ARRAY:
                _timeline_entries = keyframes
                _timeline_next_id = _timeline_entries.size() + 1

    _camera_move_speed = float(
        config.get_value("camera", "move_speed", _camera_move_speed)
    )
    _camera_move_speed = clampf(_camera_move_speed, 0.5, 50.0)
    _mouse_sensitivity_degrees = float(
        config.get_value(
            "camera",
            "mouse_sensitivity_degrees",
            _mouse_sensitivity_degrees
        )
    )
    _mouse_sensitivity_degrees = clampf(_mouse_sensitivity_degrees, 0.03, 1.0)


func _save_settings() -> void:
    var config := ConfigFile.new()
    config.set_value("ui", "font_size", _font_size)
    config.set_value("ui", "hud_width", _hud_width)
    config.set_value("ui", "walkthrough_completed", _walkthrough_completed)
    config.set_value("ui", "walkthrough_version", _walkthrough_version)
    config.set_value("viewer", "playback_speed", _playback_speed)
    config.set_value("fitness", "distance", _fitness_distance_weight)
    config.set_value("fitness", "average_speed", _fitness_speed_weight)
    config.set_value("fitness", "upright", _fitness_upright_weight)
    config.set_value("fitness", "stability", _fitness_stability_weight)
    config.set_value("fitness", "energy", _fitness_energy_weight)
    config.set_value("world", "terrain", _terrain_kind)
    config.set_value("world", "seed", _world_seed)
    config.set_value("world", "gravity_x", _gravity_x)
    config.set_value("world", "gravity_y", _gravity_y)
    config.set_value("world", "gravity_z", _gravity_z)
    config.set_value("world", "ground_friction", _ground_friction)
    config.set_value("world", "slope_degrees", _slope_degrees)
    config.set_value("world", "hill_height", _hill_height)
    config.set_value("world", "hill_wavelength", _hill_wavelength)
    config.set_value("world", "stair_height", _stair_height)
    config.set_value("world", "stair_depth", _stair_depth)
    config.set_value("world", "walls_enabled", _walls_enabled)
    config.set_value("world", "blocks_enabled", _blocks_enabled)
    config.set_value("world", "gaps_enabled", _gaps_enabled)
    config.set_value("world", "pits_enabled", _pits_enabled)
    config.set_value("world", "obstacle_count", _obstacle_count)
    config.set_value("world", "obstacle_spacing", _obstacle_spacing)
    config.set_value("world", "obstacle_size", _obstacle_size)
    config.set_value("world", "gap_width", _gap_width)
    config.set_value("world", "pit_depth", _pit_depth)
    config.set_value("evolution", "motor_strength", _motor_strength)
    config.set_value("evolution", "trials_per_creature", _trials_per_creature)
    config.set_value("evolution", "trial_aggregation", _trial_aggregation)
    config.set_value(
        "evolution",
        "structural_mutation_chance",
        _structural_mutation_chance
    )
    config.set_value("accelerator", "mode", _accelerator_mode)
    config.set_value("accelerator", "gpu_ids", _gpu_ids)
    config.set_value("accelerator", "batch_size", _gpu_batch_size)
    config.set_value("accelerator", "max_parts", _gpu_max_parts)
    config.set_value("accelerator", "max_joints", _gpu_max_joints)
    config.set_value("accelerator", "cpu_fallback", _cpu_fallback)
    config.set_value("accelerator", "throughput_mode", _throughput_mode)
    config.set_value("timeline", "json", JSON.stringify(_timeline_config_dictionary()))
    config.set_value("camera", "move_speed", _camera_move_speed)
    config.set_value(
        "camera",
        "mouse_sensitivity_degrees",
        _mouse_sensitivity_degrees
    )
    config.save(_settings_path())


func _update_layout() -> void:
    var viewport_size := get_viewport().get_visible_rect().size
    var max_for_window := maxf(
        MIN_HUD_WIDTH,
        minf(MAX_HUD_WIDTH, viewport_size.x - 240.0)
    )
    var requested_width := clampf(_hud_width, MIN_HUD_WIDTH, max_for_window)

    if is_instance_valid(_hud_panel):
        # Container children can force the panel wider than the requested size.
        # Use that real minimum so the resize handle stays on the actual edge.
        var content_min_width := _hud_panel.get_combined_minimum_size().x
        _visible_hud_width = maxf(requested_width, content_min_width)
        _hud_panel.position = Vector2.ZERO
        _hud_panel.size = Vector2(_visible_hud_width, viewport_size.y)
        _visible_hud_width = _hud_panel.size.x
    else:
        _visible_hud_width = requested_width

    if is_instance_valid(_hud_resize_handle):
        _hud_resize_handle.position = Vector2(
            _visible_hud_width - HUD_RESIZE_HANDLE_WIDTH * 0.5,
            0.0
        )
        _hud_resize_handle.size = Vector2(HUD_RESIZE_HANDLE_WIDTH, viewport_size.y)


func _on_hud_resize_input(event: InputEvent) -> void:
    if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
        _hud_dragging = event.pressed
        if not event.pressed:
            _hud_width = _visible_hud_width
            _save_settings()
        get_viewport().set_input_as_handled()
        return

    if event is InputEventMouseMotion and _hud_dragging:
        var viewport_width := get_viewport().get_visible_rect().size.x
        var max_for_window := maxf(
            MIN_HUD_WIDTH,
            minf(MAX_HUD_WIDTH, viewport_width - 240.0)
        )
        _hud_width = clampf(
            get_viewport().get_mouse_position().x,
            MIN_HUD_WIDTH,
            max_for_window
        )
        _update_layout()
        get_viewport().set_input_as_handled()


func _unhandled_input(event: InputEvent) -> void:
    if event is InputEventKey and event.keycode == KEY_ESCAPE and event.pressed:
        if _mouse_looking:
            _set_mouse_look(false)
            get_viewport().set_input_as_handled()
        return

    if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_RIGHT:
        if event.pressed:
            if _settings_window != null and _settings_window.visible:
                return
            if event.position.x <= _visible_hud_width:
                return
            _set_mouse_look(true)
        else:
            _set_mouse_look(false)
        get_viewport().set_input_as_handled()
        return

    if event is InputEventMouseMotion and _mouse_looking:
        var sensitivity := deg_to_rad(_mouse_sensitivity_degrees)
        _camera_yaw -= event.relative.x * sensitivity
        _camera_pitch = clampf(
            _camera_pitch - event.relative.y * sensitivity,
            deg_to_rad(-89.0),
            deg_to_rad(89.0)
        )
        _camera.rotation = Vector3(_camera_pitch, _camera_yaw, 0.0)
        get_viewport().set_input_as_handled()


func _set_mouse_look(enabled: bool) -> void:
    _mouse_looking = enabled
    Input.mouse_mode = (
        Input.MOUSE_MODE_CAPTURED if enabled else Input.MOUSE_MODE_VISIBLE
    )


func _update_camera_movement(delta: float) -> void:
    if not is_instance_valid(_camera):
        return
    if _settings_window != null and _settings_window.visible:
        return

    var focus_owner := get_viewport().gui_get_focus_owner()
    if focus_owner is LineEdit:
        return

    var direction := Vector3.ZERO
    var basis := _camera.global_transform.basis

    if Input.is_physical_key_pressed(KEY_W):
        direction += -basis.z
    if Input.is_physical_key_pressed(KEY_S):
        direction += basis.z
    if Input.is_physical_key_pressed(KEY_A):
        direction += -basis.x
    if Input.is_physical_key_pressed(KEY_D):
        direction += basis.x

    if direction.length_squared() > 0.0:
        _camera.global_position += (
            direction.normalized() * _camera_move_speed * delta
        )


func _build_file_dialogs() -> void:
    _save_dialog = FileDialog.new()
    _save_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
    _save_dialog.filters = PackedStringArray(["*.json ; Creature Genome JSON"])
    _save_dialog.current_dir = _portable_champions_dir()
    _save_dialog.current_file = "creature_genome.json"
    _save_dialog.file_selected.connect(_on_save_file_selected)
    add_child(_save_dialog)

    _load_dialog = FileDialog.new()
    _load_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _load_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
    _load_dialog.filters = PackedStringArray(["*.json ; Creature Genome JSON"])
    _load_dialog.current_dir = _portable_champions_dir()
    _load_dialog.file_selected.connect(_on_load_file_selected)
    add_child(_load_dialog)

    _experiment_save_dialog = FileDialog.new()
    _experiment_save_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _experiment_save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
    _experiment_save_dialog.filters = PackedStringArray(["*.evo ; EvoLab Experiment"])
    _experiment_save_dialog.current_dir = _portable_experiments_dir()
    _experiment_save_dialog.current_file = "experiment.evo"
    _experiment_save_dialog.file_selected.connect(_on_experiment_save_file_selected)
    add_child(_experiment_save_dialog)

    _experiment_load_dialog = FileDialog.new()
    _experiment_load_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _experiment_load_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
    _experiment_load_dialog.filters = PackedStringArray(["*.evo ; EvoLab Experiment"])
    _experiment_load_dialog.current_dir = _portable_experiments_dir()
    _experiment_load_dialog.file_selected.connect(_on_experiment_load_file_selected)
    add_child(_experiment_load_dialog)

    _results_load_dialog = FileDialog.new()
    _results_load_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _results_load_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
    _results_load_dialog.filters = PackedStringArray([
        "*.evoresults ; EvoLab Evolution Results"
    ])
    _results_load_dialog.current_dir = _portable_saves_dir()
    _results_load_dialog.file_selected.connect(_on_results_load_file_selected)
    add_child(_results_load_dialog)


func _add_collapsible_section(
    parent: VBoxContainer,
    title: String,
    expanded: bool
) -> VBoxContainer:
    var wrapper := VBoxContainer.new()
    wrapper.add_theme_constant_override("separation", 4)
    parent.add_child(wrapper)

    var header := Button.new()
    header.alignment = HORIZONTAL_ALIGNMENT_LEFT
    header.focus_mode = Control.FOCUS_NONE
    header.text = ("▼ " if expanded else "▶ ") + title
    wrapper.add_child(header)

    var content := VBoxContainer.new()
    content.add_theme_constant_override("separation", 6)
    content.visible = expanded
    wrapper.add_child(content)

    header.pressed.connect(
        func():
            content.visible = not content.visible
            header.text = ("▼ " if content.visible else "▶ ") + title
    )
    return content


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
    _cuda_devices = parsed.get("cuda_devices", [])
    var gpu_text := "no CUDA GPU"
    if not _cuda_devices.is_empty():
        var names: PackedStringArray = []
        for device_value in _cuda_devices:
            if typeof(device_value) == TYPE_DICTIONARY:
                var device: Dictionary = device_value
                names.append(
                    "%s: %s"
                    % [str(device.get("id", "?")), str(device.get("name", "GPU"))]
                )
        gpu_text = ", ".join(names)

    _capabilities_label.text = (
        "v%s • %s CPU threads • %s • CUDA: %s"
        % [
            str(parsed.get("app_version", "?")),
            str(parsed.get("logical_cpu_threads", "?")),
            str(backend.get("name", "unknown")),
            gpu_text,
        ]
    )


func _base_creature_args(
    world_override = null,
    motor_strength_override := -1.0
) -> PackedStringArray:
    var world_file := _world_file_for_cli(world_override)

    var motor_strength := float(_motor_strength_spin.value)
    if motor_strength_override >= 0.0:
        motor_strength = motor_strength_override

    return PackedStringArray([
        "creature-stream",
        "--event-port", str(_event_port),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--frame-hz", "60",
        "--playback-speed", "%.2f" % _playback_speed,
        "--seed", str(int(_seed_spin.value)),
        "--max-segments", str(int(_max_segments_spin.value)),
        "--world-file", world_file,
        "--motor-strength", str(motor_strength),
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
        var temp_path := _runtime_path("mutation_parent.json")
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
    if not _can_start_action("evolution"):
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
    _results_data.clear()
    _results_button.disabled = true
    _watch_champion_button.disabled = true
    _metrics.text = "[color=#9aa7bd]Creating and evaluating generation 1...[/color]"

    var champion_path := _evolution_champion_path()
    var results_path := _evolution_results_path()
    var checkpoint_path := _evolution_checkpoint_path()
    if FileAccess.file_exists(champion_path):
        DirAccess.remove_absolute(champion_path)
    if FileAccess.file_exists(results_path):
        DirAccess.remove_absolute(results_path)
    if FileAccess.file_exists(checkpoint_path):
        DirAccess.remove_absolute(checkpoint_path)

    var world_file := _world_file_for_cli()
    var timeline_file := _timeline_file_for_cli()
    if world_file.is_empty() or timeline_file.is_empty():
        _set_status("Could not write runtime experiment configuration.")
        return

    var args := PackedStringArray([
        "evolve",
        "--population", str(int(_population_spin.value)),
        "--evaluation-pool", str(int(_batch_spin.value)),
        "--generations", str(int(_generations_spin.value)),
        "--tournament", str(int(_tournament_spin.value)),
        "--elite", str(int(_elite_spin.value)),
        "--crossover", str(_crossover_spin.value),
        "--mutations", str(int(_evolution_mutations_spin.value)),
        "--structural-mutation-chance", str(_structural_mutation_spin.value),
        "--max-segments", str(int(_max_segments_spin.value)),
        "--seed", str(int(_seed_spin.value)),
        "--workers", str(int(_workers_spin.value)),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--motor-strength", str(_motor_strength_spin.value),
        "--trials", str(int(_trials_spin.value)),
        "--trial-aggregation", _trial_aggregation_value(),
        "--timeline-file", timeline_file,
        "--fitness-distance", str(_fitness_distance_spin.value),
        "--fitness-speed", str(_fitness_speed_spin.value),
        "--fitness-upright", str(_fitness_upright_spin.value),
        "--fitness-stability", str(_fitness_stability_spin.value),
        "--fitness-energy", str(_fitness_energy_spin.value),
        "--world-file", world_file,
        "--event-port", str(_event_port),
        "--champion-output", champion_path,
        "--result-output", results_path,
        "--experiment-name", _experiment_name,
        "--checkpoint-output", checkpoint_path,
    ])
    args.append_array(_accelerator_cli_args())

    if not _current_genome.is_empty():
        var parent_path := _runtime_path("evolution_parent.json")
        if not _write_genome_file(parent_path, _current_genome):
            _set_status("Could not write evolution parent genome.")
            return
        args.append_array(PackedStringArray(["--genome", parent_path]))

    _set_status("Starting evolution process...")
    if _start_job("evolution", args):
        _set_status("Evolution process started • waiting for generation 1")


func _on_resume_evolution_pressed() -> void:
    if _job_pid > 0:
        return

    var checkpoint_path := _evolution_checkpoint_path()
    if not FileAccess.file_exists(checkpoint_path):
        _set_status("No evolution checkpoint exists to resume.")
        _resume_evolution_button.disabled = true
        return

    if int(_tournament_spin.value) > int(_population_spin.value):
        _set_status("Tournament size cannot exceed population.")
        return
    if int(_elite_spin.value) >= int(_population_spin.value):
        _set_status("Elite kept must be smaller than population.")
        return

    _probe_mesh.visible = false
    _progress_bar.value = 0
    _metrics.text = "[color=#9aa7bd]Resuming evolution from checkpoint...[/color]"

    var champion_path := _evolution_champion_path()
    var results_path := _evolution_results_path()
    var world_file := _world_file_for_cli()
    var timeline_file := _timeline_file_for_cli()
    if world_file.is_empty() or timeline_file.is_empty():
        _set_status("Could not write runtime experiment configuration.")
        return
    var args := PackedStringArray([
        "evolve",
        "--population", str(int(_population_spin.value)),
        "--evaluation-pool", str(int(_batch_spin.value)),
        "--generations", str(int(_generations_spin.value)),
        "--tournament", str(int(_tournament_spin.value)),
        "--elite", str(int(_elite_spin.value)),
        "--crossover", str(_crossover_spin.value),
        "--mutations", str(int(_evolution_mutations_spin.value)),
        "--structural-mutation-chance", str(_structural_mutation_spin.value),
        "--max-segments", str(int(_max_segments_spin.value)),
        "--seed", str(int(_seed_spin.value)),
        "--workers", str(int(_workers_spin.value)),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--motor-strength", str(_motor_strength_spin.value),
        "--trials", str(int(_trials_spin.value)),
        "--trial-aggregation", _trial_aggregation_value(),
        "--timeline-file", timeline_file,
        "--fitness-distance", str(_fitness_distance_spin.value),
        "--fitness-speed", str(_fitness_speed_spin.value),
        "--fitness-upright", str(_fitness_upright_spin.value),
        "--fitness-stability", str(_fitness_stability_spin.value),
        "--fitness-energy", str(_fitness_energy_spin.value),
        "--world-file", world_file,
        "--event-port", str(_event_port),
        "--champion-output", champion_path,
        "--result-output", results_path,
        "--experiment-name", _experiment_name,
        "--checkpoint-output", checkpoint_path,
        "--resume-checkpoint", checkpoint_path,
    ])
    args.append_array(_accelerator_cli_args())

    if not _current_genome.is_empty():
        var parent_path := _runtime_path("evolution_parent.json")
        if not _write_genome_file(parent_path, _current_genome):
            _set_status("Could not write evolution parent genome.")
            return
        args.append_array(PackedStringArray(["--genome", parent_path]))

    if _start_job("evolution", args):
        _set_status("Resuming evolution from the latest completed generation...")


func _on_watch_champion_pressed() -> void:
    if _job_pid > 0 or _current_genome.is_empty():
        return

    var champion_path := _evolution_champion_path()
    if not FileAccess.file_exists(champion_path):
        if not _write_genome_file(champion_path, _current_genome):
            _set_status("Could not prepare champion genome.")
            return

    _prepare_creature_run()
    var args := _base_creature_args(
        _champion_world,
        _champion_motor_strength
    )
    args.append_array(PackedStringArray(["--genome", champion_path]))

    if _start_job("creature", args):
        _set_status("Watching evolution champion in real time...")


func _on_save_experiment_pressed() -> void:
    if _job_pid > 0:
        return
    _experiment_fork_pending = false
    var safe_name := _experiment_name.to_snake_case()
    if safe_name.is_empty():
        safe_name = "experiment"
    _experiment_save_dialog.current_file = safe_name + ".evo"
    _experiment_save_dialog.popup_centered_ratio(0.72)


func _on_load_experiment_pressed() -> void:
    if _job_pid > 0:
        return
    _experiment_load_dialog.popup_centered_ratio(0.72)


func _on_fork_experiment_pressed() -> void:
    if _job_pid > 0:
        return
    _experiment_fork_pending = true
    var safe_name := _experiment_name.to_snake_case()
    if safe_name.is_empty():
        safe_name = "experiment"
    _experiment_save_dialog.current_file = safe_name + "_fork.evo"
    _experiment_save_dialog.popup_centered_ratio(0.72)


func _experiment_ancestor_dictionary() -> Dictionary:
    if not _current_genome.is_empty():
        return _current_genome.duplicate(true)

    if not _backend_exists():
        return {}

    var temp_path := _runtime_path("experiment_default_ancestor.json")
    var output: Array = []
    var exit_code := OS.execute(
        _backend_path(),
        PackedStringArray([
            "genome-generate",
            "--output",
            temp_path,
            "--seed",
            str(int(_seed_spin.value)),
        ]),
        output,
        true,
        false
    )
    if exit_code != 0 or not FileAccess.file_exists(temp_path):
        return {}

    var file := FileAccess.open(temp_path, FileAccess.READ)
    if file == null:
        return {}
    var parsed = JSON.parse_string(file.get_as_text())
    file.close()
    if typeof(parsed) == TYPE_DICTIONARY:
        return parsed
    return {}


func _parse_gpu_ids_for_json(value: String) -> Array:
    var ids: Array = []
    for part in value.split(",", false):
        var trimmed := part.strip_edges()
        if trimmed.is_valid_int():
            ids.append(int(trimmed))
    return ids


func _experiment_dictionary(name_override := "") -> Dictionary:
    var ancestor := _experiment_ancestor_dictionary()
    if ancestor.is_empty():
        return {}

    _sync_world_state_from_controls()
    var experiment_name := _experiment_name
    if not str(name_override).is_empty():
        experiment_name = str(name_override)

    return {
        "format_version": 1,
        "name": experiment_name,
        "ancestor": ancestor,
        "evolution": {
            "population_size": int(_population_spin.value),
            "evaluation_pool_size": int(_batch_spin.value),
            "generations": int(_generations_spin.value),
            "tournament_size": int(_tournament_spin.value),
            "elite_count": int(_elite_spin.value),
            "crossover_chance": float(_crossover_spin.value),
            "mutations_per_child": int(_evolution_mutations_spin.value),
            "seed": int(_seed_spin.value),
            "worker_threads": int(_workers_spin.value),
            "simulation": {
                "dt": float(_dt_spin.value),
                "duration_seconds": float(_seconds_spin.value),
                "world": _world_config_dictionary(),
                "motor_strength_multiplier": float(_motor_strength_spin.value),
                "deterministic": true,
            },
            "fitness": _fitness_config_dictionary(),
            "mutation": {
                "min_segments": 2,
                "max_segments": int(_max_segments_spin.value),
                "min_half_extent": 0.0001,
                "max_half_extent": 15.0,
                "structural_mutation_chance":
                    float(_structural_mutation_spin.value),
            },
            "trials_per_creature": int(_trials_spin.value),
            "trial_aggregation": _trial_aggregation_value(),
            "accelerator": {
                "mode": _accelerator_mode,
                "gpu_ids": _parse_gpu_ids_for_json(_gpu_ids_edit.text),
                "batch_size": int(_gpu_batch_spin.value),
                "max_parts": int(_gpu_max_parts_spin.value),
                "max_joints": int(_gpu_max_joints_spin.value),
                "cpu_fallback": _cpu_fallback_check.button_pressed,
                "throughput_mode": (
                    "deterministic"
                    if _throughput_mode == "deterministic"
                    else "max_throughput"
                ),
            },
            "timeline": _timeline_config_dictionary(),
        },
    }


func _on_experiment_save_file_selected(path: String) -> void:
    var file_name := path.get_file().get_basename()
    var save_name := file_name
    if _experiment_fork_pending:
        save_name = "Fork of %s" % _experiment_name

    var experiment := _experiment_dictionary(save_name)
    if experiment.is_empty():
        _set_status("Could not create experiment file.")
        _experiment_fork_pending = false
        return

    var file := FileAccess.open(path, FileAccess.WRITE)
    if file == null:
        _set_status("Could not save experiment: %s" % path)
        _experiment_fork_pending = false
        return

    file.store_string(JSON.stringify(experiment, "	"))
    file.close()
    _experiment_name = str(experiment.get("name", file_name))
    _experiment_fork_pending = false
    _set_status("Saved experiment: %s" % path)


func _on_experiment_load_file_selected(path: String) -> void:
    var file := FileAccess.open(path, FileAccess.READ)
    if file == null:
        _set_status("Could not open experiment: %s" % path)
        return

    var parsed = JSON.parse_string(file.get_as_text())
    file.close()
    if typeof(parsed) != TYPE_DICTIONARY:
        _set_status("Experiment file is not valid JSON.")
        return

    var experiment: Dictionary = parsed
    if int(experiment.get("format_version", 0)) != 1:
        _set_status("Unsupported experiment file version.")
        return

    if not _apply_experiment_dictionary(experiment):
        _set_status("Experiment file is incomplete or invalid.")
        return

    _set_status(
        "Loaded experiment '%s' • %d timeline keyframes"
        % [_experiment_name, _timeline_entries.size()]
    )


func _apply_experiment_dictionary(experiment: Dictionary) -> bool:
    var ancestor_value = experiment.get("ancestor", null)
    var evolution_value = experiment.get("evolution", null)
    if (
        typeof(ancestor_value) != TYPE_DICTIONARY
        or typeof(evolution_value) != TYPE_DICTIONARY
    ):
        return false

    var evolution: Dictionary = evolution_value
    _experiment_name = str(experiment.get("name", "Experiment"))
    _current_genome = ancestor_value.duplicate(true)
    _current_genome_source = "experiment: %s" % _experiment_name
    _save_button.disabled = false

    _population_spin.value = int(
        evolution.get("population_size", _population_spin.value)
    )
    _batch_spin.value = int(
        evolution.get("evaluation_pool_size", _batch_spin.value)
    )
    _generations_spin.value = int(
        evolution.get("generations", _generations_spin.value)
    )
    _tournament_spin.value = int(
        evolution.get("tournament_size", _tournament_spin.value)
    )
    _elite_spin.value = int(evolution.get("elite_count", _elite_spin.value))
    _crossover_spin.value = float(
        evolution.get("crossover_chance", _crossover_spin.value)
    )
    _evolution_mutations_spin.value = int(
        evolution.get("mutations_per_child", _evolution_mutations_spin.value)
    )
    _seed_spin.value = int(evolution.get("seed", _seed_spin.value))
    _workers_spin.value = int(
        evolution.get("worker_threads", _workers_spin.value)
    )

    var simulation_value = evolution.get("simulation", {})
    if typeof(simulation_value) == TYPE_DICTIONARY:
        var simulation: Dictionary = simulation_value
        _dt_spin.value = float(simulation.get("dt", _dt_spin.value))
        _seconds_spin.value = float(
            simulation.get("duration_seconds", _seconds_spin.value)
        )
        _motor_strength_spin.value = float(
            simulation.get(
                "motor_strength_multiplier",
                _motor_strength_spin.value
            )
        )
        var world_value = simulation.get("world", {})
        if typeof(world_value) == TYPE_DICTIONARY:
            _apply_world_dictionary(world_value)

    var fitness_value = evolution.get("fitness", {})
    if typeof(fitness_value) == TYPE_DICTIONARY:
        var weights_value = fitness_value.get("weights", {})
        if typeof(weights_value) == TYPE_DICTIONARY:
            var weights: Dictionary = weights_value
            _fitness_distance_spin.value = float(
                weights.get("distance", _fitness_distance_spin.value)
            )
            _fitness_speed_spin.value = float(
                weights.get("average_speed", _fitness_speed_spin.value)
            )
            _fitness_upright_spin.value = float(
                weights.get("upright", _fitness_upright_spin.value)
            )
            _fitness_stability_spin.value = float(
                weights.get("stability", _fitness_stability_spin.value)
            )
            _fitness_energy_spin.value = float(
                weights.get("energy", _fitness_energy_spin.value)
            )

    var mutation_value = evolution.get("mutation", {})
    if typeof(mutation_value) == TYPE_DICTIONARY:
        var mutation: Dictionary = mutation_value
        _max_segments_spin.value = int(
            mutation.get("max_segments", _max_segments_spin.value)
        )
        _structural_mutation_spin.value = float(
            mutation.get(
                "structural_mutation_chance",
                _structural_mutation_spin.value
            )
        )

    _trials_spin.value = int(
        evolution.get("trials_per_creature", _trials_spin.value)
    )
    _trial_aggregation = str(
        evolution.get("trial_aggregation", _trial_aggregation)
    )
    var aggregation_index := ["mean", "median", "worst", "best"].find(
        _trial_aggregation
    )
    _trial_aggregation_option.select(maxi(aggregation_index, 0))

    var accelerator_value = evolution.get("accelerator", {})
    if typeof(accelerator_value) == TYPE_DICTIONARY:
        var accelerator: Dictionary = accelerator_value
        _accelerator_mode = str(accelerator.get("mode", _accelerator_mode))
        var accelerator_index := ["cpu", "auto", "cuda"].find(_accelerator_mode)
        _accelerator_option.select(maxi(accelerator_index, 0))

        var gpu_ids_value = accelerator.get("gpu_ids", [])
        if typeof(gpu_ids_value) == TYPE_ARRAY:
            var gpu_parts: PackedStringArray = []
            for gpu_id in gpu_ids_value:
                gpu_parts.append(str(gpu_id))
            _gpu_ids = ",".join(gpu_parts)
            _gpu_ids_edit.text = _gpu_ids

        _gpu_batch_spin.value = int(
            accelerator.get("batch_size", _gpu_batch_spin.value)
        )
        _gpu_max_parts_spin.value = int(
            accelerator.get("max_parts", _gpu_max_parts_spin.value)
        )
        _gpu_max_joints_spin.value = int(
            accelerator.get("max_joints", _gpu_max_joints_spin.value)
        )
        _cpu_fallback_check.button_pressed = bool(
            accelerator.get("cpu_fallback", _cpu_fallback_check.button_pressed)
        )
        var throughput := str(
            accelerator.get("throughput_mode", "deterministic")
        )
        _throughput_mode = (
            "max" if throughput in ["max", "max_throughput"] else "deterministic"
        )
        _throughput_option.select(0 if _throughput_mode == "deterministic" else 1)

    var timeline_value = evolution.get("timeline", {})
    if typeof(timeline_value) == TYPE_DICTIONARY:
        var keyframes = timeline_value.get("keyframes", [])
        if typeof(keyframes) == TYPE_ARRAY:
            _timeline_entries = keyframes.duplicate(true)
            _sort_timeline_entries()
            _timeline_next_id = _timeline_entries.size() + 1
            _refresh_timeline_list()

    _sync_world_state_from_controls()
    _refresh_world_preview()
    _build_creature_from_genome(_current_genome)
    _save_settings()
    return true


func _apply_world_dictionary(world: Dictionary) -> void:
    var gravity: Array = world.get(
        "gravity",
        [_gravity_x, _gravity_y, _gravity_z]
    )
    if gravity.size() >= 3:
        _gravity_x = float(gravity[0])
        _gravity_y = float(gravity[1])
        _gravity_z = float(gravity[2])

    _ground_friction = float(
        world.get("ground_friction", _ground_friction)
    )
    _terrain_kind = str(world.get("terrain", _terrain_kind))
    _world_seed = int(world.get("seed", _world_seed))
    _slope_degrees = float(world.get("slope_degrees", _slope_degrees))
    _hill_height = float(world.get("hill_height", _hill_height))
    _hill_wavelength = float(
        world.get("hill_wavelength", _hill_wavelength)
    )
    _stair_height = float(world.get("stair_height", _stair_height))
    _stair_depth = float(world.get("stair_depth", _stair_depth))
    _walls_enabled = bool(world.get("walls_enabled", _walls_enabled))
    _blocks_enabled = bool(world.get("blocks_enabled", _blocks_enabled))
    _gaps_enabled = bool(world.get("gaps_enabled", _gaps_enabled))
    _pits_enabled = bool(world.get("pits_enabled", _pits_enabled))
    _obstacle_count = int(world.get("obstacle_count", _obstacle_count))
    _obstacle_spacing = float(
        world.get("obstacle_spacing", _obstacle_spacing)
    )
    _obstacle_size = float(world.get("obstacle_size", _obstacle_size))
    _gap_width = float(world.get("gap_width", _gap_width))
    _pit_depth = float(world.get("pit_depth", _pit_depth))

    var terrain_index := ["flat", "slope", "hills", "stairs"].find(
        _terrain_kind
    )
    _terrain_option.select(maxi(terrain_index, 0))
    _world_seed_spin.value = _world_seed
    _gravity_x_spin.value = _gravity_x
    _gravity_y_spin.value = _gravity_y
    _gravity_z_spin.value = _gravity_z
    _ground_friction_spin.value = _ground_friction
    _slope_spin.value = _slope_degrees
    _hill_height_spin.value = _hill_height
    _hill_wavelength_spin.value = _hill_wavelength
    _stair_height_spin.value = _stair_height
    _stair_depth_spin.value = _stair_depth
    _walls_check.button_pressed = _walls_enabled
    _blocks_check.button_pressed = _blocks_enabled
    _gaps_check.button_pressed = _gaps_enabled
    _pits_check.button_pressed = _pits_enabled
    _obstacle_count_spin.value = _obstacle_count
    _obstacle_spacing_spin.value = _obstacle_spacing
    _obstacle_size_spin.value = _obstacle_size
    _gap_width_spin.value = _gap_width
    _pit_depth_spin.value = _pit_depth


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


func _evolution_champion_path() -> String:
    return _runtime_path("active_evolution_champion.json")


func _evolution_results_path() -> String:
    return _runtime_path("active_evolution_results.evoresults")


func _evolution_checkpoint_path() -> String:
    return _runtime_path("active_evolution_checkpoint.evockpt")


func _champion_objective_label() -> String:
    var weights := {
        "Distance": _fitness_distance_weight,
        "Speed": _fitness_speed_weight,
        "Upright": _fitness_upright_weight,
        "Stability": _fitness_stability_weight,
        "Energy": _fitness_energy_weight,
    }

    if not _results_data.is_empty():
        var result_value = _results_data.get("result", {})
        if typeof(result_value) == TYPE_DICTIONARY:
            var result: Dictionary = result_value
            var final_settings_value = result.get("final_settings", {})
            if typeof(final_settings_value) == TYPE_DICTIONARY:
                var final_settings: Dictionary = final_settings_value
                var fitness_value = final_settings.get("fitness", {})
                if typeof(fitness_value) == TYPE_DICTIONARY:
                    var fitness: Dictionary = fitness_value
                    var saved_weights_value = fitness.get("weights", {})
                    if typeof(saved_weights_value) == TYPE_DICTIONARY:
                        var saved_weights: Dictionary = saved_weights_value
                        weights["Distance"] = float(saved_weights.get("distance", weights["Distance"]))
                        weights["Speed"] = float(saved_weights.get("average_speed", weights["Speed"]))
                        weights["Upright"] = float(saved_weights.get("upright", weights["Upright"]))
                        weights["Stability"] = float(saved_weights.get("stability", weights["Stability"]))
                        weights["Energy"] = float(saved_weights.get("energy", weights["Energy"]))

    var best_name := "Fitness"
    var best_weight := 0.0
    for name in weights:
        var magnitude := absf(float(weights[name]))
        if magnitude > best_weight:
            best_weight = magnitude
            best_name = str(name)
    return best_name


func _filename_number(value: float) -> String:
    var text := "%.3f" % value
    while text.contains(".") and text.ends_with("0"):
        text = text.left(text.length() - 1)
    if text.ends_with("."):
        text = text.left(text.length() - 1)
    if text == "-0":
        text = "0"
    return text


func _champion_archive_path(fitness: float) -> String:
    _ensure_portable_directories()
    var now := Time.get_datetime_dict_from_system()
    var months := [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sept", "Oct", "Nov", "Dec"
    ]
    var month_index := clampi(int(now.get("month", 1)) - 1, 0, 11)
    var hour_24 := int(now.get("hour", 0))
    var minute := int(now.get("minute", 0))
    var am_pm := "am" if hour_24 < 12 else "pm"
    var hour_12 := hour_24 % 12
    if hour_12 == 0:
        hour_12 = 12

    var base_name := (
        "champ_%s%d-%d-%d-%02d%s_%s-%s"
        % [
            months[month_index],
            int(now.get("day", 1)),
            int(now.get("year", 1970)),
            hour_12,
            minute,
            am_pm,
            _champion_objective_label(),
            _filename_number(fitness),
        ]
    )
    var candidate := _portable_champions_dir().path_join(base_name + ".json")
    var suffix := 2
    while FileAccess.file_exists(candidate):
        candidate = _portable_champions_dir().path_join(
            "%s_%d.json" % [base_name, suffix]
        )
        suffix += 1
    return candidate


func _archive_current_champion(fitness: float) -> String:
    if _current_genome.is_empty():
        return ""
    var archive_path := _champion_archive_path(fitness)
    if not _write_genome_file(archive_path, _current_genome):
        return ""
    return archive_path


func _load_results_file(path: String) -> bool:
    if not FileAccess.file_exists(path):
        return false

    var file := FileAccess.open(path, FileAccess.READ)
    if file == null:
        return false

    var parsed = JSON.parse_string(file.get_as_text())
    file.close()
    if typeof(parsed) != TYPE_DICTIONARY:
        return false

    _results_data = parsed
    _results_button.disabled = false
    _refresh_results_window()
    return true


func _load_genome_file(path: String) -> bool:
    if not FileAccess.file_exists(path):
        return false

    var file := FileAccess.open(path, FileAccess.READ)
    if file == null:
        return false

    var parsed = JSON.parse_string(file.get_as_text())
    file.close()

    if typeof(parsed) != TYPE_DICTIONARY:
        return false

    _current_genome = parsed
    _save_button.disabled = false
    return true


func _normalize_integral_json_numbers(value):
    match typeof(value):
        TYPE_DICTIONARY:
            var result := {}
            for key in value.keys():
                result[key] = _normalize_integral_json_numbers(value[key])
            return result
        TYPE_ARRAY:
            var result: Array = []
            for item in value:
                result.append(_normalize_integral_json_numbers(item))
            return result
        TYPE_FLOAT:
            var float_value := float(value)
            if (
                is_finite(float_value)
                and float_value == floor(float_value)
                and absf(float_value) <= 9007199254740991.0
            ):
                return int(float_value)
            return float_value
        _:
            return value


func _write_genome_file(path: String, genome: Dictionary) -> bool:
    var file := FileAccess.open(path, FileAccess.WRITE)
    if file == null:
        return false
    var normalized = _normalize_integral_json_numbers(genome)
    file.store_string(JSON.stringify(normalized, "\t"))
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

    var world_file := _world_file_for_cli()
    if world_file.is_empty():
        _set_status("Could not write runtime world configuration.")
        return

    var args := PackedStringArray([
        "stream",
        "--event-port", str(_event_port),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--frame-hz", "60",
        "--playback-speed", "%.2f" % _playback_speed,
        "--world-file", world_file,
    ])

    if _start_job("live", args):
        _set_status("Single-box physics running...")


func _on_batch_pressed() -> void:
    if not _can_start_action("benchmark"):
        return

    _clear_creature_meshes()
    _probe_mesh.visible = true
    _reset_probe()
    _progress_bar.value = 0
    _metrics.text = "[color=#9aa7bd]Evaluating independent simulations in parallel...[/color]"

    var world_file := _world_file_for_cli()
    if world_file.is_empty():
        _set_status("Could not write runtime world configuration.")
        return

    var args := PackedStringArray([
        "probe",
        "--batch", str(int(_batch_spin.value)),
        "--workers", str(int(_workers_spin.value)),
        "--seconds", str(_seconds_spin.value),
        "--dt", str(_dt_spin.value),
        "--world-file", world_file,
        "--backend", _accelerator_mode,
        "--gpus", _gpu_ids_for_cli(),
        "--event-port", str(_event_port),
    ])

    _set_status("Starting benchmark process...")
    if _start_job("batch", args):
        _set_status("Benchmark process started • waiting for backend")


func _can_start_action(action_name: String) -> bool:
    if _job_pid <= 0:
        return true

    if OS.is_process_running(_job_pid):
        var running_name := _job_kind if not _job_kind.is_empty() else "simulation"
        _set_status(
            "Cannot start %s while %s is still running."
            % [action_name, running_name]
        )
        return false

    # Do not let a stale process ID make a button silently do nothing.
    _job_pid = 0
    _job_kind = ""
    _dead_process_since_ms = -1
    _job_started_ms = -1
    _job_received_event = false
    _reset_replay()
    _finish_job_controls()
    return true


func _start_job(kind: String, args: PackedStringArray) -> bool:
    if kind == "live" or kind == "creature":
        _begin_replay(kind)

    _job_kind = kind
    _dead_process_since_ms = -1
    _job_started_ms = Time.get_ticks_msec()
    _job_received_event = false
    _job_pid = OS.create_process(_backend_path(), args, false)

    if _job_pid <= 0:
        _job_pid = 0
        _job_kind = ""
        _job_started_ms = -1
        _set_status("Failed to start Rust simulator.")
        return false

    _set_run_buttons_disabled(true)
    _set_world_controls_enabled(false)
    _set_timeline_controls_enabled(false)
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
    _dead_process_since_ms = -1
    _job_started_ms = -1
    _job_received_event = false
    _reset_replay()
    _finish_job_controls()


func _finish_job_controls() -> void:
    _set_run_buttons_disabled(false)
    _set_world_controls_enabled(true)
    _set_timeline_controls_enabled(true)
    _stop_button.disabled = true
    _save_button.disabled = _current_genome.is_empty()


func _set_timeline_controls_enabled(enabled: bool) -> void:
    if _timeline_generation_spin != null:
        _timeline_generation_spin.editable = enabled
    if _timeline_condition_option != null:
        _timeline_condition_option.disabled = not enabled
    if _timeline_condition_value_spin != null:
        _timeline_condition_value_spin.editable = (
            enabled and _timeline_condition_option.selected != 0
        )
    if _timeline_list != null:
        _timeline_list.mouse_filter = (
            Control.MOUSE_FILTER_STOP if enabled else Control.MOUSE_FILTER_IGNORE
        )
    if _add_timeline_button != null:
        _add_timeline_button.disabled = not enabled
    if _remove_timeline_button != null:
        _remove_timeline_button.disabled = not enabled
    if _clear_timeline_button != null:
        _clear_timeline_button.disabled = not enabled


func _set_run_buttons_disabled(disabled: bool) -> void:
    _seed_creature_button.disabled = disabled
    _mutate_button.disabled = disabled
    _random_button.disabled = disabled
    _load_button.disabled = disabled
    _live_button.disabled = disabled
    _batch_button.disabled = disabled
    _evolve_button.disabled = disabled
    _resume_evolution_button.disabled = (
        disabled or not FileAccess.file_exists(_evolution_checkpoint_path())
    )
    _watch_champion_button.disabled = disabled or not _has_evolution_champion
    _save_button.disabled = disabled or _current_genome.is_empty()
    _save_experiment_button.disabled = disabled
    _load_experiment_button.disabled = disabled
    _fork_experiment_button.disabled = disabled


func _handle_event(event: Dictionary) -> void:
    _job_received_event = true
    var kind := str(event.get("kind", ""))

    if kind == "batch_error":
        _job_pid = 0
        _job_kind = ""
        _dead_process_since_ms = -1
        _job_started_ms = -1
        _progress_bar.value = 0
        _set_status("Benchmark error: %s" % str(event.get("message", "unknown error")))
        _metrics.text = (
            "[color=#ff8a8a][b]Benchmark failed:[/b] %s[/color]"
            % str(event.get("message", "unknown error"))
        )
        _finish_job_controls()
        return

    if kind == "creature_stream_error":
        _job_pid = 0
        _job_kind = ""
        _dead_process_since_ms = -1
        _job_started_ms = -1
        _walkthrough_guided_run_active = false
        _reset_replay()
        _set_status("Creature simulator error: %s" % str(event.get("message", "unknown error")))
        _metrics.text = (
            "[color=#ff8a8a][b]Creature test failed:[/b] %s[/color]"
            % str(event.get("message", "unknown error"))
        )
        _finish_job_controls()
        return

    match kind:
        "evolution_started":
            _build_world_from_geometry(event.get("world_geometry", []))
            _progress_bar.value = 0
            _set_status(
                "Evolution started • %s survivors • requested eval pool %s • %s generations"
                % [
                    str(event.get("population", 0)),
                    str(event.get("evaluation_pool", event.get("population", 0))),
                    str(event.get("generations", 0)),
                ]
            )

        "generation_complete":
            var generation := int(event.get("generation", 0))
            var generations := int(event.get("generations", 1))
            _progress_bar.value = float(event.get("fraction", 0.0)) * 100.0

            var champion_file := str(event.get("champion_file", ""))
            if champion_file != "" and _load_genome_file(champion_file):
                _current_genome_source = "generation %d champion" % generation
                _probe_mesh.visible = false
                _build_creature_from_genome(_current_genome)

            var best_metrics: Dictionary = event.get("best_metrics", {})
            var effective_world_value = event.get("effective_world", {})
            if typeof(effective_world_value) == TYPE_DICTIONARY:
                _render_world_config(effective_world_value)

            var triggered_events: Array = event.get(
                "triggered_timeline_events",
                []
            )
            var timeline_suffix := ""
            if not triggered_events.is_empty():
                timeline_suffix = " • triggered: %s" % ", ".join(triggered_events)

            _set_status(
                "Generation %d / %d • eval %s • best %.4f • average %.4f%s"
                % [
                    generation,
                    generations,
                    str(event.get("evaluation_pool_size", event.get("effective_population", 0))),
                    float(event.get("best_fitness", 0.0)),
                    float(event.get("average_fitness", 0.0)),
                    timeline_suffix,
                ]
            )
            _metrics.text = (
                "[table=2]"
                + "[cell]Generation[/cell][cell][b]%d / %d[/b][/cell]" % [generation, generations]
                + "[cell]Best fitness[/cell][cell][b]%.4f[/b][/cell]" % float(event.get("best_fitness", 0.0))
                + "[cell]Distance[/cell][cell]%.4f m[/cell]" % float(best_metrics.get("distance", 0.0))
                + "[cell]Average speed[/cell][cell]%.4f m/s[/cell]" % float(best_metrics.get("average_speed", 0.0))
                + "[cell]Upright[/cell][cell]%.3f[/cell]" % float(best_metrics.get("upright", 0.0))
                + "[cell]Stability[/cell][cell]%.3f[/cell]" % float(best_metrics.get("stability", 0.0))
                + "[cell]Energy / effort[/cell][cell]%.4f[/cell]" % float(best_metrics.get("energy", 0.0))
                + "[cell]Champion segments[/cell][cell]%s[/cell]" % str(event.get("best_segments", 0))
                + "[cell]Brain nodes[/cell][cell]%s[/cell]" % str(event.get("best_brain_nodes", 0))
                + "[cell]Sensors used[/cell][cell]%s unique / %s nodes[/cell]"
                    % [
                        str(event.get("best_brain_unique_sensors", 0)),
                        str(event.get("best_brain_sensor_nodes", 0)),
                    ]
                + "[cell]Brain outputs[/cell][cell]%s[/cell]" % str(event.get("best_brain_outputs", 0))
                + "[cell]Survivor population[/cell][cell]%s[/cell]"
                    % str(event.get("effective_population", 0))
                + "[cell]Candidates evaluated / gen[/cell][cell]%s[/cell]"
                    % str(event.get("evaluation_pool_size", event.get("effective_population", 0)))
                + "[cell]Trials / creature[/cell][cell]%s (%s)[/cell]"
                    % [
                        str(event.get("effective_trials_per_creature", 1)),
                        str(event.get("effective_trial_aggregation", "mean")),
                    ]
                + "[cell]Trial duration[/cell][cell]%.2f s[/cell]"
                    % float(event.get("effective_duration_seconds", 0.0))
                + "[cell]Motor strength[/cell][cell]%.2fx[/cell]"
                    % float(event.get("effective_motor_strength_multiplier", 1.0))
                + "[cell]Evaluations[/cell][cell]%s[/cell]" % str(event.get("evaluations_completed", 0))
                + "[/table]"
            )

        "evolution_complete":
            var completed_checkpoint := _evolution_checkpoint_path()
            if FileAccess.file_exists(completed_checkpoint):
                DirAccess.remove_absolute(completed_checkpoint)
            _resume_evolution_button.disabled = true

            var champion_file := str(event.get("champion_file", ""))
            if champion_file != "":
                _load_genome_file(champion_file)

            var results_file := str(event.get("results_file", ""))
            if results_file != "":
                _load_results_file(results_file)

            var archived_champion := _archive_current_champion(
                float(event.get("champion_fitness", 0.0))
            )
            _current_genome_source = (
                archived_champion
                if not archived_champion.is_empty()
                else champion_file
            )
            _build_creature_from_genome(_current_genome)
            _has_evolution_champion = not _current_genome.is_empty()

            var final_settings_value = event.get("final_settings", {})
            if typeof(final_settings_value) == TYPE_DICTIONARY:
                var final_settings: Dictionary = final_settings_value
                var final_simulation_value = final_settings.get("simulation", {})
                if typeof(final_simulation_value) == TYPE_DICTIONARY:
                    var final_simulation: Dictionary = final_simulation_value
                    var final_world_value = final_simulation.get("world", {})
                    if typeof(final_world_value) == TYPE_DICTIONARY:
                        _champion_world = final_world_value.duplicate(true)
                        _render_world_config(_champion_world)
                    _champion_motor_strength = float(
                        final_simulation.get(
                            "motor_strength_multiplier",
                            _motor_strength_spin.value
                        )
                    )

            _progress_bar.value = 100
            var champion_metrics: Dictionary = event.get("champion_metrics", {})
            _set_status(
                "Evolution complete • champion score %.4f • %s evaluations"
                % [
                    float(event.get("champion_fitness", 0.0)),
                    str(event.get("evaluations_completed", 0)),
                ]
            )
            _metrics.text = (
                "[table=2]"
                + "[cell]Champion fitness[/cell][cell][b]%.4f[/b][/cell]" % float(event.get("champion_fitness", 0.0))
                + "[cell]Distance[/cell][cell]%.4f m[/cell]" % float(champion_metrics.get("distance", 0.0))
                + "[cell]Average speed[/cell][cell]%.4f m/s[/cell]" % float(champion_metrics.get("average_speed", 0.0))
                + "[cell]Upright[/cell][cell]%.3f[/cell]" % float(champion_metrics.get("upright", 0.0))
                + "[cell]Stability[/cell][cell]%.3f[/cell]" % float(champion_metrics.get("stability", 0.0))
                + "[cell]Energy / effort[/cell][cell]%.4f[/cell]" % float(champion_metrics.get("energy", 0.0))
                + "[cell]Generations[/cell][cell]%s[/cell]" % str(event.get("generations_completed", 0))
                + "[cell]Evaluations[/cell][cell]%s[/cell]" % str(event.get("evaluations_completed", 0))
                + "[cell]Brain nodes[/cell][cell]%s[/cell]" % str(event.get("champion_brain_nodes", 0))
                + "[cell]Sensors used[/cell][cell]%s unique / %s nodes[/cell]"
                    % [
                        str(event.get("champion_brain_unique_sensors", 0)),
                        str(event.get("champion_brain_sensor_nodes", 0)),
                    ]
                + "[cell]Brain outputs[/cell][cell]%s[/cell]" % str(event.get("champion_brain_outputs", 0))
                + "[cell]Wall time[/cell][cell]%.3f s[/cell]" % float(event.get("wall_seconds", 0.0))
                + "[cell]Next[/cell][cell]Click Watch Champion[/cell]"
                + "[/table]"
            )
            _job_pid = 0
            _job_kind = ""
            _dead_process_since_ms = -1
            _finish_job_controls()

        "evolution_error":
            _progress_bar.value = 0
            _set_status("Evolution error: %s" % str(event.get("message", "unknown error")))
            _metrics.text = (
                "[color=#ff8a8a][b]Evolution stopped:[/b] %s[/color]"
                % str(event.get("message", "unknown error"))
            )
            _job_pid = 0
            _job_kind = ""
            _dead_process_since_ms = -1
            _finish_job_controls()

        "batch_started":
            _build_world_from_geometry(event.get("world_geometry", []))
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
            _dead_process_since_ms = -1
            _finish_job_controls()

        "stream_started":
            _build_world_from_geometry(event.get("world_geometry", []))
            _begin_replay("live")
            _progress_bar.value = 0
            _last_state_time = 0.0
            _set_status(
                "Single-box stream • %s Hz • %sx"
                % [
                    str(event.get("frame_hz", 60)),
                    _format_float(event.get("playback_speed", 1.0), 2),
                ]
            )

        "world_state":
            _queue_replay_state(event.get("state", {}))

        "stream_complete":
            _replay_source_complete = true
            _replay_final_time = float(event.get("simulated_seconds", 0.0))
            _set_status(
                "Single-box replay buffered • playing at %sx"
                % _format_float(_playback_speed, 2)
            )

        "creature_stream_started":
            _build_world_from_geometry(event.get("world_geometry", []))
            _begin_replay("creature")
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
                "%s • %s segments • %s joints • %s brain nodes • %s mutations • %sx"
                % [
                    str(event.get("creature_name", "Creature")),
                    str(event.get("segment_count", 0)),
                    str(event.get("joint_count", 0)),
                    str(event.get("brain_nodes", 0)),
                    str(_current_mutation_count),
                    _format_float(event.get("playback_speed", 1.0), 2),
                ]
            )

        "creature_state":
            _queue_replay_state(event.get("state", {}))

        "creature_stream_complete":
            _replay_source_complete = true
            _replay_final_time = float(event.get("simulated_seconds", 0.0))
            _set_status(
                "Creature replay buffered • playing at %sx"
                % _format_float(_playback_speed, 2)
            )


func _begin_replay(kind: String) -> void:
    _replay_frames.clear()
    _replay_active = true
    _replay_source_complete = false
    _replay_started = false
    _replay_kind = kind
    _replay_clock = 0.0
    _replay_final_time = 0.0


func _reset_replay() -> void:
    _replay_frames.clear()
    _replay_active = false
    _replay_source_complete = false
    _replay_started = false
    _replay_kind = ""
    _replay_clock = 0.0
    _replay_final_time = 0.0


func _queue_replay_state(state_value) -> void:
    if not _replay_active or typeof(state_value) != TYPE_DICTIONARY:
        return

    _replay_frames.append(state_value)

    if _replay_frames.size() == 1:
        _apply_replay_state(state_value)


func _update_replay(delta: float) -> void:
    if not _replay_active or _replay_frames.is_empty():
        return

    if not _replay_started:
        if _replay_frames.size() < 2:
            if _replay_source_complete:
                _apply_replay_state(_replay_frames[0])
                _finish_replay()
            return
        _replay_started = true
        _replay_clock = float(_replay_frames[0].get("simulated_seconds", 0.0))

    _replay_clock += delta * _playback_speed

    while (
        _replay_frames.size() >= 2
        and float(_replay_frames[1].get("simulated_seconds", 0.0)) <= _replay_clock
    ):
        _replay_frames.pop_front()

    if _replay_frames.size() >= 2:
        var first: Dictionary = _replay_frames[0]
        var second: Dictionary = _replay_frames[1]
        var first_time := float(first.get("simulated_seconds", 0.0))
        var second_time := float(second.get("simulated_seconds", first_time))
        var span := second_time - first_time
        var weight := 0.0
        if span > 0.000001:
            weight = clampf((_replay_clock - first_time) / span, 0.0, 1.0)

        if _replay_kind == "creature":
            _show_creature_state(_interpolate_creature_state(first, second, weight))
        else:
            _show_world_state(_interpolate_world_state(first, second, weight))
        return

    _apply_replay_state(_replay_frames[0])

    if _replay_source_complete:
        var final_time := float(
            _replay_frames[0].get("simulated_seconds", _replay_final_time)
        )
        if _replay_clock >= final_time:
            _finish_replay()


func _apply_replay_state(state: Dictionary) -> void:
    if _replay_kind == "creature":
        _show_creature_state(state)
    else:
        _show_world_state(state)


func _finish_replay() -> void:
    var completed_kind := _replay_kind
    var completed_time := _replay_final_time
    _reset_replay()
    _job_pid = 0
    _job_kind = ""
    _dead_process_since_ms = -1
    _job_started_ms = -1
    _job_received_event = false
    _progress_bar.value = 100

    if completed_kind == "creature":
        _set_status(
            "Creature replay complete • %s s • genome ready to save/mutate"
            % _format_float(completed_time, 3)
        )
        if _walkthrough_guided_run_active:
            _walkthrough_guided_run_active = false
            _walkthrough_step = 4
            _refresh_walkthrough()
            call_deferred("_resume_walkthrough_after_test")
    else:
        _set_status(
            "Single-box replay complete • %s s"
            % _format_float(completed_time, 3)
        )

    _finish_job_controls()


func _resume_walkthrough_after_test() -> void:
    if _walkthrough_window != null:
        _walkthrough_window.popup_centered()


func _interpolate_world_state(
    first: Dictionary,
    second: Dictionary,
    weight: float
) -> Dictionary:
    var state := first.duplicate(true)
    state["simulated_seconds"] = lerpf(
        float(first.get("simulated_seconds", 0.0)),
        float(second.get("simulated_seconds", 0.0)),
        weight
    )
    state["step"] = int(round(lerpf(
        float(first.get("step", 0)),
        float(second.get("step", 0)),
        weight
    )))
    state["position"] = _lerp_vector_array(
        first.get("position", [0.0, 0.0, 0.0]),
        second.get("position", [0.0, 0.0, 0.0]),
        weight
    )
    state["linear_velocity"] = _lerp_vector_array(
        first.get("linear_velocity", [0.0, 0.0, 0.0]),
        second.get("linear_velocity", [0.0, 0.0, 0.0]),
        weight
    )
    state["angular_velocity"] = _lerp_vector_array(
        first.get("angular_velocity", [0.0, 0.0, 0.0]),
        second.get("angular_velocity", [0.0, 0.0, 0.0]),
        weight
    )
    state["rotation_xyzw"] = _slerp_quaternion_array(
        first.get("rotation_xyzw", [0.0, 0.0, 0.0, 1.0]),
        second.get("rotation_xyzw", [0.0, 0.0, 0.0, 1.0]),
        weight
    )
    return state


func _interpolate_creature_state(
    first: Dictionary,
    second: Dictionary,
    weight: float
) -> Dictionary:
    var state := first.duplicate(true)
    state["simulated_seconds"] = lerpf(
        float(first.get("simulated_seconds", 0.0)),
        float(second.get("simulated_seconds", 0.0)),
        weight
    )
    state["step"] = int(round(lerpf(
        float(first.get("step", 0)),
        float(second.get("step", 0)),
        weight
    )))

    var first_bodies: Array = first.get("bodies", [])
    var second_bodies: Array = second.get("bodies", [])
    var body_count := mini(first_bodies.size(), second_bodies.size())
    var bodies: Array = []

    for index in range(body_count):
        if (
            typeof(first_bodies[index]) != TYPE_DICTIONARY
            or typeof(second_bodies[index]) != TYPE_DICTIONARY
        ):
            continue

        var first_body: Dictionary = first_bodies[index]
        var second_body: Dictionary = second_bodies[index]
        var body := first_body.duplicate(true)

        body["position"] = _lerp_vector_array(
            first_body.get("position", [0.0, 0.0, 0.0]),
            second_body.get("position", [0.0, 0.0, 0.0]),
            weight
        )
        body["linear_velocity"] = _lerp_vector_array(
            first_body.get("linear_velocity", [0.0, 0.0, 0.0]),
            second_body.get("linear_velocity", [0.0, 0.0, 0.0]),
            weight
        )
        body["angular_velocity"] = _lerp_vector_array(
            first_body.get("angular_velocity", [0.0, 0.0, 0.0]),
            second_body.get("angular_velocity", [0.0, 0.0, 0.0]),
            weight
        )
        body["rotation_xyzw"] = _slerp_quaternion_array(
            first_body.get("rotation_xyzw", [0.0, 0.0, 0.0, 1.0]),
            second_body.get("rotation_xyzw", [0.0, 0.0, 0.0, 1.0]),
            weight
        )
        bodies.append(body)

    state["bodies"] = bodies
    return state


func _lerp_vector_array(first_value, second_value, weight: float) -> Array:
    var first := _vector3_from_array(first_value)
    var second := _vector3_from_array(second_value)
    var value := first.lerp(second, weight)
    return [value.x, value.y, value.z]


func _slerp_quaternion_array(first_value, second_value, weight: float) -> Array:
    var first := _quaternion_from_array(first_value)
    var second := _quaternion_from_array(second_value)
    var value := first.slerp(second, weight).normalized()
    return [value.x, value.y, value.z, value.w]


func _vector3_from_array(value) -> Vector3:
    if typeof(value) == TYPE_ARRAY and value.size() >= 3:
        return Vector3(float(value[0]), float(value[1]), float(value[2]))
    return Vector3.ZERO


func _quaternion_from_array(value) -> Quaternion:
    if typeof(value) == TYPE_ARRAY and value.size() >= 4:
        return Quaternion(
            float(value[0]),
            float(value[1]),
            float(value[2]),
            float(value[3])
        ).normalized()
    return Quaternion.IDENTITY


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
        + "[cell]Sensors used[/cell][cell]%s[/cell]" % str(_current_brain_unique_sensor_count())
        + "[cell]Brain outputs[/cell][cell]%s[/cell]" % str(_current_brain_output_count())
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


func _current_brain_output_count() -> int:
    if _current_genome.is_empty():
        return 0
    var brain_value = _current_genome.get("brain", {})
    if typeof(brain_value) != TYPE_DICTIONARY:
        return 0
    var brain: Dictionary = brain_value
    var outputs: Array = brain.get("outputs", [])
    return outputs.size()


func _current_brain_unique_sensor_count() -> int:
    if _current_genome.is_empty():
        return 0

    var brain_value = _current_genome.get("brain", {})
    if typeof(brain_value) != TYPE_DICTIONARY:
        return 0

    var brain: Dictionary = brain_value
    var outputs: Array = brain.get("outputs", [])
    var sensors: Dictionary = {}

    for output_value in outputs:
        if typeof(output_value) != TYPE_DICTIONARY:
            continue
        var output: Dictionary = output_value
        _collect_expression_sensors(output.get("expression", null), sensors)

    return sensors.size()


func _collect_expression_sensors(expression_value, sensors: Dictionary) -> void:
    if expression_value == null or typeof(expression_value) != TYPE_DICTIONARY:
        return

    var expression: Dictionary = expression_value
    if expression.has("Sensor"):
        sensors[JSON.stringify(expression["Sensor"])] = true
        return

    for key in ["Negate", "Sin", "Cos"]:
        if expression.has(key):
            _collect_expression_sensors(expression[key], sensors)
            return

    for key in ["Add", "Subtract", "Multiply"]:
        if expression.has(key):
            var pair = expression[key]
            if typeof(pair) == TYPE_ARRAY and pair.size() >= 2:
                _collect_expression_sensors(pair[0], sensors)
                _collect_expression_sensors(pair[1], sensors)
            return

    if expression.has("Clamp"):
        var clamp_value = expression["Clamp"]
        if typeof(clamp_value) == TYPE_DICTIONARY:
            _collect_expression_sensors(clamp_value.get("value", null), sensors)


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
