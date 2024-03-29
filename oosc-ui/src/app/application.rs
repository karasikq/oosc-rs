use crate::ui::components::{root::Root, Component};
use anyhow::{Context, Result};
use std::{thread, time::Duration};

use super::{config::Config, context};
use cpal::{traits::DeviceTrait, Device};
use crossterm::event::{self, Event, KeyCode};

pub struct Application {
    pub ctx: context::Context,
    pub config: Config,
    root: Root,
    device: Device,
}

impl Application {
    pub fn new() -> Result<Self> {
        let (_, device, config) = context::Context::get_default_device()?;
        let sample_rate = config.sample_rate().0;
        let synth_config = Config {
            channels: 2,
            sample_rate,
            delta_time: 1.0 / sample_rate as f32,
            buffer_size: 2048,
        };
        let ctx = context::Context::build_default(&synth_config)?;
        let root = Root::new(&ctx);
        Ok(Application {
            ctx,
            config: synth_config,
            root,
            device,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.main_loop()
    }

    pub fn detach_stream(&mut self) -> Result<cpal::Stream> {
        let err_fn = |err| println!("An error occurred on stream: {}", err);
        let callbacks = self.ctx.callbacks.get_callbacks();
        let mut total_playback_seconds = 0.;
        let sample_rate = self.config.sample_rate as f32;
        let channels_rate = self.config.channels as f32 * sample_rate;
        Ok(self.device.build_output_stream(
            &self.config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                callbacks
                    .iter()
                    .try_for_each(|callback| -> Result<()> {
                        let mut callback = callback.lock().unwrap();
                        Ok(callback.process_stream(data, total_playback_seconds, sample_rate)?)
                    })
                    .unwrap();
                total_playback_seconds += data.len() as f32 / channels_rate;
            },
            err_fn,
            None,
        )?)
    }

    fn main_loop(&mut self) -> Result<(), anyhow::Error> {
        self.ctx.terminal.write().unwrap().clear()?;
        let area = self.ctx.terminal.write().unwrap().current_buffer_mut().area;
        self.root.resize(area)?;
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
