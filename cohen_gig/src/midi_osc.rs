use nannou_osc as osc;
use nannou::prelude::*;
use std::collections::VecDeque;

pub const NOTE_ON: u8 = 144;
pub const NOTE_OFF: u8 = 128;

pub const PORT: u16 = 9000;


#[derive(Debug, Clone)]
pub struct Note {
    pub pitch: u8,
    pub velocity: u8,
}

#[derive(Debug, Clone)]
pub struct MidiPianoFrame {
	pub notes: Vec<Note>,
	pub sustain_pedal: f32,
	pub soft_pedal: f32,	
}

pub struct MidiOsc {
    receiver: osc::Receiver,
    pub midi_buffer: VecDeque<MidiPianoFrame>,
    pub midi_buffer_frame_len: usize,
    pub smoothing_speed: f32,
    pub max_unique_pitches: usize,
    pub midi_cv: f32,
    pub mod_amp1: f32,
    pub mod_amp2: f32,
    pub mod_amp3: f32,
    pub mod_amp4: f32,
}

impl MidiOsc {
    pub fn new() -> Self {
        let midi_buffer_frame_len = 512;
        let midi_buffer = default_midi_buffer(midi_buffer_frame_len);

        // Bind an `osc::Receiver` to a port.
        let receiver = osc::receiver(PORT).unwrap();

        Self {
            receiver, 
            midi_buffer, 
            midi_buffer_frame_len,
            max_unique_pitches: 10,
            smoothing_speed: 0.15, 
            midi_cv: 0.0,
            mod_amp1: 1.0,
            mod_amp2: 1.0,
            mod_amp3: 1.0,
            mod_amp4: 1.0,
        }
    }

    pub fn update(&mut self) {
        // --------------------- OPEN SOUND CONTROL
        let mut notes = Vec::new();
        
        // Receive any pending osc packets.
        let mut received_packets = Vec::new();
        for (packet, addr) in self.receiver.try_iter() {
            received_packets.push((addr, packet));
        }

        let all_msgs = received_packets
            .drain(..)
            .flat_map(|(_addr, packet)| packet.into_msgs());

        for message in all_msgs {
            let args = match &message.args {
                None => continue,
                Some(args) => args,
            };
            let on_off = args[0].clone().int().unwrap() as u8;
            let pitch = args[1].clone().int().unwrap() as u8;
            let velocity = args[2].clone().int().unwrap() as u8;
            if on_off == NOTE_ON {
                notes.push( Note {
                    pitch, 
                    velocity
                });
            } 
        }
        
        let mpf = MidiPianoFrame {
            notes,
            sustain_pedal: 0.0,
            soft_pedal: 0.0,
        };

        self.midi_buffer.push_front(mpf);
        self.midi_buffer.pop_back();

        //println!("{:#?}", model.midi_buffer);

        //analyse(&self.midi_buffer);

        let unique_pitches = unique_pitches(&self.midi_buffer).len();
        let target_midi_cv = map_range(unique_pitches, 0 , self.max_unique_pitches, 0.0, 1.0);

        let smoothing_speed = if target_midi_cv < self.midi_cv {
            self.smoothing_speed * 0.1
        } else {
            self.smoothing_speed
        };
        self.midi_cv = self.midi_cv * (1.0 - smoothing_speed)
            + target_midi_cv * smoothing_speed;
    }
}


// Each frame, we look for new key pressed events. If there are some, we push back either a single 
// Note, or a “chord” of notes as a single unifying type into the ring buffer at the current frame.

// RingBuffer Methods:
// - Remove duplicates 
// - Clear
// - Resize

// Analysis Methods:
// - Min Max Range (Pitch and Velocity)
// - Min, Max, Median Average (Pitch and Velocity)
// - Single notes vs Chords 
// - Number of unique pitched notes
// - Number of unique chords
// - Overall rhythmic density (how dense or sparse are notes inputed into the ring buffer)
// - Number of accents / loud notes (how many notes over some threshold of volume exist in the buffer)
// - Sustained vs non-sustained notes (how many notes had the sustain pedal down whilst played into the buffer)

pub fn notes<'a>(buffer: &'a VecDeque<MidiPianoFrame>) -> impl 'a + Iterator<Item = Vec<&Note>> {
    buffer.iter().filter_map(|frame| if frame.notes.len() != 0 {
        let mut notes = Vec::new();
        for note in &frame.notes {
            notes.push(note);
        }
        Some(notes)
    } else {
        None
    })
}

pub fn num_chords(buffer: &VecDeque<MidiPianoFrame>) -> usize {
    let num_chords: Vec<_> = notes(&buffer).filter_map(|v| {
        if v.len() > 1 {
            Some(())
        } else {
            None
        }
    }).collect();
    num_chords.len()
}

pub fn unique_pitches<'a>(buffer: &'a VecDeque<MidiPianoFrame>) -> Vec<u8> {
    let mut pitches = Vec::new();
    notes(&buffer).for_each(|notes| {
        for note in &notes {
            pitches.push(note.pitch);
        }
    });

    pitches.sort_unstable();
    pitches.dedup();
    pitches
}

pub fn min_pitch<'a>(buffer: &'a VecDeque<MidiPianoFrame>) -> Option<u8> {
    let pitches = unique_pitches(&buffer);
    match pitches.first() {
        Some(min) => Some(*min),
        None => None,
    }
}

pub fn max_pitch<'a>(buffer: &'a VecDeque<MidiPianoFrame>) -> Option<u8> {
    let pitches = unique_pitches(&buffer);
    match pitches.last() {
        Some(max) => Some(*max),
        None => None,
    }
}

pub fn avg_pitch<'a>(buffer: &'a VecDeque<MidiPianoFrame>) -> u8 {
    let pitches = unique_pitches(&buffer);
    let mut sum = 0;
    for p in &pitches {
        sum += p;
    }
    sum / (pitches.len() as u8)
}

fn analyse(midi_buffer: &VecDeque<MidiPianoFrame>) {
    
    let pitches = unique_pitches(&midi_buffer);
    println!("unique_pitches = {:?}", &pitches); 
    println!("unique_pitches len = {}", pitches.len());  
    if let Some(min) = min_pitch(&midi_buffer) {
        println!("min_pitch = {:?}", min);
    }
    if let Some(max) = max_pitch(&midi_buffer) {
        println!("max_pitch = {:?}", max);
    }
    
    //println!("avg_pitch {}", avg_pitch(&midi_buffer));

    //println!("Num Chords {}", num_chords(&midi_buffer));
}

pub fn default_midi_buffer(midi_buffer_frame_len: usize) -> VecDeque<MidiPianoFrame> {
    (0..midi_buffer_frame_len).map(|_| {
        MidiPianoFrame {
            notes: Vec::new(),
            sustain_pedal: 0.0,
            soft_pedal: 0.0,
        }
    }).collect()
}