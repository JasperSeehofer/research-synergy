//! Bridge to the force layout Web Worker.

use gloo_worker::reactor::ReactorBridge;
use gloo_worker::Spawnable;
use resyn_worker::{ForceLayoutWorker, LayoutInput};

/// A typed bridge to the ForceLayoutWorker Web Worker.
///
/// Provides a `send` method to dispatch layout computation requests and
/// receives results via the ReactorBridge stream interface.
///
/// Note: The `#[reactor]` macro generates `ForceLayoutWorker::spawner()` which
/// returns a `ReactorSpawner`. Outputs are received by polling the bridge as a
/// Stream or by awaiting `bridge.next()`.
pub struct WorkerBridge {
    pub bridge: ReactorBridge<ForceLayoutWorker>,
}

impl WorkerBridge {
    /// Create a new bridge to the force layout worker.
    ///
    /// The spawn path `./resyn_worker.js` matches Trunk's default output
    /// filename for a worker build. Validated during Plan 04 integration testing.
    pub fn new() -> Self {
        let bridge = ForceLayoutWorker::spawner().spawn("./resyn_worker.js");
        Self { bridge }
    }

    /// Send a layout input to the worker for asynchronous processing.
    /// Results are received by polling `bridge` as a Stream.
    pub fn send(&self, input: LayoutInput) {
        self.bridge.send_input(input);
    }
}

impl Default for WorkerBridge {
    fn default() -> Self {
        Self::new()
    }
}
