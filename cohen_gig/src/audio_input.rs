use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const WAVEFORM_HISTORY_MULTIPLIER: usize = 16;
pub const MAX_INPUT_GAIN_DB: f32 = 24.0;
const INPUT_GAIN_SOFT_KNEE: f32 = 0.85;
const DEVICE_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

struct AudioAnalysis {
    samples: Vec<f32>,
}

struct AudioRuntime {
    _stream: cpal::Stream,
    analysis_rx: mpsc::Receiver<AudioAnalysis>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AudioDeviceInfo {
    name: String,
    label: String,
    is_builtin: bool,
}

pub struct AudioInput {
    runtime: Option<AudioRuntime>,
    available_devices: Vec<AudioDeviceInfo>,
    selected_device_name: Option<String>,
    device_error: Option<String>,
    last_device_refresh: Instant,
    pub peak_history: VecDeque<f32>,
    pub waveform_history: VecDeque<f32>,
    pub envelope_history: VecDeque<f32>,
    pub history_len: usize,
    waveform_history_len: usize,
    pub gain_db: f32,
    pub threshold: f32,
    pub attack: f32,
    pub hold: f32,
    pub release: f32,
    pub envelope: f32,
    hold_remaining: f32,
    // Modulation depth per shader param, set by Korg rotary knobs A-D.
    pub mod_amp1: f32,
    pub mod_amp2: f32,
    pub mod_amp3: f32,
    pub mod_amp4: f32,
}

impl AudioInput {
    pub fn new(history_len: usize, preferred_device_name: String) -> Self {
        let waveform_history_len = history_len * WAVEFORM_HISTORY_MULTIPLIER;
        let mut audio_input = Self {
            runtime: None,
            available_devices: Vec::new(),
            selected_device_name: None,
            device_error: None,
            last_device_refresh: Instant::now(),
            peak_history: VecDeque::from(vec![0.0; history_len]),
            waveform_history: VecDeque::from(vec![0.0; waveform_history_len]),
            envelope_history: VecDeque::from(vec![0.0; history_len]),
            history_len,
            waveform_history_len,
            gain_db: 0.0,
            threshold: 0.1,
            attack: 0.01,
            hold: 0.1,
            release: 0.3,
            envelope: 0.0,
            hold_remaining: 0.0,
            mod_amp1: 0.0,
            mod_amp2: 0.0,
            mod_amp3: 0.0,
            mod_amp4: 0.0,
        };

        audio_input.refresh_available_devices();

        let preferred_device_name = preferred_device_name.trim();
        if !preferred_device_name.is_empty() {
            let _ = audio_input.switch_to_device(preferred_device_name.to_string());
        }
        audio_input.ensure_selected_device();

        audio_input
    }

    pub fn update(&mut self) {
        self.refresh_available_devices_if_needed();

        // Take the max peak from all audio callbacks since last frame.
        let mut peak = 0.0f32;
        let gain = db_to_gain(self.gain_db);
        if let Some(runtime) = self.runtime.as_mut() {
            for analysis in runtime.analysis_rx.try_iter() {
                for sample in analysis.samples {
                    let sample = apply_input_gain(sample, gain);
                    peak = peak.max(sample.abs());
                    self.waveform_history.push_back(sample);
                }
            }
        }

        self.peak_history.push_back(peak);
        if self.peak_history.len() > self.history_len {
            self.peak_history.pop_front();
        }
        while self.waveform_history.len() > self.waveform_history_len {
            self.waveform_history.pop_front();
        }

        // Envelope follower with hold: when peak crosses threshold, reset hold
        // timer. While hold is active, keep attacking. Only release once hold expires.
        let dt = 1.0 / 60.0; // assuming ~60fps
        if peak > self.threshold {
            self.hold_remaining = self.hold;
            let coeff = 1.0 - (-1.0 / (self.attack * 60.0)).exp();
            self.envelope += (1.0 - self.envelope) * coeff;
        } else if self.hold_remaining > 0.0 {
            self.hold_remaining -= dt;
            let coeff = 1.0 - (-1.0 / (self.attack * 60.0)).exp();
            self.envelope += (1.0 - self.envelope) * coeff;
        } else {
            let coeff = 1.0 - (-1.0 / (self.release * 60.0)).exp();
            self.envelope *= 1.0 - coeff;
        }

        self.envelope_history.push_back(self.envelope);
        if self.envelope_history.len() > self.history_len {
            self.envelope_history.pop_front();
        }
    }

