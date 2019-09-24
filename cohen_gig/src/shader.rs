//! Items related to hotloading the shader crate.

use hotlib::BuildError;
use nannou::prelude::*;
use shader_shared::Uniforms;
use std::sync::mpsc;

/// Describes the result of the last incoming library.
#[derive(Debug)]
pub enum LastIncoming {
    Succeeded,
    Failed(BuildError),
}

/// The current activity within the shader receiver.
#[derive(Debug)]
pub enum Activity<'a> {
    Incoming,
    LastIncoming(&'a LastIncoming),
}

/// A handle for receiving hotloading shader updates on the main thread.
pub struct ShaderReceiver {
    // For receiving notification of processing shaders.
    rx: mpsc::Receiver<Incoming>,
    // The incoming shader instance if there is one.
    incoming: Option<Incoming>,
    // The moment at which the last result was received.
    last_timestamp: std::time::Instant,
    // The result of the last incoming shader.
    last_incoming: LastIncoming,
}

/// A loaded instance of the shader crate.
pub struct Shader {
    // The last successfully loaded shader library instance.
    lib: hotlib::TempLibrary,
}

/// The function signature of the shader function.
pub type ShaderFnPtr = fn(Vector3, &Uniforms) -> LinSrgb;

struct Incoming {
    rx: mpsc::Receiver<Result<hotlib::TempLibrary, BuildError>>,
}

impl ShaderReceiver {
    /// Whether or not the shader is currently incoming. If not, whether the last incoming shader
    /// built successfully or not.
    pub fn activity(&self) -> Activity {
        if self.incoming.is_some() {
            Activity::Incoming
        } else {
            Activity::LastIncoming(&self.last_incoming)
        }
    }

    /// The last incoming shader result.
    pub fn last_incoming(&self) -> &LastIncoming {
        &self.last_incoming
    }

    /// The moment at which the last incoming shader result was received, whether success or
    /// failure.
    pub fn last_timestamp(&self) -> std::time::Instant {
        self.last_timestamp
    }

    /// Update the shader receiver to check for newly received shaders.
    pub fn update(&mut self) -> Option<Shader> {
        loop {
            // If we're already aware of an incoming lib, wait for it.
            if let Some(ref incoming) = self.incoming {
                let res = incoming.rx.try_iter().next()?;
                self.incoming = None;
                self.last_timestamp = std::time::Instant::now();
                match res {
                    Ok(lib) => {
                        self.last_incoming = LastIncoming::Succeeded;
                        return Some(Shader::from(lib));
                    }
                    Err(err) => {
                        self.last_incoming = LastIncoming::Failed(err);
                        return None;
                    }
                }
            }

            // Otherwise check for notification of the most recent incoming shader.
            self.incoming = Some(self.rx.try_iter().last()?);
        }
    }
}

impl Shader {
    /// Load the shader function.
    pub fn get_fn(&self) -> libloading::Symbol<ShaderFnPtr> {
        unsafe {
            self.lib.get("shader".as_bytes()).expect("failed to load shader fn symbol")
        }
    }
}

impl From<hotlib::TempLibrary> for Shader {
    fn from(lib: hotlib::TempLibrary) -> Self {
        Shader { lib }
    }
}

fn shader_toml_path() -> std::path::PathBuf {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = path.parent().expect("could not find workspace dir");
    workspace_dir.join("shader").join("Cargo").with_extension("toml")
}

/// Create the shader watch and run it on a separate thread.
///
/// Returns the current state of the library and a handle for receiving shader updates.
pub fn spawn_watch() -> ShaderReceiver {
    let (incoming_tx, rx) = mpsc::channel();

    // Spawn the shader watch thread.
    std::thread::spawn(move || {
        // Begin the watch.
        let shader_watch = hotlib::watch(&shader_toml_path())
            .expect("failed to start watching shader");

        fn build_and_send(tx: &mpsc::Sender<Incoming>, pkg: hotlib::Package) {
            // Notify of incoming library.
            let (result_tx, rx) = mpsc::channel();
            tx.send(Incoming { rx }).ok();

            // Attempt to build the library and send the result.
            let res = pkg.build().map(|build| {
                build.load().expect("failed to load shader library")
            });
            result_tx.send(res).ok();
        }

        // Initial build.
        let pkg = shader_watch.package();
        build_and_send(&incoming_tx, pkg);

        loop {
            let mut pkg = match shader_watch.next() {
                Err(hotlib::NextError::ChannelClosed) => break,
                Err(err) => panic!("{}", err),
                Ok(pkg) => pkg,
            };
            std::thread::sleep(std::time::Duration::from_millis(16));
            while let Ok(Some(recent_pkg)) = shader_watch.try_next() {
                pkg = recent_pkg;
            }
            build_and_send(&incoming_tx, pkg);
        }
    });

    let last_incoming = LastIncoming::Succeeded;
    let incoming = None;
    let last_timestamp = std::time::Instant::now();
    let shader_rx = ShaderReceiver { rx, incoming, last_timestamp, last_incoming };
    shader_rx
}

// A function that matches the `ShaderFnPtr` that can be used as a fallback while the dylib is
// building and loading for the first time.
pub fn black(_: Vector3, _: &Uniforms) -> LinSrgb {
    lin_srgb(0.0, 0.0, 0.0)
}
