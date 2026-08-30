use crate::engine::NamEngine;
use crate::params::{NamParams, gain_linear};
use nice_plug::prelude::*;
use std::sync::{Arc, Mutex};

mod engine;
mod params;

pub struct NamPlugin {
    params: Arc<NamParams>,
    engine: Arc<NamEngine>,
    last_capture: bool,
}

impl Default for NamPlugin {
    fn default() -> Self {
        let nam_file = Arc::new(Mutex::new(None));
        let engine = NamEngine::new(Arc::clone(&nam_file));
        let params = Arc::new(NamParams::new(nam_file));
        Self {
            params,
            engine,
            last_capture: false,
        }
    }
}

impl Plugin for NamPlugin {
    const NAME: &'static str = "NAM Plugin";
    const VENDOR: &'static str = "hedgieinsocks";
    const URL: &'static str = "https://github.com/hedgieinsocks/nam-clap";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(1),
        main_output_channels: NonZeroU32::new(1),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type Editor = ();
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn activate(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.engine.set_format(
            f64::from(buffer_config.sample_rate),
            buffer_config.max_buffer_size,
        );

        if !self.engine.model_loaded()
            && let Some(path) = self.params.nam_file.lock().unwrap().clone()
        {
            self.engine.spawn_load_path(path);
        }

        self.last_capture = self.params.capture.value();

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let capture = self.params.capture.value();
        if capture && !self.last_capture {
            self.engine.spawn_pick_file();
        }
        self.last_capture = capture;

        let input_gain = gain_linear(&self.params.input_gain);
        let output_gain = gain_linear(&self.params.output_gain);

        if let [channel, ..] = buffer.as_slice() {
            self.engine.process(channel, input_gain, output_gain);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for NamPlugin {
    const CLAP_ID: &'static str = "io.github.hedgieinsocks.nam-clap";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Neural Amp Modeler CLAP plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Mono];
}

nice_export_clap!(NamPlugin);
