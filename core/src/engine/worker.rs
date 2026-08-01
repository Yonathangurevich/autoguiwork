//! The worker: turns saved GUI automations into something an HTTP server can
//! drive, one run at a time.
//!
//! Why a dedicated OS thread + a single-slot queue:
//!   - rustautogui takes over the REAL mouse/keyboard and is blocking; it must
//!     run on one plain thread, not inside async tasks that hop threads.
//!   - There is only one mouse/keyboard/screen, so two automations can NEVER
//!     run at once — they'd fight over the physical input. mpsc::channel(1)
//!     enforces "one at a time"; extra requests queue in arrival order.
//!
//! Shape:
//!   async HTTP handler → (mpsc) → THIS worker thread → runs → (oneshot) → handler
//!   many requests in                one at a time                reply to the
//!                                                                 exact caller

use std::collections::HashMap;

use tokio::sync::{mpsc, oneshot};

use crate::engine::context::Context;
use crate::engine::report::RunReport;
use crate::engine::storage::load_app_by_endpoint;
use crate::models::engine_error::EngineErrorKind;

/// One unit of work handed to the worker: which automation, its args, and a
/// one-shot channel to send the result back to the specific caller waiting.
pub struct WorkerJob {
    pub endpoint_id: String,
    pub args: HashMap<String, String>,
    pub reply: oneshot::Sender<Result<RunReport, EngineErrorKind>>,
}

/// A handle the HTTP layer keeps to submit jobs to the worker.
#[derive(Clone)]
pub struct WorkerHandle {
    tx: mpsc::Sender<WorkerJob>,
}

impl WorkerHandle {
    /// Submit a job and await its result. Called from an async HTTP handler.
    /// The job runs on the worker thread whenever it reaches the front of the
    /// single-slot queue; this await resolves when that run finishes.
    pub async fn run(
        &self,
        endpoint_id: String,
        args: HashMap<String, String>,
    ) -> Result<RunReport, EngineErrorKind> {
        let (reply, wait) = oneshot::channel();
        let job = WorkerJob {
            endpoint_id,
            args,
            reply,
        };
        // If send fails, the worker thread is gone.
        self.tx
            .send(job)
            .await
            .map_err(|_| EngineErrorKind::WorkerUnavailable)?;
        // If recv fails, the worker dropped the reply without sending.
        wait.await.map_err(|_| EngineErrorKind::WorkerUnavailable)?
    }
}

/// Start the worker. Spawns one dedicated OS thread that owns the GUI and
/// processes jobs serially, and returns a WorkerHandle for submitting jobs.
///
/// The channel has capacity 1: at most one job waits while another runs;
/// further submissions block (async) until a slot frees — natural backpressure.
pub fn start_worker() -> WorkerHandle {
    let (tx, mut rx) = mpsc::channel::<WorkerJob>(1);

    std::thread::spawn(move || {
        // The GUI handle lives on THIS thread for the worker's whole life —
        // created once, reused for every run.
        let mut gui = match rustautogui::RustAutoGui::new(false) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("worker: failed to init GUI: {e}");
                return;
            }
        };

        // blocking_recv: this is a plain thread, so we block on the async
        // channel instead of awaiting. Loop until all senders are dropped.
        while let Some(job) = rx.blocking_recv() {
            let result = run_one(&mut gui, &job);
            // Ignore send errors: if the caller gave up (dropped the receiver),
            // there's simply no one to report to — the run still happened.
            let _ = job.reply.send(result);
        }
    });

    WorkerHandle { tx }
}

/// Execute a single job: resolve the endpoint to an automation, validate +
/// seed its args, run it, and return the report.
fn run_one(
    gui: &mut rustautogui::RustAutoGui,
    job: &WorkerJob,
) -> Result<RunReport, EngineErrorKind> {
    let app = load_app_by_endpoint(&job.endpoint_id)?
        .ok_or_else(|| EngineErrorKind::EndpointNotFound {
            endpoint_id: job.endpoint_id.clone(),
        })?;

    let mut ctx = Context::new();
    // Validate + seed the request's args against the automation's declared
    // inputs BEFORE running — a missing required input fails here, no half-run.
    app.seed_inputs(&mut ctx, &job.args)?;

    let report = app.execute(gui, &mut ctx);

    // Persist the run so the UI can show endpoint history + errors. Best-effort:
    // a logging failure must never fail the actual run.
    let _ = crate::engine::storage::save_run(&app.name, &report);

    Ok(report)
}
