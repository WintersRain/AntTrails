use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hecs::World;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::camera::Camera;
use crate::colony::ColonyState;
use crate::components::{Ant, ColonyMember, Position};
use crate::input::Command;
use crate::render::render_frame;
use crate::spatial::SpatialGrid;
use crate::systems;
use crate::systems::pheromone::PheromoneGrid;
use crate::systems::water::{RainEvent, WaterGrid};
use crate::terrain::Terrain;

const TARGET_FPS: u64 = 30;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FPS);
const NUM_COLONIES: usize = 3;
const NUM_FOOD_SOURCES: usize = 15;
const NUM_APHIDS: usize = 10;
const NUM_WATER_SOURCES: usize = 5;

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    world: World,
    terrain: Terrain,
    colonies: Vec<ColonyState>,
    camera: Camera,
    pheromones: PheromoneGrid,
    water: WaterGrid,
    spatial_grid: SpatialGrid,
    rain_event: Option<RainEvent>,
    running: bool,
    paused: bool,
    tick: u64,
    speed_multiplier: f32,
}

impl App {
    pub fn new() -> Result<Self> {
        // Initialize terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Generate initial terrain
        let seed = fastrand::u32(..);
        let terrain = Terrain::generate(200, 100, seed);

        // Initialize pheromone grid
        let pheromones = PheromoneGrid::new(terrain.width, terrain.height, NUM_COLONIES);

        // Initialize water grid
        let mut water = WaterGrid::new(terrain.width, terrain.height);

        // Initialize ECS world
        let mut world = World::new();

        // Create colonies and spawn initial ants
        let colonies = systems::spawn::spawn_colonies(&mut world, &terrain, NUM_COLONIES);

        // Spawn food sources on surface
        systems::food::spawn_food_sources(&mut world, &terrain, NUM_FOOD_SOURCES);

        // Spawn aphids underground
        systems::aphid::spawn_aphids(&mut world, &terrain, NUM_APHIDS);

        // Spawn some initial water in caves
        systems::water::spawn_water_sources(&mut water, &terrain, NUM_WATER_SOURCES);

        // Ensure queens have Age component
        systems::lifecycle::ensure_queen_ages(&mut world);

        // Initialize spatial grid for neighbor lookups
        let spatial_grid = SpatialGrid::new(terrain.width, terrain.height, 8);

        // Center camera on first colony's queen
        let camera = Camera::new(0, terrain.height as i32 / 5 - 5);

        Ok(Self {
            terminal,
            world,
            terrain,
            colonies,
            camera,
            pheromones,
            water,
            spatial_grid,
            rain_event: None,
            running: true,
            paused: false,
            tick: 0,
            speed_multiplier: 1.0,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut last_frame = Instant::now();

        while self.running {
            // Handle input (non-blocking)
            if event::poll(Duration::from_millis(1))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_input(key.code);
                    }
                }
            }

            // Update game state
            let now = Instant::now();
            if now.duration_since(last_frame) >= FRAME_DURATION {
                if !self.paused {
                    self.update();
                }
                self.render()?;
                last_frame = now;
            }
        }

