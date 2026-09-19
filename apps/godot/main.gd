extends Node

const EVENT_PORT_START := 47821
const EVENT_PORT_TRIES := 32
const SETTINGS_PATH := "user://settings.cfg"
const DEFAULT_HUD_WIDTH := 510.0
const MIN_HUD_WIDTH := 320.0
const MAX_HUD_WIDTH := 900.0
const HUD_RESIZE_HANDLE_WIDTH := 8.0

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
var _settings_button: Button
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
var _font_size_spin: SpinBox
var _camera_speed_spin: SpinBox
var _mouse_sensitivity_spin: SpinBox
var _playback_speed_spin: SpinBox
var _hud_resize_handle: ColorRect

var _probe_mesh: MeshInstance3D
var _camera: Camera3D
var _world_meshes: Array[MeshInstance3D] = []
var _creature_meshes: Dictionary = {}
var _current_genome: Dictionary = {}
var _current_genome_source := ""
var _current_mutation_count := 0
var _has_evolution_champion := false
var _champion_world: Dictionary = {}
var _champion_motor_strength := 1.0

var _font_size := 16
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

var _save_dialog: FileDialog
var _load_dialog: FileDialog
var _experiment_save_dialog: FileDialog
var _experiment_load_dialog: FileDialog

var _udp: PacketPeerUDP
var _event_port := 0
var _job_pid := 0
var _job_kind := ""
var _dead_process_since_ms := -1
var _last_state_time := 0.0

var _replay_frames: Array = []
var _replay_active := false
var _replay_source_complete := false
var _replay_started := false
var _replay_kind := ""
var _replay_clock := 0.0
var _replay_final_time := 0.0


