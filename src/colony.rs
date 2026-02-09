#![allow(dead_code)]

use ratatui::style::Color;

/// Predefined colony colors
pub const COLONY_COLORS: [Color; 6] = [
    Color::Red,
    Color::Blue,
    Color::Yellow,
    Color::Magenta,
    Color::Cyan,
    Color::Green,
];

#[derive(Debug)]
pub struct ColonyState {
    pub id: u8,
    pub color: Color,
    pub food_stored: u32,
    pub queen_alive: bool,
    pub home_x: i32,
    pub home_y: i32,
}

impl ColonyState {
    pub fn new(id: u8, home_x: i32, home_y: i32, initial_food: u32) -> Self {
        Self {
            id,
            color: COLONY_COLORS[id as usize % COLONY_COLORS.len()],
            food_stored: initial_food,
            queen_alive: true,
            home_x,
            home_y,
        }
    }

    pub fn population_summary(&self, world: &hecs::World) -> PopulationCount {
        use crate::components::{Ant, AntRole, ColonyMember};

        let mut count = PopulationCount::default();

        for (_entity, (ant, member)) in world.query::<(&Ant, &ColonyMember)>().iter() {
            if member.colony_id != self.id {
                continue;
            }
            match ant.role {
                AntRole::Queen => count.queens += 1,
                AntRole::Worker => count.workers += 1,
                AntRole::Soldier => count.soldiers += 1,
                AntRole::Egg => count.eggs += 1,
                AntRole::Larvae => count.larvae += 1,
            }
        }

        count
    }
}

#[derive(Debug, Default)]
pub struct PopulationCount {
    pub queens: u16,
    pub workers: u16,
    pub soldiers: u16,
    pub eggs: u16,
    pub larvae: u16,
}

impl PopulationCount {
    pub fn total(&self) -> u16 {
        self.queens + self.workers + self.soldiers + self.eggs + self.larvae
    }
}
