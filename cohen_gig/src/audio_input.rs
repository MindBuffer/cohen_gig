use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::mpsc;

const WAVEFORM_HISTORY_MULTIPLIER: usize = 16;
pub const MAX_INPUT_GAIN_DB: f32 = 24.0;
const INPUT_GAIN_SOFT_KNEE: f32 = 0.85;

struct AudioAnalysis {
    samples: Vec<f32>,
}

pub struct AudioInput {
    _stream: cpal::Stream,
    analysis_rx: mpsc::Receiver<AudioAnalysis>,
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
    pub fn new(history_len: usize) -> Self {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("no audio input device available");
        let supported_config = device
            .default_input_config()
            .expect("no default input config");

        let (analysis_tx, analysis_rx) = mpsc::channel();
        let config = supported_config.config();
        let waveform_history_len = history_len * WAVEFORM_HISTORY_MULTIPLIER;

        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, analysis_tx),
            cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, analysis_tx),
            cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, analysis_tx),
            fmt => panic!("unsupported sample format: {:?}", fmt),
        };
        stream.play().expect("failed to start audio input stream");

        Self {
            _stream: stream,
            analysis_rx,
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
        }
    }

    pub fn update(&mut self) {
        // Take the max peak from all audio callbacks since last frame.
        let mut peak = 0.0f32;
        let gain = db_to_gain(self.gain_db);
        for analysis in self.analysis_rx.try_iter() {
            for sample in analysis.samples {
                let sample = apply_input_gain(sample, gain);
                peak = peak.max(sample.abs());
                self.waveform_history.push_back(sample);
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

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    analysis_tx: mpsc::Sender<AudioAnalysis>,
) -> cpal::Stream
where
    T: cpal::Sample + cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    device
        .build_input_stream(
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
        .expect("failed to build audio input stream")
}
