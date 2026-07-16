use std::env;
use std::path::PathBuf;
use std::process;
use std::sync::{mpsc, Arc};

use lens::{open_browser, HttpRenderer, LensError, Workspace, WorkspaceServer};

struct Config {
    target: Option<PathBuf>,
    no_open: bool,
    port: u16,
    renderer_url: Option<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("lens: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), LensError> {
    let Some(config) = parse_args()? else {
        return Ok(());
    };
    let cwd = env::current_dir()?;
    let workspace = Workspace::from_arg(config.target.as_deref(), &cwd)?;
    let renderer_url = config.renderer_url;
    let renderer: Arc<dyn lens::PlantUmlRenderer> = match renderer_url.as_deref() {
        Some(url) => Arc::new(HttpRenderer::new(url)),
        None => Arc::new(lens::UnconfiguredRenderer),
    };
    let mut server = WorkspaceServer::start(workspace, renderer, config.port)?;
    let url = server.url();

    println!("Lens workspace available at {url}");
    println!(
        "PlantUML renderer: {}",
        renderer_url.as_deref().unwrap_or("not configured")
    );
    if !config.no_open {
        if let Err(error) = open_browser(&url) {
            eprintln!("lens: could not open the default browser: {error}");
            eprintln!("lens: open {url} manually or use --no-open");
        }
    }

    let (shutdown_sender, shutdown_receiver) = mpsc::channel();
    if let Err(error) = ctrlc::set_handler(move || {
        let _ = shutdown_sender.send(());
    }) {
        eprintln!("lens: graceful Ctrl-C handling unavailable: {error}");
        server.wait();
        return Ok(());
    }
    let _ = shutdown_receiver.recv();
    server.shutdown();
    Ok(())
}

fn parse_args() -> Result<Option<Config>, LensError> {
    let mut target = None;
    let mut no_open = false;
    let mut port = 0;
    let mut renderer_url = env::var("LENS_RENDERER_URL").ok();
    let mut args = env::args_os().skip(1);

    while let Some(argument) = args.next() {
        if argument == "--help" || argument == "-h" {
            println!(
                "Usage: lens [PATH] [OPTIONS]\n\nStarts a local browser workspace. With no PATH, the current Git repository is used.\n\nOptions:\n  --no-open                 Do not launch the default browser\n  --port PORT               Bind to PORT (default: an available port)\n  --renderer-url URL        PlantUML POST endpoint (remote rendering is opt-in)\n  -h, --help                Show this help"
            );
            return Ok(None);
        }
        if argument == "--no-open" {
            no_open = true;
            continue;
        }
        if argument == "--port" {
            let value = args
                .next()
                .ok_or_else(|| LensError::BadRequest("--port requires a value".to_string()))?;
            port = value.to_string_lossy().parse().map_err(|_| {
                LensError::BadRequest("--port must be a valid TCP port".to_string())
            })?;
            continue;
        }
        if argument == "--renderer-url" {
            renderer_url = Some(
                args.next()
                    .ok_or_else(|| {
                        LensError::BadRequest("--renderer-url requires a value".to_string())
                    })?
                    .to_string_lossy()
                    .into_owned(),
            );
            continue;
        }
        if argument.to_string_lossy().starts_with('-') {
            return Err(LensError::BadRequest(format!(
                "unknown option: {}",
                argument.to_string_lossy()
            )));
        }
        if target.replace(PathBuf::from(argument)).is_some() {
            return Err(LensError::BadRequest(
                "only one target path may be provided".to_string(),
            ));
        }
    }

    Ok(Some(Config {
        target,
        no_open,
        port,
        renderer_url,
    }))
}
