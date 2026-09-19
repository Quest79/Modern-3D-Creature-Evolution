use serde::{Deserialize, Serialize};

const TAU: f32 = std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerrainKind {
    #[default]
    Flat,
    Slope,
    Hills,
    Stairs,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldBoxKind {
    Ground,
    PitFloor,
    Wall,
    Block,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorldBox {
    pub kind: WorldBoxKind,
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    pub rotation_radians: [f32; 3],
    pub friction: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorldConfig {
    pub gravity: [f32; 3],
    pub ground_half_extents: [f32; 3],
    pub ground_friction: f32,
    pub terrain: TerrainKind,
    pub seed: u64,
    pub slope_degrees: f32,
    pub hill_height: f32,
    pub hill_wavelength: f32,
    pub stair_height: f32,
    pub stair_depth: f32,
    pub walls_enabled: bool,
    pub blocks_enabled: bool,
    pub gaps_enabled: bool,
    pub pits_enabled: bool,
    pub obstacle_count: usize,
    pub obstacle_spacing: f32,
    pub obstacle_size: f32,
    pub gap_width: f32,
    pub pit_depth: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            gravity: [0.0, -9.81, 0.0],
            ground_half_extents: [50.0, 0.1, 12.0],
            ground_friction: 1.0,
            terrain: TerrainKind::Flat,
            seed: 1,
            slope_degrees: 8.0,
            hill_height: 0.75,
            hill_wavelength: 8.0,
            stair_height: 0.25,
            stair_depth: 1.25,
            walls_enabled: false,
            blocks_enabled: false,
            gaps_enabled: false,
            pits_enabled: false,
            obstacle_count: 6,
            obstacle_spacing: 5.0,
            obstacle_size: 1.0,
            gap_width: 1.5,
            pit_depth: 1.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeatureKind {
    Wall,
    Block,
    Gap,
    Pit,
}

#[derive(Clone, Copy, Debug)]
struct Feature {
    kind: FeatureKind,
    x: f32,
    z: f32,
    size_scale: f32,
}

impl WorldConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.gravity.iter().any(|value| !value.is_finite()) {
            return Err("world gravity must contain only finite values".into());
        }
        if self
            .ground_half_extents
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err("ground_half_extents must be finite and greater than 0".into());
        }
        if !self.ground_friction.is_finite() || self.ground_friction < 0.0 {
            return Err("ground_friction must be finite and non-negative".into());
        }
        if !self.slope_degrees.is_finite() || !(-35.0..=35.0).contains(&self.slope_degrees) {
            return Err("slope_degrees must be between -35 and 35".into());
        }
        if !self.hill_height.is_finite() || !(0.0..=10.0).contains(&self.hill_height) {
            return Err("hill_height must be between 0 and 10".into());
        }
        if !self.hill_wavelength.is_finite() || !(1.0..=100.0).contains(&self.hill_wavelength) {
            return Err("hill_wavelength must be between 1 and 100".into());
        }
        if !self.stair_height.is_finite() || !(0.01..=5.0).contains(&self.stair_height) {
            return Err("stair_height must be between 0.01 and 5".into());
        }
        if !self.stair_depth.is_finite() || !(0.1..=20.0).contains(&self.stair_depth) {
            return Err("stair_depth must be between 0.1 and 20".into());
        }
        if self.obstacle_count > 100 {
            return Err("obstacle_count must not exceed 100".into());
        }
        if !self.obstacle_spacing.is_finite() || !(1.0..=50.0).contains(&self.obstacle_spacing) {
            return Err("obstacle_spacing must be between 1 and 50".into());
        }
        if !self.obstacle_size.is_finite() || !(0.1..=10.0).contains(&self.obstacle_size) {
            return Err("obstacle_size must be between 0.1 and 10".into());
        }
        if !self.gap_width.is_finite() || !(0.1..=10.0).contains(&self.gap_width) {
            return Err("gap_width must be between 0.1 and 10".into());
        }
        if !self.pit_depth.is_finite() || !(0.1..=20.0).contains(&self.pit_depth) {
            return Err("pit_depth must be between 0.1 and 20".into());
        }
        Ok(())
    }

    pub fn geometry(&self) -> Vec<WorldBox> {
        let features = self.features();
        let has_cutouts = features
            .iter()
            .any(|feature| matches!(feature.kind, FeatureKind::Gap | FeatureKind::Pit));

        let mut geometry = Vec::new();

        if matches!(self.terrain, TerrainKind::Flat | TerrainKind::Slope) && !has_cutouts {
            let angle = if self.terrain == TerrainKind::Slope {
                self.slope_degrees.to_radians()
            } else {
                0.0
            };
            geometry.push(WorldBox {
                kind: WorldBoxKind::Ground,
                center: [0.0, -self.ground_half_extents[1], 0.0],
                half_extents: self.ground_half_extents,
                rotation_radians: [0.0, 0.0, angle],
                friction: self.ground_friction,
            });
        } else {
            self.push_tiled_ground(&features, &mut geometry);
        }

        for feature in features {
            match feature.kind {
                FeatureKind::Wall => {
                    let ground_y = self.terrain_height(feature.x);
                    let height = (self.obstacle_size * feature.size_scale).max(0.2);
                    geometry.push(WorldBox {
                        kind: WorldBoxKind::Wall,
                        center: [feature.x, ground_y + height, feature.z],
                        half_extents: [
                            (self.obstacle_size * 0.18).max(0.08),
                            height,
                            (self.ground_half_extents[2] * 0.30).max(0.5),
                        ],
                        rotation_radians: [0.0, 0.0, 0.0],
                        friction: self.ground_friction,
                    });
                }
                FeatureKind::Block => {
                    let ground_y = self.terrain_height(feature.x);
                    let half = (self.obstacle_size * 0.5 * feature.size_scale).max(0.1);
                    geometry.push(WorldBox {
                        kind: WorldBoxKind::Block,
                        center: [feature.x, ground_y + half, feature.z],
                        half_extents: [half, half, half],
                        rotation_radians: [0.0, 0.0, 0.0],
                        friction: self.ground_friction,
                    });
                }
                FeatureKind::Gap | FeatureKind::Pit => {}
            }
        }

        geometry
    }

    pub fn surface_height_at(&self, x: f32, _z: f32) -> Option<f32> {
        let features = self.features();
        for feature in &features {
            let half_width = self.feature_half_width(*feature);
            if (x - feature.x).abs() <= half_width {
                match feature.kind {
                    FeatureKind::Gap => return None,
                    FeatureKind::Pit => return Some(self.terrain_height(x) - self.pit_depth),
                    FeatureKind::Wall | FeatureKind::Block => {}
                }
            }
        }
        Some(self.terrain_height(x))
    }

    fn terrain_height(&self, x: f32) -> f32 {
        match self.terrain {
            TerrainKind::Flat => 0.0,
            TerrainKind::Slope => self.slope_degrees.to_radians().tan() * x,
            TerrainKind::Hills => self.hill_height * (TAU * x / self.hill_wavelength).sin(),
            TerrainKind::Stairs => {
                if x <= 0.0 {
                    0.0
                } else {
                    (x / self.stair_depth).floor() * self.stair_height
                }
            }
        }
    }

    fn terrain_angle(&self, x: f32) -> f32 {
        match self.terrain {
            TerrainKind::Flat | TerrainKind::Stairs => 0.0,
            TerrainKind::Slope => self.slope_degrees.to_radians(),
            TerrainKind::Hills => {
                let derivative = self.hill_height
                    * (TAU / self.hill_wavelength)
                    * (TAU * x / self.hill_wavelength).cos();
                derivative.atan()
            }
        }
    }

    fn push_tiled_ground(&self, features: &[Feature], geometry: &mut Vec<WorldBox>) {
        let tile_width = 0.5_f32;
        let half_x = self.ground_half_extents[0];
        let half_y = self.ground_half_extents[1];
        let half_z = self.ground_half_extents[2];
        let mut x = -half_x + tile_width * 0.5;

        while x < half_x {
            let mut skip = false;
            let mut pit_drop = 0.0;
            for feature in features {
                let half_width = self.feature_half_width(*feature);
                if (x - feature.x).abs() <= half_width {
                    match feature.kind {
                        FeatureKind::Gap => {
                            skip = true;
                            break;
                        }
                        FeatureKind::Pit => pit_drop = pit_drop.max(self.pit_depth),
                        FeatureKind::Wall | FeatureKind::Block => {}
                    }
                }
            }

            if !skip {
                let angle = self.terrain_angle(x);
                let height = self.terrain_height(x) - pit_drop;
                geometry.push(WorldBox {
                    kind: if pit_drop > 0.0 {
                        WorldBoxKind::PitFloor
                    } else {
                        WorldBoxKind::Ground
                    },
                    center: [x, height - half_y, 0.0],
                    half_extents: [tile_width * 0.52, half_y, half_z],
                    rotation_radians: [0.0, 0.0, angle],
                    friction: self.ground_friction,
                });
            }

            x += tile_width;
        }
    }

    fn feature_half_width(&self, feature: Feature) -> f32 {
        match feature.kind {
            FeatureKind::Gap => self.gap_width * 0.5 * feature.size_scale,
            FeatureKind::Pit => self.obstacle_size * 0.75 * feature.size_scale,
            FeatureKind::Wall | FeatureKind::Block => self.obstacle_size * 0.5 * feature.size_scale,
        }
    }

    fn features(&self) -> Vec<Feature> {
        let mut enabled = Vec::new();
        if self.walls_enabled {
            enabled.push(FeatureKind::Wall);
        }
        if self.blocks_enabled {
            enabled.push(FeatureKind::Block);
        }
        if self.gaps_enabled {
            enabled.push(FeatureKind::Gap);
        }
        if self.pits_enabled {
            enabled.push(FeatureKind::Pit);
        }
        if enabled.is_empty() || self.obstacle_count == 0 {
            return Vec::new();
        }

        let mut rng = WorldRng::new(self.seed);
        let mut features = Vec::with_capacity(self.obstacle_count);
        for index in 0..self.obstacle_count {
            let kind = enabled[index % enabled.len()];
            let jitter = rng.range(-0.15, 0.15) * self.obstacle_spacing;
            let x = 6.0 + index as f32 * self.obstacle_spacing + jitter;
            if x >= self.ground_half_extents[0] - 2.0 {
                break;
            }

            let usable_z = (self.ground_half_extents[2] - self.obstacle_size * 1.5).max(0.0);
            let z = match kind {
                FeatureKind::Gap | FeatureKind::Pit => 0.0,
                FeatureKind::Wall | FeatureKind::Block => rng.range(-usable_z, usable_z),
            };
            let size_scale = rng.range(0.75, 1.25);

            features.push(Feature {
                kind,
                x,
                z,
                size_scale,
            });
        }

        features
    }
}

struct WorldRng {
    state: u64,
}

impl WorldRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn unit(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32;
        bits as f32 / ((1_u32 << 24) - 1) as f32
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.unit()
    }
}

#[cfg(test)]
mod tests {
    use super::{TerrainKind, WorldBoxKind, WorldConfig};

    #[test]
    fn flat_world_has_one_ground_box_by_default() {
        let geometry = WorldConfig::default().geometry();
        assert_eq!(geometry.len(), 1);
        assert_eq!(geometry[0].kind, WorldBoxKind::Ground);
    }

    #[test]
    fn procedural_geometry_is_deterministic() {
        let world = WorldConfig {
            terrain: TerrainKind::Hills,
            walls_enabled: true,
            blocks_enabled: true,
            gaps_enabled: true,
            pits_enabled: true,
            ..WorldConfig::default()
        };
        assert_eq!(world.geometry(), world.geometry());
    }

    #[test]
    fn gap_removes_surface() {
        let world = WorldConfig {
            gaps_enabled: true,
            obstacle_count: 1,
            ..WorldConfig::default()
        };
        let gap_x = world
            .geometry()
            .iter()
            .find(|shape| shape.kind == WorldBoxKind::Ground)
            .map(|_| ())
            .is_some();
        assert!(gap_x);
        assert!(world.geometry().len() > 1);
    }
}
