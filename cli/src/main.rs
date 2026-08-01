use autoguiwork::{
    engine::context::Context,
    engine::report::{ActionStatus, RunOutcome},
    engine::server::serve,
    engine::storage::{list_apps, load_app},
    models::engine_error::EngineErrorKind,
    tools::cancel_watcher::spawn_escape_watcher,
};

fn main() {
    // Simple CLI dispatch: `autoWork run <name>` or `autoWork list`.
    // This is the command-line stand-in for what the Tauri UI will call later.
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str());

    let result = match command {
        Some("run") => match args.get(2) {
            Some(name) => run_app(name),
            None => {
                eprintln!("usage: autoWork run <name>");
                std::process::exit(2);
            }
        },
        Some("list") => list_command(),
        Some("serve") => {
            // Default port 8787; override with `autoWork serve <port>`.
            let port = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8787);
            serve_command(port)
        }
        _ => {
            eprintln!("commands:");
            eprintln!("  autoWork run <name>    run a saved automation");
            eprintln!("  autoWork list          list saved automations");
            eprintln!("  autoWork serve [port]  run as a localhost worker server");
            std::process::exit(2);
        }
    };

    // One place to turn an EngineError into a clear, non-panicking message.
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

/// Load a saved automation by name and execute it, with Esc-to-cancel armed.
fn run_app(name: &str) -> Result<(), EngineErrorKind> {
    let app = load_app(name)?;
    println!("running '{}' ({} actions)...", app.name, app.actions.len());
    println!("(press Esc to stop)");

    let mut gui = rustautogui::RustAutoGui::new(false).map_err(EngineErrorKind::AutoGui)?;

    // Build the Context here so we can grab its cancel handle and start the
    // Esc watcher BEFORE the run begins — the watcher and the runner share the
    // same flag.
    let mut ctx = Context::new();
    spawn_escape_watcher(ctx.cancel_handle());

    // execute() now returns a full RunReport — it never "errors", a failed run
    // is just a report with a Failed outcome. Print the log line by line.
    let report = app.execute(&mut gui, &mut ctx);
    for r in &report.results {
        let status = match &r.status {
            ActionStatus::Ok => "ok".to_string(),
            ActionStatus::Failed(msg) => format!("FAIL  {msg}"),
            ActionStatus::Skipped => "skipped".to_string(),
        };
        println!("  [{}] {:<28} {:>6}ms  {}", r.index, r.name, r.duration_ms, status);
    }

    match report.outcome {
        RunOutcome::Completed => {
            println!("done.");
            Ok(())
        }
        RunOutcome::Failed { at_index } => {
            eprintln!("run failed at action [{at_index}].");
            std::process::exit(1);
        }
        RunOutcome::Cancelled { at_index } => {
            eprintln!("run cancelled at action [{at_index}].");
            std::process::exit(1);
        }
    }
}

/// Print every saved automation (the CLI version of the future UI's card list).
fn list_command() -> Result<(), EngineErrorKind> {
    let names = list_apps()?;
    if names.is_empty() {
        println!("no automations saved yet.");
    } else {
        println!("saved automations:");
        for name in names {
            println!("  {name}");
        }
    }
    Ok(())
}

/// Run as a localhost worker server. Builds a tokio runtime just for this
/// command (the other commands stay sync), then serves until killed.
fn serve_command(port: u16) -> Result<(), EngineErrorKind> {
    let runtime = tokio::runtime::Runtime::new().map_err(|source| {
        EngineErrorKind::ServerBind {
            addr: format!("127.0.0.1:{port}"),
            source,
        }
    })?;
    runtime.block_on(serve(port))
}