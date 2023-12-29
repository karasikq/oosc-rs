use crate::ui::components::{root::Root, Component};
use anyhow::{Context, Result};
use std::{thread, time::Duration};

use super::{config::Config, context};
use cpal::traits::DeviceTrait;
use crossterm::event::{self, Event, KeyCode};

pub struct Application {
    pub ctx: context::Context,
    pub config: Config,
    root: Root,
}

impl Application {
    pub fn new() -> Result<Self> {
        let config = Config {
            channels: 2,
            sample_rate: 48000,
            delta_time: 1.0 / 48000.0,
            buffer_size: 512,
        };
        let ctx = context::Context::build_default(&config)?;
        let mut root = Root::new(ctx.synthesizer.clone());
        root.resize(ctx.terminal.write().unwrap().current_buffer_mut().area)?;
        Ok(Application { ctx, config, root })
    }

    pub fn run(&mut self) -> Result<()> {
        self.main_loop()
    }

    pub fn detach_stream(&mut self) -> Result<cpal::Stream> {
        let (_, device, config) = context::Context::get_default_device(&self.config)?;
        let err_fn = |err| println!("an error occurred on stream: {}", err);
        let callbacks = self.ctx.callbacks.get_callbacks();
        let mut total_playback_seconds = 0.;
        let delta = self.config.buffer_size as f32 / self.config.sample_rate as f32;
        Ok(device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                callbacks
                    .iter()
                    .try_for_each(|callback| -> Result<()> {
                        let mut callback = callback.lock().unwrap();
                        Ok(callback.process_stream(data, total_playback_seconds)?)
                    })
                    .unwrap();
                total_playback_seconds += delta;
            },
            err_fn,
            None,
        )?)
    }

    fn main_loop(&mut self) -> Result<(), anyhow::Error> {
        loop {
            thread::sleep(Duration::from_millis(16));
            if !self.read_events()? {
                break;
            }
            self.ctx.terminal.write().unwrap().draw(|f| {
                let _ = self.root.draw(f, f.size());
            })?;
        }
        context::restore_terminal()
    }

    fn read_events(&mut self) -> Result<bool> {
        let _syn = self.ctx.synthesizer.lock().unwrap();
        if event::poll(Duration::from_millis(0)).context("event poll failed")? {
            let event = event::read().context("event read failed")?;
            let event_copy = event.clone();
            self.root.handle_events(Some(event_copy))?;
            if let Event::Key(key) = event {
                return Ok(!matches!(key.code, KeyCode::Char('q')));
            }
        }
        Ok(true)
    }
}
