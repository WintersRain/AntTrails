mod app;
mod camera;
mod colony;
mod components;
mod input;
mod render;
mod spatial;
mod systems;
mod terrain;

use app::App;

fn main() -> anyhow::Result<()> {
    let mut app = App::new()?;
    app.run()
}
