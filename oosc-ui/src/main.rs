pub mod app;
pub mod ui;

use self::app::application::Application;
use anyhow::Error;
use cpal::traits::StreamTrait;

fn main() -> Result<(), Error> {
    let mut app = Application::new()?;
    let stream = app.detach_stream()?;
    stream.play().unwrap();
    app.run()?;
    Ok(())
}
