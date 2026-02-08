use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Quit,
    Pause,
    SpeedUp,
    SpeedDown,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    TogglePheromones,
}

impl Command {
    pub fn from_key(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => Some(Command::Quit),
            KeyCode::Char(' ') => Some(Command::Pause),
            KeyCode::Char('+') | KeyCode::Char('=') => Some(Command::SpeedUp),
            KeyCode::Char('-') | KeyCode::Char('_') => Some(Command::SpeedDown),
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => Some(Command::ScrollUp),
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => Some(Command::ScrollDown),
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => Some(Command::ScrollLeft),
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => Some(Command::ScrollRight),
            KeyCode::Char('p') | KeyCode::Char('P') => Some(Command::TogglePheromones),
            _ => None,
        }
    }
}
