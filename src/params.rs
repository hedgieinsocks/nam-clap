use nice_plug::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const GAIN_MIN_DB: f32 = -20.0;
const GAIN_MAX_DB: f32 = 20.0;

#[derive(Params)]
pub struct NamParams {
    #[id = "input_gain"]
    pub input_gain: FloatParam,
    #[id = "output_gain"]
    pub output_gain: FloatParam,
    #[id = "load"]
    pub load: BoolParam,
    #[persist = "nam_file"]
    pub nam_file: Arc<Mutex<Option<PathBuf>>>,
}

fn gain_param(name: &str) -> FloatParam {
    FloatParam::new(
        name,
        0.0,
        FloatRange::Linear {
            min: GAIN_MIN_DB,
            max: GAIN_MAX_DB,
        },
    )
    .with_unit(" dB")
    .with_value_to_string(formatters::v2s_f32_rounded(1))
}

impl NamParams {
    pub fn new(nam_file: Arc<Mutex<Option<PathBuf>>>) -> Self {
        let display_path = Arc::clone(&nam_file);
        Self {
            input_gain: gain_param("Input Gain"),
            output_gain: gain_param("Output Gain"),
            load: BoolParam::new("NAM File", false).with_value_to_string(Arc::new(move |_| {
                nam_file_display_name(display_path.lock().unwrap().as_ref())
            })),
            nam_file,
        }
    }
}

pub fn nam_file_display_name(path: Option<&PathBuf>) -> String {
    match path {
        Some(path) => path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string(),
        None => "None".to_string(),
    }
}

pub fn gain_linear(param: &FloatParam) -> f32 {
    util::db_to_gain(param.smoothed.next())
}
