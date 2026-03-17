use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::mpsc;

pub struct AudioInput {
    _stream: cpal::Stream,
    peak_rx: mpsc::Receiver<f32>,
    pub peak_history: VecDeque<f32>,
    pub envelope_history: VecDeque<f32>,
    pub history_len: usize,
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

        let (peak_tx, peak_rx) = mpsc::channel();
        let config = supported_config.config();

        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, peak_tx),
            cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, peak_tx),
            cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, peak_tx),
            fmt => panic!("unsupported sample format: {:?}", fmt),
        };
        stream.play().expect("failed to start audio input stream");

        Self {
            _stream: stream,
            peak_rx,
            peak_history: VecDeque::from(vec![0.0; history_len]),
            envelope_history: VecDeque::from(vec![0.0; history_len]),
            history_len,
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
        for p in self.peak_rx.try_iter() {
            peak = peak.max(p);
        }

        self.peak_history.push_back(peak);
        if self.peak_history.len() > self.history_len {
            self.peak_history.pop_front();
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
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    peak_tx: mpsc::Sender<f32>,
) -> cpal::Stream
where
    T: cpal::Sample + cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let peak = data
                    .iter()
                    .map(|&s| <f32 as cpal::FromSample<T>>::from_sample_(s).abs())
                    .fold(0.0f32, f32::max);
                let _ = peak_tx.send(peak);
            },
            |err| eprintln!("audio input error: {}", err),
            None,
        )
        .expect("failed to build audio input stream")
}