    pub fn gain_multiplier(&self) -> f32 {
        db_to_gain(self.gain_db)
    }

    pub fn available_device_labels(&self) -> Vec<String> {
        self.available_devices
            .iter()
            .map(|device| device.label.clone())
            .collect()
    }

    pub fn selected_device_index(&self) -> Option<usize> {
        let selected_device_name = self.selected_device_name.as_deref()?;
        self.available_devices
            .iter()
            .position(|device| device.name == selected_device_name)
    }

    pub fn select_device(&mut self, index: usize) -> Option<String> {
        let device_name = self.available_devices.get(index)?.name.clone();

        if self.selected_device_name.as_deref() == Some(device_name.as_str())
            && self.runtime.is_some()
        {
            return Some(device_name);
        }

        self.switch_to_device(device_name.clone()).ok()?;
        Some(device_name)
    }

    pub fn device_error(&self) -> Option<&str> {
        self.device_error.as_deref()
    }

    fn refresh_available_devices_if_needed(&mut self) {
        if self.last_device_refresh.elapsed() >= DEVICE_REFRESH_INTERVAL {
            self.refresh_available_devices();
        }
    }

    fn refresh_available_devices(&mut self) {
        self.available_devices = enumerate_input_devices();
        self.last_device_refresh = Instant::now();
        self.ensure_selected_device();
    }

    fn ensure_selected_device(&mut self) {
        if let Some(selected_device_name) = self.selected_device_name.clone() {
            let device_is_available = self
                .available_devices
                .iter()
                .any(|device| device.name == selected_device_name);

            if device_is_available && self.runtime.is_some() {
                return;
            }

            if device_is_available && self.switch_to_device(selected_device_name).is_ok() {
                return;
            }

            self.runtime = None;
            self.selected_device_name = None;
        }

        if let Some(device_name) = preferred_fallback_device_name(&self.available_devices) {
            let _ = self.switch_to_device(device_name);
        } else {
            self.runtime = None;
            self.selected_device_name = None;
            self.device_error = Some("No audio input device available".to_string());
            self.reset_analysis_state();
        }
    }

    fn switch_to_device(&mut self, device_name: String) -> Result<(), String> {
        let runtime = match build_runtime_for_device(&device_name) {
            Ok(runtime) => runtime,
            Err(err) => {
                self.device_error = Some(err.clone());
                return Err(err);
            }
        };
        self.runtime = Some(runtime);
        self.selected_device_name = Some(device_name);
        self.device_error = None;
        self.reset_analysis_state();
        Ok(())
    }