func _ready() -> void:
    _load_settings()
    _ui_theme = Theme.new()
    _ui_theme.default_font_size = _font_size

    _build_3d_preview()
    _build_ui()
    _build_settings_window()
    _build_file_dialogs()
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
        elif _dead_process_since_ms < 0:
            # Give final UDP packets a moment to arrive before declaring the
            # process dead. Fast benchmark jobs can otherwise race the GUI.
            _dead_process_since_ms = Time.get_ticks_msec()
        elif Time.get_ticks_msec() - _dead_process_since_ms >= 300:
            var ended_kind := _job_kind
            _job_pid = 0
            _dead_process_since_ms = -1

            if (
                (ended_kind == "live" or ended_kind == "creature")
                and _replay_active
            ):
                # The producer is expected to finish before slow-motion
                # playback. If its tiny completion packet was dropped, use the
                # newest buffered frame as the replay end instead of aborting.
                if not _replay_source_complete and not _replay_frames.is_empty():
                    _replay_source_complete = true
                    _replay_final_time = float(
                        _replay_frames.back().get("simulated_seconds", 0.0)
                    )
            elif ended_kind == "evolution" and _load_genome_file(_evolution_champion_path()):
                _job_kind = ""
                _has_evolution_champion = true
                _current_genome_source = "latest evolution champion"
                _build_creature_from_genome(_current_genome)
                _set_status("Evolution process ended • latest champion recovered.")
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

    _settings_button = Button.new()
    _settings_button.text = "⚙ Settings"
    _settings_button.pressed.connect(_on_settings_pressed)
    header_row.add_child(_settings_button)

    var subtitle := Label.new()
    subtitle.text = "Step 6 • Experiment Timeline / Scheduling"
    subtitle.modulate = Color(0.72, 0.78, 0.88)
    column.add_child(subtitle)

    column.add_child(HSeparator.new())

    var editor_heading := Label.new()
    editor_heading.text = "Creature generation"
    _section_headings.append(editor_heading)
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

    var experiment_row := HBoxContainer.new()
    experiment_row.add_theme_constant_override("separation", 6)
    column.add_child(experiment_row)

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

    column.add_child(HSeparator.new())

    var evolution_heading := Label.new()
    evolution_heading.text = "Evolution"
    _section_headings.append(evolution_heading)
    column.add_child(evolution_heading)

    _population_spin = _add_number_row(column, "Population", 2, 10000, 50, 1)
    _generations_spin = _add_number_row(column, "Generations", 1, 10000, 100, 1)
    _tournament_spin = _add_number_row(column, "Tournament size", 1, 1000, 7, 1)
    _elite_spin = _add_number_row(column, "Elite kept", 1, 999, 2, 1)
    _crossover_spin = _add_number_row(column, "Brain crossover", 0.0, 1.0, 0.5, 0.05)
    _crossover_spin.tooltip_text = "Chance that a child receives a brain subtree from a second selected parent."
    _evolution_mutations_spin = _add_number_row(column, "Mutations / child", 1, 500, 8, 1)
    _structural_mutation_spin = _add_number_row(
        column,
        "Structural mutation chance",
        0.0,
        1.0,
        _structural_mutation_chance,
        0.01
    )
    _motor_strength_spin = _add_number_row(
        column, "Motor strength", 0.0, 20.0, _motor_strength, 0.05
    )
    _motor_strength_spin.suffix = "x"
    _trials_spin = _add_number_row(
        column, "Trials / creature", 1, 100, _trials_per_creature, 1
    )

    var aggregation_row := HBoxContainer.new()
    column.add_child(aggregation_row)
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

    var fitness_heading := Label.new()
    fitness_heading.text = "Fitness weights"
    _section_headings.append(fitness_heading)
    column.add_child(fitness_heading)

    _fitness_distance_spin = _add_number_row(
        column, "Distance", -100.0, 100.0, _fitness_distance_weight, 0.05
    )
    _fitness_speed_spin = _add_number_row(
        column, "Average speed", -100.0, 100.0, _fitness_speed_weight, 0.05
    )
    _fitness_upright_spin = _add_number_row(
        column, "Upright", -100.0, 100.0, _fitness_upright_weight, 0.05
    )
    _fitness_stability_spin = _add_number_row(
        column, "Stability", -100.0, 100.0, _fitness_stability_weight, 0.05
    )
    _fitness_energy_spin = _add_number_row(
        column, "Energy / effort", -100.0, 100.0, _fitness_energy_weight, 0.01
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

    column.add_child(HSeparator.new())

    var world_heading := Label.new()
    world_heading.text = "World / terrain"
    _section_headings.append(world_heading)
    column.add_child(world_heading)

    var terrain_row := HBoxContainer.new()
    column.add_child(terrain_row)
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

    _world_seed_spin = _add_number_row(column, "World seed", 0, 999999999, _world_seed, 1)
    _gravity_x_spin = _add_number_row(column, "Gravity X", -30.0, 30.0, _gravity_x, 0.1)
    _gravity_y_spin = _add_number_row(column, "Gravity Y", -30.0, 30.0, _gravity_y, 0.1)
    _gravity_z_spin = _add_number_row(column, "Gravity Z", -30.0, 30.0, _gravity_z, 0.1)
    _ground_friction_spin = _add_number_row(
        column, "Ground friction", 0.0, 5.0, _ground_friction, 0.05
    )
    _slope_spin = _add_number_row(column, "Slope angle (deg)", -35.0, 35.0, _slope_degrees, 0.5)
    _hill_height_spin = _add_number_row(column, "Hill height", 0.0, 10.0, _hill_height, 0.05)
    _hill_wavelength_spin = _add_number_row(
        column, "Hill wavelength", 1.0, 100.0, _hill_wavelength, 0.25
    )
    _stair_height_spin = _add_number_row(
        column, "Stair height", 0.01, 5.0, _stair_height, 0.05
    )
    _stair_depth_spin = _add_number_row(
        column, "Stair depth", 0.1, 20.0, _stair_depth, 0.05
    )

    var obstacle_label := Label.new()
    obstacle_label.text = "Obstacle types"
    column.add_child(obstacle_label)

    var obstacle_row_a := HBoxContainer.new()
    obstacle_row_a.add_theme_constant_override("separation", 10)
    column.add_child(obstacle_row_a)
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
    column.add_child(obstacle_row_b)
    _gaps_check = CheckBox.new()
    _gaps_check.text = "Gaps"
    _gaps_check.button_pressed = _gaps_enabled
    obstacle_row_b.add_child(_gaps_check)
    _pits_check = CheckBox.new()
    _pits_check.text = "Pits"
    _pits_check.button_pressed = _pits_enabled
    obstacle_row_b.add_child(_pits_check)

    _obstacle_count_spin = _add_number_row(
        column, "Obstacle count", 0, 100, _obstacle_count, 1
    )
    _obstacle_spacing_spin = _add_number_row(
        column, "Obstacle spacing", 1.0, 50.0, _obstacle_spacing, 0.25
    )
    _obstacle_size_spin = _add_number_row(
        column, "Obstacle size", 0.1, 10.0, _obstacle_size, 0.05
    )
    _gap_width_spin = _add_number_row(
        column, "Gap width", 0.1, 10.0, _gap_width, 0.05
    )
    _pit_depth_spin = _add_number_row(
        column, "Pit depth", 0.1, 20.0, _pit_depth, 0.05
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

    column.add_child(HSeparator.new())

    var timeline_heading := Label.new()
    timeline_heading.text = "Experiment timeline"
    _section_headings.append(timeline_heading)
    column.add_child(timeline_heading)

    var timeline_help := Label.new()
    timeline_help.text = (
        "Keyframes snapshot the current world, fitness, population, mutation, "
        + "motor, duration, and trial settings."
    )
    timeline_help.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    timeline_help.modulate = Color(0.70, 0.76, 0.86)
    column.add_child(timeline_help)

    _timeline_generation_spin = _add_number_row(
        column, "Keyframe generation", 1, 10000, 1, 1
    )

    var condition_row := HBoxContainer.new()
    column.add_child(condition_row)
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
        column, "Condition threshold", -100000.0, 100000.0, 1.0, 0.1
    )
    _timeline_condition_value_spin.editable = false

    _timeline_list = ItemList.new()
    _timeline_list.custom_minimum_size = Vector2(0, 115)
    _timeline_list.select_mode = ItemList.SELECT_SINGLE
    column.add_child(_timeline_list)

    var timeline_buttons := HBoxContainer.new()
    timeline_buttons.add_theme_constant_override("separation", 6)
    column.add_child(timeline_buttons)

    var add_keyframe_button := Button.new()
    add_keyframe_button.text = "Add Snapshot"
    add_keyframe_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    add_keyframe_button.pressed.connect(_on_add_timeline_keyframe)
    timeline_buttons.add_child(add_keyframe_button)

    var remove_keyframe_button := Button.new()
    remove_keyframe_button.text = "Remove"
    remove_keyframe_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    remove_keyframe_button.pressed.connect(_on_remove_timeline_keyframe)
    timeline_buttons.add_child(remove_keyframe_button)

    var clear_timeline_button := Button.new()
    clear_timeline_button.text = "Clear"
    clear_timeline_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    clear_timeline_button.pressed.connect(_on_clear_timeline)
    timeline_buttons.add_child(clear_timeline_button)

    _refresh_timeline_list()

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
    _section_headings.append(sim_heading)
    column.add_child(sim_heading)

    _seconds_spin = _add_number_row(column, "Seconds / simulation", 0.1, 120.0, 8.0, 0.1)
    _dt_spin = _add_number_row(column, "Physics dt (seconds)", 0.0001, 0.05, 1.0 / 120.0, 0.0001)
    _playback_speed_spin = _add_number_row(
        column,
        "Playback speed",
        0.01,
        2.0,
        _playback_speed,
        0.01
    )
    _playback_speed_spin.suffix = "x"
    _playback_speed_spin.tooltip_text = "Live viewer speed. 0.01x = 100× slower, 2.00x = 2× faster."
    _playback_speed_spin.value_changed.connect(_on_playback_speed_changed)
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

    _apply_font_size()


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

    _font_size_spin = _add_number_row(column, "Font size", 10, 32, _font_size, 1)
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


func _refresh_world_preview() -> void:
    if not _backend_exists():
        return

    var output: Array = []
    var exit_code := OS.execute(
        _backend_path(),
        PackedStringArray([
            "world-geometry",
            "--world-json",
            JSON.stringify(_world_config_dictionary()),
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

    _build_world_from_geometry(parsed.get("geometry", []))


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
                material.albedo_color = Color(0.15, 0.18, 0.22)
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
    if config.load(SETTINGS_PATH) != OK:
        return

    _font_size = int(config.get_value("ui", "font_size", _font_size))
    _font_size = clampi(_font_size, 10, 32)
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
    _motor_strength = clampf(_motor_strength, 0.0, 20.0)
    _trials_per_creature = int(
        config.get_value("evolution", "trials_per_creature", _trials_per_creature)
    )
    _trials_per_creature = clampi(_trials_per_creature, 1, 100)
    _trial_aggregation = str(
        config.get_value("evolution", "trial_aggregation", _trial_aggregation)
    )
    if not _trial_aggregation in ["mean", "median", "worst", "best"]:
        _trial_aggregation = "mean"
    _structural_mutation_chance = float(
        config.get_value(
            "evolution",
            "structural_mutation_chance",
            _structural_mutation_chance
        )
    )
    _structural_mutation_chance = clampf(_structural_mutation_chance, 0.0, 1.0)

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
    config.set_value("timeline", "json", JSON.stringify(_timeline_config_dictionary()))
    config.set_value("camera", "move_speed", _camera_move_speed)
    config.set_value(
        "camera",
        "mouse_sensitivity_degrees",
        _mouse_sensitivity_degrees
    )
    config.save(SETTINGS_PATH)


func _update_layout() -> void:
    var viewport_size := get_viewport().get_visible_rect().size
    var max_for_window := maxf(
        MIN_HUD_WIDTH,
        minf(MAX_HUD_WIDTH, viewport_size.x - 240.0)
    )
    _visible_hud_width = clampf(_hud_width, MIN_HUD_WIDTH, max_for_window)

    if is_instance_valid(_hud_panel):
        _hud_panel.position = Vector2.ZERO
        _hud_panel.size = Vector2(_visible_hud_width, viewport_size.y)

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
    _save_dialog.current_file = "creature_genome.json"
    _save_dialog.file_selected.connect(_on_save_file_selected)
    add_child(_save_dialog)

    _load_dialog = FileDialog.new()
    _load_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _load_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
    _load_dialog.filters = PackedStringArray(["*.json ; Creature Genome JSON"])
    _load_dialog.file_selected.connect(_on_load_file_selected)
    add_child(_load_dialog)

    _experiment_save_dialog = FileDialog.new()
    _experiment_save_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _experiment_save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
    _experiment_save_dialog.filters = PackedStringArray(["*.evo ; EvoLab Experiment"])
    _experiment_save_dialog.current_file = "experiment.evo"
    _experiment_save_dialog.file_selected.connect(_on_experiment_save_file_selected)
    add_child(_experiment_save_dialog)

    _experiment_load_dialog = FileDialog.new()
    _experiment_load_dialog.access = FileDialog.ACCESS_FILESYSTEM
    _experiment_load_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
    _experiment_load_dialog.filters = PackedStringArray(["*.evo ; EvoLab Experiment"])
    _experiment_load_dialog.file_selected.connect(_on_experiment_load_file_selected)
    add_child(_experiment_load_dialog)


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
        "--playback-speed", "%.2f" % _playback_speed,
        "--seed", str(int(_seed_spin.value)),
        "--max-segments", str(int(_max_segments_spin.value)),
        "--world-json", _world_json(),
        "--motor-strength", str(_motor_strength_spin.value),
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

    var champion_path := _evolution_champion_path()
    if FileAccess.file_exists(champion_path):
        DirAccess.remove_absolute(champion_path)

    var args := PackedStringArray([
        "evolve",
        "--population", str(int(_population_spin.value)),
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
        "--timeline-json", _timeline_json(),
        "--fitness-distance", str(_fitness_distance_spin.value),
        "--fitness-speed", str(_fitness_speed_spin.value),
        "--fitness-upright", str(_fitness_upright_spin.value),
        "--fitness-stability", str(_fitness_stability_spin.value),
        "--fitness-energy", str(_fitness_energy_spin.value),
        "--world-json", _world_json(),
        "--event-port", str(_event_port),
        "--champion-output", champion_path,
    ])

    if not _current_genome.is_empty():
        var parent_path := ProjectSettings.globalize_path("user://evolution_parent.json")
        if not _write_genome_file(parent_path, _current_genome):
            _set_status("Could not write evolution parent genome.")
            return
        args.append_array(PackedStringArray(["--genome", parent_path]))

    if _start_job("evolution", args):
        _set_status("Evolution running • weighted fitness enabled")


func _on_watch_champion_pressed() -> void:
    if _job_pid > 0 or _current_genome.is_empty():
        return

    var champion_path := _evolution_champion_path()
    if not FileAccess.file_exists(champion_path):
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


func _evolution_champion_path() -> String:
    return ProjectSettings.globalize_path("user://evolution_champion.json")


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
        "--playback-speed", "%.2f" % _playback_speed,
        "--world-json", _world_json(),
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
        "--world-json", _world_json(),
        "--event-port", str(_event_port),
    ])

    if _start_job("batch", args):
        _set_status("Parallel benchmark running...")


func _start_job(kind: String, args: PackedStringArray) -> bool:
    if kind == "live" or kind == "creature":
        _begin_replay(kind)

    _job_kind = kind
    _dead_process_since_ms = -1
    _job_pid = OS.create_process(_backend_path(), args, false)

    if _job_pid <= 0:
        _job_pid = 0
        _job_kind = ""
        _set_status("Failed to start Rust simulator.")
        return false

    _set_run_buttons_disabled(true)
    _set_world_controls_enabled(false)
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
    _reset_replay()
    _finish_job_controls()


func _finish_job_controls() -> void:
    _set_run_buttons_disabled(false)
    _set_world_controls_enabled(true)
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
            _build_world_from_geometry(event.get("world_geometry", []))
            _progress_bar.value = 0
            _set_status(
                "Evolution started • %s creatures × %s generations"
                % [str(event.get("population", 0)), str(event.get("generations", 0))]
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
            _set_status(
                "Generation %d / %d • best score %.4f • average %.4f"
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
                + "[cell]Evaluations[/cell][cell]%s[/cell]" % str(event.get("evaluations_completed", 0))
                + "[/table]"
            )

        "evolution_complete":
            var champion_file := str(event.get("champion_file", ""))
            if champion_file != "":
                _load_genome_file(champion_file)

            _current_genome_source = "evolution champion"
            _build_creature_from_genome(_current_genome)
            _has_evolution_champion = not _current_genome.is_empty()
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
    _progress_bar.value = 100

    if completed_kind == "creature":
        _set_status(
            "Creature replay complete • %s s • genome ready to save/mutate"
            % _format_float(completed_time, 3)
        )
    else:
        _set_status(
            "Single-box replay complete • %s s"
            % _format_float(completed_time, 3)
        )

    _finish_job_controls()


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
