mod app;
mod model;
mod plain;
mod theme;
mod tui;

use std::io::IsTerminal;

use clap::Parser;

use app::App;
use model::Category;

#[derive(Parser)]
#[command(name = "codez", about = "Interactive terminal dictionary of status and error codes")]
struct Cli {
    /// Code or text to look up. With a query, codez prints and exits.
    query: Option<String>,

    /// Filter to HTTP status codes
    #[arg(long)]
    http: bool,
    /// Filter to shell exit codes
    #[arg(long)]
    exit: bool,
    /// Filter to curl exit codes
    #[arg(long)]
    curl: bool,
    /// Filter to git errors
    #[arg(long)]
    git: bool,
    /// Filter to C/POSIX/Zephyr errno (hidden from the default view)
    #[arg(long)]
    errno: bool,
    /// Filter to Bluetooth LE error codes
    #[arg(long)]
    ble: bool,
    /// Filter to Bluetooth LE Audio codes
    #[arg(long)]
    leaudio: bool,
    /// Filter to Rust compiler error codes
    #[arg(long)]
    rust: bool,
    /// Filter to Docker exit codes
    #[arg(long)]
    docker: bool,
    /// Filter to Podman exit codes
    #[arg(long)]
    podman: bool,

    /// Force plain (non-interactive) output
    #[arg(long)]
    plain: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new(model::load_all());

    // Category flags. If several are passed, the first listed wins.
    let flag_categories = [
        (cli.http, Category::Http),
        (cli.exit, Category::Exit),
        (cli.curl, Category::Curl),
        (cli.git, Category::Git),
        (cli.errno, Category::Errno),
        (cli.ble, Category::Ble),
        (cli.leaudio, Category::LeAudio),
        (cli.rust, Category::Rust),
        (cli.docker, Category::Docker),
        (cli.podman, Category::Podman),
    ];
    app.filter = flag_categories
        .iter()
        .find(|(on, _)| *on)
        .map(|(_, c)| *c);

    if let Some(q) = cli.query.as_deref() {
        app.query = q.to_string();
    }

    let interactive = std::io::stdout().is_terminal() && !cli.plain && cli.query.is_none();

    let code = if interactive {
        match tui::run(app) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("codez error: {e}");
                1
            }
        }
    } else {
        let (out, code) = plain::render(&app);
        if code == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        code
    };

    std::process::exit(code);
}
