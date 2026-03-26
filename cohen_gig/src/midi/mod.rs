pub mod learn;
pub mod mapping;

use std::collections::HashSet;
use std::sync::mpsc;
use std::time::Instant;

const PORT_SCAN_INTERVAL_SECS: f64 = 2.0;

#[derive(Debug, Clone)]
pub struct MidiMessage {
    pub port_name: String,
    pub cc: u8,
    pub value: u8,
}

pub struct MidiManager {
    connections: Vec<(String, midir::MidiInputConnection<()>)>,
    connected_port_names: HashSet<String>,
    tx: mpsc::Sender<MidiMessage>,
    rx: mpsc::Receiver<MidiMessage>,
    last_scan: Instant,
}

impl MidiManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let mut manager = Self {
            connections: Vec::new(),
            connected_port_names: HashSet::new(),
            tx,
            rx,
            last_scan: Instant::now(),
        };
        manager.scan_and_connect();
        manager
    }

    pub fn poll(&mut self) {
        if self.last_scan.elapsed().as_secs_f64() >= PORT_SCAN_INTERVAL_SECS {
            self.scan_and_connect();
        }
    }

    pub fn drain(&self) -> mpsc::TryIter<'_, MidiMessage> {
        self.rx.try_iter()
    }

    #[allow(dead_code)]
    pub fn connected_ports(&self) -> &HashSet<String> {
        &self.connected_port_names
    }

    fn scan_and_connect(&mut self) {
        self.last_scan = Instant::now();

        let Ok(midi_in) = midir::MidiInput::new("cohen_gig_scan") else {
            return;
        };

        let available: HashSet<String> = (0..midi_in.port_count())
            .filter_map(|i| midi_in.port_name(i).ok())
            .collect();

        // Disconnect removed ports.
        let removed: Vec<String> = self
            .connected_port_names
            .difference(&available)
            .cloned()
            .collect();
        for name in &removed {
            println!("[midi] disconnected: {name}");
            self.connections.retain(|(n, _)| n != name);
            self.connected_port_names.remove(name);
        }

        // Connect new ports.
        for name in &available {
            if self.connected_port_names.contains(name) {
                continue;
            }
            if let Some(conn) = connect_port(name, self.tx.clone()) {
                println!("[midi] connected: {name}");
                self.connections.push((name.clone(), conn));
                self.connected_port_names.insert(name.clone());
            }
        }
    }
}

fn connect_port(
    name: &str,
    tx: mpsc::Sender<MidiMessage>,
) -> Option<midir::MidiInputConnection<()>> {
    let midi_in = midir::MidiInput::new(name).ok()?;
    let port_index = (0..midi_in.port_count()).find(|&i| midi_in.port_name(i).ok().as_deref() == Some(name))?;
    let port_name = name.to_string();
    midi_in
        .connect(
            port_index,
            name,
            move |_stamp, msg, _| {
                if msg.len() >= 3 && (msg[0] & 0xF0) == 0xB0 {
                    let _ = tx.send(MidiMessage {
                        port_name: port_name.clone(),
                        cc: msg[1],
                        value: msg[2],
                    });
                }
            },
            (),
        )
        .ok()
}
