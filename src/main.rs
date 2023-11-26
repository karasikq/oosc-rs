use std::{time::Duration, thread};

use cpal::traits::{DeviceTrait, StreamTrait};

use crate::app::context;

use self::{app::application::Application, error::Error};

pub mod app;
pub mod core;
pub mod effects;
pub mod error;
pub mod utils;

fn main() -> Result<(), Error> {
    let mut app = Application::new()?;
    let (_, device, config) = context::Context::get_default_device(&app.config)?;
    let err_fn = |err| println!("an error occurred on stream: {}", err);
    let syn = app.ctx.synthesizer.clone();
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut s = syn.lock().unwrap();
                let buf = s.output();
                if buf.is_err() {
                    println!("{}", buf.err().unwrap().to_string());
                    return;
                }
                let buf = buf.unwrap();
                let mut b = buf.iter(0).unwrap();
                for frame in data.chunks_exact_mut(2) {
                    let s = b.next().unwrap();
                    for f in frame.iter_mut() {
                        *f = s;
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    app.run()?;
    // thread::sleep(Duration::from_millis(2000));
    Ok(())
}