    fn reset_analysis_state(&mut self) {
        self.peak_history = VecDeque::from(vec![0.0; self.history_len]);
        self.waveform_history = VecDeque::from(vec![0.0; self.waveform_history_len]);
        self.envelope_history = VecDeque::from(vec![0.0; self.history_len]);
        self.envelope = 0.0;
        self.hold_remaining = 0.0;
    }
}

fn db_to_gain(gain_db: f32) -> f32 {
    10.0f32.powf(gain_db.clamp(0.0, MAX_INPUT_GAIN_DB) / 20.0)
}

fn apply_input_gain(sample: f32, gain: f32) -> f32 {
    let boosted = sample * gain;
    let abs = boosted.abs();
    if abs <= INPUT_GAIN_SOFT_KNEE {
        boosted
    } else {
        let knee_range = 1.0 - INPUT_GAIN_SOFT_KNEE;
        let compressed = INPUT_GAIN_SOFT_KNEE
            + knee_range * (1.0 - (-(abs - INPUT_GAIN_SOFT_KNEE) / knee_range).exp());
        boosted.signum() * compressed.min(1.0)
    }
}

fn enumerate_input_devices() -> Vec<AudioDeviceInfo> {
    let host = cpal::default_host();
    let mut devices = match host.input_devices() {
        Ok(devices) => devices
            .filter_map(|device| {
                let name = device.name().ok()?;
                device.default_input_config().ok()?;

                let is_builtin = is_builtin_microphone(&name);
                let label = if is_builtin {
                    format!("{} (Built-in)", name)
                } else {
                    name.clone()
                };

                Some(AudioDeviceInfo {
                    name,
                    label,
                    is_builtin,
                })
            })
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    devices.sort_by(|left, right| {
        right.is_builtin.cmp(&left.is_builtin).then_with(|| {
            left.name
                .to_ascii_lowercase()
                .cmp(&right.name.to_ascii_lowercase())
        })
    });

    devices
}

fn preferred_fallback_device_name(devices: &[AudioDeviceInfo]) -> Option<String> {
    if let Some(device) = devices.iter().find(|device| device.is_builtin) {
        return Some(device.name.clone());
    }

    if let Some(default_device_name) = default_input_device_name() {
        if let Some(device) = devices
            .iter()
            .find(|device| device.name == default_device_name)
        {
            return Some(device.name.clone());
        }
    }

    devices.first().map(|device| device.name.clone())
}

fn default_input_device_name() -> Option<String> {
    cpal::default_host().default_input_device()?.name().ok()
}

fn is_builtin_microphone(device_name: &str) -> bool {
    let device_name = device_name.to_ascii_lowercase();
    device_name.contains("macbook pro microphone")
        || device_name.contains("built-in microphone")
        || device_name.contains("built in microphone")
        || device_name.contains("internal microphone")
        || (device_name.contains("microphone") && device_name.contains("macbook"))
}

fn build_runtime_for_device(device_name: &str) -> Result<AudioRuntime, String> {
    let host = cpal::default_host();
    let device = find_input_device_by_name(&host, device_name)
        .ok_or_else(|| format!("Audio input '{}' is no longer available", device_name))?;
    let supported_config = device
        .default_input_config()
        .map_err(|err| format!("Couldn't read audio config for '{}': {}", device_name, err))?;

    let (analysis_tx, analysis_rx) = mpsc::channel();
    let config = supported_config.config();
    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, analysis_tx),
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, analysis_tx),
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, analysis_tx),
        fmt => {
            return Err(format!(
                "Audio input '{}' uses unsupported sample format {:?}",
                device_name, fmt
            ));
        }
    }
    .map_err(|err| format!("Couldn't build audio stream for '{}': {}", device_name, err))?;

    stream
        .play()
        .map_err(|err| format!("Couldn't start audio stream for '{}': {}", device_name, err))?;

    Ok(AudioRuntime {
        _stream: stream,
        analysis_rx,
    })
}

fn find_input_device_by_name(host: &cpal::Host, device_name: &str) -> Option<cpal::Device> {
    let devices = host.input_devices().ok()?;
    for device in devices {
        let Ok(name) = device.name() else {
            continue;
        };
        if name == device_name {
            return Some(device);
        }
    }
    None
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    analysis_tx: mpsc::Sender<AudioAnalysis>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut samples = Vec::with_capacity(data.len() / channels.max(1));
            for frame in data.chunks(channels.max(1)) {
                let sample = frame
                    .iter()
                    .map(|&s| <f32 as cpal::FromSample<T>>::from_sample_(s))
                    .sum::<f32>()
                    / frame.len() as f32;
                samples.push(sample.clamp(-1.0, 1.0));
            }
            let _ = analysis_tx.send(AudioAnalysis { samples });
        },
        |err| eprintln!("audio input error: {}", err),
        None,
    )
}
