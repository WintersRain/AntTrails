use hecs::World;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::camera::Camera;
use crate::colony::{ColonyState, COLONY_COLORS};
use crate::components::{Ant, AntRole, AntState, Aphid, Carrying, ColonyMember, FoodSource, Position};
use crate::systems::pheromone::{PheromoneGrid, PheromoneType};
use crate::systems::water::WaterGrid;
use crate::terrain::{Terrain, TerrainType};

pub fn render_frame(
    frame: &mut Frame,
    terrain: &Terrain,
    water: &WaterGrid,
    world: &World,
    colonies: &[ColonyState],
    camera: &Camera,
    tick: u64,
    paused: bool,
    speed: f32,
    raining: bool,
    pheromones: &PheromoneGrid,
    show_pheromones: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(36)])
        .split(frame.area());

    render_terrain(frame, chunks[0], terrain, water, world, camera, pheromones, show_pheromones);
    render_stats(
        frame,
        chunks[1],
        world,
        colonies,
        terrain.seed,
        camera,
        tick,
        paused,
        speed,
        raining,
    );
}

fn render_terrain(
    frame: &mut Frame,
    area: Rect,
    terrain: &Terrain,
    water: &WaterGrid,
    world: &World,
    camera: &Camera,
    pheromones: &PheromoneGrid,
    show_pheromones: bool,
) {
    let block = Block::default().borders(Borders::ALL).title(" World ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let view_width = inner.width as i32;
    let view_height = inner.height as i32;

    // Collect entity positions for rendering
    let mut entity_chars: std::collections::HashMap<(i32, i32), (char, Color)> =
        std::collections::HashMap::new();

    // Food sources
    for (_entity, (pos, food)) in world.query::<(&Position, &FoodSource)>().iter() {
        if food.amount > 0 {
            entity_chars.insert((pos.x, pos.y), ('♠', Color::LightGreen));
        }
    }

    // Aphids
    for (_entity, (pos, aphid)) in world.query::<(&Position, &Aphid)>().iter() {
        let color = aphid
            .colony_owner
            .map(|c| COLONY_COLORS[c as usize % COLONY_COLORS.len()])
            .unwrap_or(Color::Green);
        entity_chars.insert((pos.x, pos.y), ('a', color));
    }

    // Ants (rendered last to be on top)
    for (_entity, (pos, ant, member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        let (ch, color) = ant_visual(ant, member.colony_id, world.get::<&Carrying>(_entity).is_ok());
        entity_chars.insert((pos.x, pos.y), (ch, color));
    }

    // Render terrain, water, and entities
    for dy in 0..view_height {
        for dx in 0..view_width {
            let world_x = camera.x + dx;
            let world_y = camera.y + dy;

            // Check for entity at this position
            if let Some((ch, color)) = entity_chars.get(&(world_x, world_y)) {
                let x = inner.x + dx as u16;
                let y = inner.y + dy as u16;
                if x < inner.x + inner.width && y < inner.y + inner.height {
                    frame
                        .buffer_mut()
                        .set_string(x, y, ch.to_string(), Style::default().fg(*color));
                }
                continue;
            }

            // Check for water
            let water_depth = water.depth(world_x, world_y);
            if water_depth > 0 {
                let (ch, color) = water_visual(water_depth);
                let x = inner.x + dx as u16;
                let y = inner.y + dy as u16;
                if x < inner.x + inner.width && y < inner.y + inner.height {
                    frame
                        .buffer_mut()
                        .set_string(x, y, ch.to_string(), Style::default().fg(color));
                }
                continue;
            }

            // Render terrain
            let (ch, color) = match terrain.get(world_x, world_y) {
                Some(TerrainType::Air) => (' ', Color::Reset),
                Some(TerrainType::Tunnel) => ('·', Color::Rgb(80, 60, 40)),
                Some(TerrainType::Soil) => ('░', Color::Rgb(139, 90, 43)),
                Some(TerrainType::SoilDense) => ('▒', Color::Rgb(101, 67, 33)),
                Some(TerrainType::Rock) => ('█', Color::DarkGray),
                Some(TerrainType::Surface) => ('▀', Color::Green),
                None => (' ', Color::Reset),
            };

            let x = inner.x + dx as u16;
            let y = inner.y + dy as u16;

            if x < inner.x + inner.width && y < inner.y + inner.height {
                // Add pheromone background layer if enabled
                let style = if show_pheromones {
                    let mut bg_r: u8 = 0;
                    let mut bg_g: u8 = 0;
                    let mut bg_b: u8 = 0;

                    for c in 0..pheromones.max_colonies as u8 {
                        let food_val = pheromones.get(world_x, world_y, c, PheromoneType::Food);
                        let home_val = pheromones.get(world_x, world_y, c, PheromoneType::Home);
                        let danger_val = pheromones.get(world_x, world_y, c, PheromoneType::Danger);

                        // Map intensity (0.0-1.0) to color (0-120)
                        // Cap at 120 to preserve foreground character visibility
                        if food_val > 0.05 {
                            bg_g = bg_g.max((food_val.clamp(0.0, 1.0) * 120.0) as u8);
                        }
                        if home_val > 0.05 {
                            bg_b = bg_b.max((home_val.clamp(0.0, 1.0) * 120.0) as u8);
                        }
                        if danger_val > 0.05 {
                            bg_r = bg_r.max((danger_val.clamp(0.0, 1.0) * 120.0) as u8);
                        }
                    }

                    if bg_r > 0 || bg_g > 0 || bg_b > 0 {
                        Style::default().fg(color).bg(Color::Rgb(bg_r, bg_g, bg_b))
                    } else {
                        Style::default().fg(color)
                    }
                } else {
                    Style::default().fg(color)
                };

                frame
                    .buffer_mut()
                    .set_string(x, y, ch.to_string(), style);
            }
        }
    }
}

/// Get visual representation of an ant
fn ant_visual(ant: &Ant, colony_id: u8, carrying: bool) -> (char, Color) {
    let color = COLONY_COLORS[colony_id as usize % COLONY_COLORS.len()];

    let ch = match ant.role {
        AntRole::Queen => 'Q',
        AntRole::Worker => {
            if carrying {
                '●' // Carrying food
            } else {
                match ant.state {
                    AntState::Digging => '⚒',
                    AntState::Fleeing => '!',
                    _ => '•',
                }
            }
        }
        AntRole::Soldier => {
            if ant.state == AntState::Fighting {
                '⚔'
            } else {
                '*'
            }
        }
        AntRole::Egg => '°',
        AntRole::Larvae => 'o',
    };

    (ch, color)
}

/// Get visual representation of water
fn water_visual(depth: u8) -> (char, Color) {
    let ch = match depth {
        1 => '·',
        2 => '~',
        3 => '≈',
        4..=5 => '▄',
        6..=7 => '█',
        _ => ' ',
    };

    // Color intensity based on depth
    let blue = 100 + (depth as u8 * 20).min(155);
    let color = Color::Rgb(0, 0, blue);

    (ch, color)
}

fn render_stats(
    frame: &mut Frame,
    area: Rect,
    world: &World,
    colonies: &[ColonyState],
    seed: u32,
    camera: &Camera,
    tick: u64,
    paused: bool,
    speed: f32,
    raining: bool,
) {
    let block = Block::default().borders(Borders::ALL).title(" AntTrails ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let status = if paused {
        "[PAUSED]"
    } else if raining {
        "[RAIN]"
    } else {
        ""
    };
    let speed_str = format!("{:.1}x", speed);

    let mut lines = vec![
        Line::from(vec![
            Span::raw("Speed: "),
            Span::styled(speed_str, Style::default().fg(Color::Yellow)),
            Span::raw(format!(" Tick: {}", tick)),
        ]),
        Line::from(vec![
            Span::raw("Seed: "),
            Span::styled(format!("{}", seed), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::raw("View: "),
            Span::styled(
                format!("({}, {})", camera.x, camera.y),
                Style::default().fg(Color::Gray),
            ),
            Span::raw(format!(" {}", status)),
        ]),
        Line::raw(""),
        Line::styled("─ Controls ─", Style::default().fg(Color::Cyan)),
        Line::raw("[Space] Pause/Resume"),
        Line::raw("[+/-]   Speed up/down"),
        Line::raw("[Arrows] Scroll"),
        Line::raw("[P]     Pheromones"),
        Line::raw("[Q]     Quit"),
        Line::raw(""),
        Line::styled("─ Legend ─", Style::default().fg(Color::Cyan)),
        Line::raw("Q=Queen •=Worker *=Soldier"),
        Line::raw("°=Egg o=Larvae a=Aphid"),
        Line::raw("♠=Food ~=Water"),
        Line::raw(""),
        Line::styled("─ Colonies ─", Style::default().fg(Color::Cyan)),
    ];

    // Add colony info
    for colony in colonies {
        let pop = colony.population_summary(world);
        let color = colony.color;

        lines.push(Line::from(vec![
            Span::styled(
                format!("Colony {} ", colony.id + 1),
                Style::default().fg(color),
            ),
            Span::raw(format!("Pop: {}", pop.total())),
        ]));
        lines.push(Line::from(vec![
            Span::raw(" Q:"),
            Span::raw(format!("{} ", pop.queens)),
            Span::raw("W:"),
            Span::raw(format!("{} ", pop.workers)),
            Span::raw("S:"),
            Span::raw(format!("{}", pop.soldiers)),
        ]));
        lines.push(Line::from(vec![
            Span::raw(" E:"),
            Span::raw(format!("{} ", pop.eggs)),
            Span::raw("L:"),
            Span::raw(format!("{} ", pop.larvae)),
            Span::raw("Food:"),
            Span::styled(
                format!("{}", colony.food_stored),
                Style::default().fg(Color::Green),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