        self.shutdown()?;
        Ok(())
    }

    fn handle_input(&mut self, key: KeyCode) {
        match Command::from_key(key) {
            Some(Command::Quit) => self.running = false,
            Some(Command::Pause) => self.paused = !self.paused,
            Some(Command::SpeedUp) => {
                self.speed_multiplier = (self.speed_multiplier * 2.0).min(4.0);
            }
            Some(Command::SpeedDown) => {
                self.speed_multiplier = (self.speed_multiplier / 2.0).max(0.5);
            }
            Some(Command::ScrollUp) => self.camera.move_by(0, -1),
            Some(Command::ScrollDown) => self.camera.move_by(0, 1),
            Some(Command::ScrollLeft) => self.camera.move_by(-1, 0),
            Some(Command::ScrollRight) => self.camera.move_by(1, 0),
            None => {}
        }
    }

    fn update(&mut self) {
        // Increment tick counter based on speed
        let ticks_this_frame = self.speed_multiplier as u64;
        for _ in 0..ticks_this_frame {
            self.tick += 1;

            // Rebuild spatial grid for this tick
            self.spatial_grid.clear();
            for (entity, (pos, _ant, member)) in
                self.world.query::<(&Position, &Ant, &ColonyMember)>().iter()
            {
                self.spatial_grid.insert(entity, pos.x, pos.y, member.colony_id);
            }

            // === Phase 1: AI & State Updates ===

            // Dig AI decides what ants should do
            systems::dig::dig_ai_system(&mut self.world, &self.terrain);

            // Combat AI - soldiers respond to danger, workers flee
            systems::combat::soldier_ai_system(&mut self.world, &self.pheromones);
            systems::combat::flee_system(&mut self.world, &self.pheromones);

            // === Phase 2: Movement ===
            systems::movement::movement_system(
                &mut self.world,
                &self.terrain,
                &self.pheromones,
                &self.colonies,
            );

            // === Phase 3: Actions ===

            // Digging (ants in dig state remove soil)
            systems::dig::dig_system(&mut self.world, &mut self.terrain);

            // Foraging (pickup and deposit food)
            systems::food::foraging_system(
                &mut self.world,
                &self.terrain,
                &self.pheromones,
                &mut self.colonies,
            );
            systems::food::check_deposit(&mut self.world, &self.colonies);

            // Combat (every 5 ticks)
            systems::combat::combat_system(&mut self.world, &mut self.pheromones, self.tick, &self.spatial_grid);

            // Aphid farming
            systems::aphid::aphid_system(&mut self.world, &mut self.colonies);

            // === Phase 4: Pheromones ===
            // 1. Decay first (reduces all values per-tick with type-specific rates)
            systems::pheromone::pheromone_decay_system(&mut self.pheromones);

            // 2. Diffuse (spread gradients spatially to create detectable trails)
            self.pheromones.diffuse();

            // 3. Then deposit new pheromone from ant positions (adaptive rates)
            systems::pheromone::pheromone_deposit_system(
                &self.world, &mut self.pheromones, &self.colonies,
            );

            // === Phase 5: Lifecycle ===
            systems::lifecycle::lifecycle_system(&mut self.world, &mut self.colonies, self.tick);

            // Food regrow
            systems::food::food_regrow_system(&mut self.world, self.tick);

            // === Phase 6: Environmental Hazards ===

            // Cave-ins (every 10 ticks)
            if self.tick % 10 == 0 {
                systems::hazard::cave_in_system(&mut self.terrain, &mut self.world);
            }

            // Water physics (every 3 ticks for performance)
            if self.tick % 3 == 0 {
                systems::water::calculate_pressure(&mut self.water, &self.terrain);
                systems::water::water_flow_system(&mut self.water, &self.terrain);
            }

            // Evaporation (every 50 ticks)
            if self.tick % 50 == 0 {
                systems::water::evaporation_system(&mut self.water, &self.terrain);
            }

            // Rain (check every tick, rare event)
            systems::water::rain_system(&mut self.water, &self.terrain, &mut self.rain_event);

            // Drowning
            systems::water::drowning_system(&mut self.world, &self.water);
            systems::water::flee_flood_system(&mut self.world, &self.water);

            // === Phase 7: Cleanup ===
            systems::hazard::cleanup_dead(&mut self.world);
        }
    }

    fn render(&mut self) -> Result<()> {
        // Get view size and clamp camera
        let size = self.terminal.size()?;
        let view_width = size.width.saturating_sub(38) as i32;
        let view_height = size.height.saturating_sub(2) as i32;
        self.camera.clamp_to_bounds(
            self.terrain.width as i32,
            self.terrain.height as i32,
            view_width,
            view_height,
        );

        let world = &self.world;
        let terrain = &self.terrain;
        let water = &self.water;
        let camera = &self.camera;
        let colonies = &self.colonies;
        let tick = self.tick;
        let paused = self.paused;
        let speed = self.speed_multiplier;
        let raining = self.rain_event.is_some();

        self.terminal.draw(|frame| {
            render_frame(
                frame, terrain, water, world, colonies, camera, tick, paused, speed, raining,
            );
        })?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Ensure terminal is restored even on panic
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
