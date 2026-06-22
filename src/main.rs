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

    /// Force plain (non-interactive) output
    #[arg(long)]
    plain: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new(model::load_all());

    // Category flags, by precedence (git > curl > exit > http) if several are passed.
    app.filter = if cli.git {
        Some(Category::Git)
    } else if cli.curl {
        Some(Category::Curl)
    } else if cli.exit {
        Some(Category::Exit)
    } else if cli.http {
        Some(Category::Http)
    } else {
        None
    };

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
