use neural_amp_modeler_rs::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

pub struct NamEngine {
    inner: Mutex<Inner>,
    sample_rate: AtomicU32,
    max_block: AtomicU32,
    nam_file: Arc<Mutex<Option<PathBuf>>>,
}

struct Inner {
    model: Option<Box<dyn NamModel>>,
    scratch: Vec<f32>,
}

impl NamEngine {
    pub fn new(nam_file: Arc<Mutex<Option<PathBuf>>>) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner {
                model: None,
                scratch: Vec::new(),
            }),
            sample_rate: AtomicU32::new(48_000),
            max_block: AtomicU32::new(4096),
            nam_file,
        })
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap()
    }

    fn try_lock(&self) -> Option<MutexGuard<'_, Inner>> {
        self.inner.try_lock().ok()
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn set_format(&self, sample_rate: f64, max_block: u32) {
        let sample_rate = sample_rate as u32;
        self.sample_rate.store(sample_rate, Ordering::Relaxed);
        self.max_block.store(max_block, Ordering::Relaxed);
        let mut inner = self.lock();
        inner.scratch.resize(max_block as usize, 0.0);
        if let Some(model) = inner.model.as_mut() {
            let _ = model.reset(sample_rate, max_block as usize);
        }
    }

    pub fn model_loaded(&self) -> bool {
        self.lock().model.is_some()
    }

    pub fn spawn_pick_file(self: &Arc<Self>) {
        let engine = Arc::clone(self);
        thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("NAM File", &["nam"])
                .pick_file()
            {
                engine.load_path(path);
            }
        });
    }

    pub fn spawn_load_path(self: &Arc<Self>, path: PathBuf) {
        let engine = Arc::clone(self);
        thread::spawn(move || engine.load_path(path));
    }

    fn load_path(&self, path: PathBuf) {
        let sys = SystemSnapshot::capture();
        let sample_rate = self.sample_rate.load(Ordering::Relaxed);
        let max_block = self.max_block.load(Ordering::Relaxed) as usize;

        let Ok(mut pair) = load_and_build_model(&path, &sys, false, LoadOptions::default()) else {
            return;
        };
        let Some(mut model) = pair.model_l.take() else {
            return;
        };
        if model.reset(sample_rate, max_block).is_err() {
            return;
        }

        let mut inner = self.lock();
        inner.model = Some(model);
        drop(inner);

        *self.nam_file.lock().unwrap() = Some(path);
    }

    pub fn process(&self, buf: &mut [f32], input_gain: f32, output_gain: f32) {
        let Some(mut inner) = self.try_lock() else {
            for s in buf.iter_mut() {
                *s *= input_gain * output_gain;
            }
            return;
        };

        for s in buf.iter_mut() {
            *s *= input_gain;
        }

        let len = buf.len().min(inner.scratch.len());

        let Inner { model, scratch, .. } = &mut *inner;
        if let Some(model) = model.as_mut() {
            let head = &mut buf[..len];
            model.process(head, &mut scratch[..len]);
            head.copy_from_slice(&scratch[..len]);
        }

        for s in buf.iter_mut() {
            *s *= output_gain;
        }
    }
}
