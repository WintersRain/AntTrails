use hecs::Entity;

/// Spatial hash grid for O(1) neighbor lookups.
/// Divides the map into cells of `cell_size` tiles each.
/// Rebuilt from scratch each tick (O(N) rebuild, O(K) query).
pub struct SpatialGrid {
    cells: Vec<Vec<(Entity, i32, i32, u8)>>, // entity, x, y, colony_id
    width: usize,  // grid width in cells
    height: usize, // grid height in cells
    cell_size: i32,
}

impl SpatialGrid {
    /// Create a new spatial grid for a world of the given pixel dimensions.
    /// cell_size determines the granularity (8 is good for combat adjacency checks).
    pub fn new(world_width: usize, world_height: usize, cell_size: i32) -> Self {
        let width = (world_width as i32 / cell_size + 1) as usize;
        let height = (world_height as i32 / cell_size + 1) as usize;
        Self {
            cells: vec![Vec::new(); width * height],
            width,
            height,
            cell_size,
        }
    }

    /// Clear all entities from the grid. Called at the start of each tick.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    /// Insert an entity at position (x, y) with its colony_id.
    pub fn insert(&mut self, entity: Entity, x: i32, y: i32, colony_id: u8) {
        let cx = (x / self.cell_size) as usize;
        let cy = (y / self.cell_size) as usize;
        if cx < self.width && cy < self.height {
            self.cells[cy * self.width + cx].push((entity, x, y, colony_id));
        }
    }

    /// Query all entities in the cell containing (x, y) and its 8 neighbors.
    /// Returns a Vec of (entity, x, y, colony_id) tuples.
    pub fn query_nearby(&self, x: i32, y: i32) -> Vec<(Entity, i32, i32, u8)> {
        let cx = (x / self.cell_size) as isize;
        let cy = (y / self.cell_size) as isize;
        let w = self.width as isize;
        let h = self.height as isize;

        let mut results = Vec::new();
        for dy in -1..=1isize {
            for dx in -1..=1isize {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && nx < w && ny >= 0 && ny < h {
                    let idx = ny as usize * self.width + nx as usize;
                    results.extend_from_slice(&self.cells[idx]);
                }
            }
        }
        results
    }
}
