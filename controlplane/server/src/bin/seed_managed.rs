//! Seeds the managed services catalog from YAML files.
//!
//! Reads every `*.yaml` / `*.yml` file under the seed directory (one per
//! service) and upserts `managed.service` rows. Pass `--with-dev-mock` to
//! also insert the optional `dev_mock_version` block of each file -- useful
//! for local development before the charts CI is wired up. Production
//! versions are registered exclusively via the `RegisterVersion` gRPC,
//! never by this binary.
//!
//! Usage:
//!
//! ```text
//! DATABASE_URL=postgres://... cargo run --bin seed_managed -- \
//!     --dir controlplane/seed/managed [--with-dev-mock]
//! ```

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use frn_core::managed::seed_directory;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let mut dir: Option<PathBuf> = None;
    let mut with_dev_mock = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --dir requires a path argument");
                    return ExitCode::from(2);
                }
                dir = Some(PathBuf::from(&args[i]));
            }
            "--with-dev-mock" => {
                with_dev_mock = true;
            }
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("error: unknown argument: {other}");
                print_help();
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let dir = match dir {
        Some(d) => d,
        None => {
            eprintln!("error: --dir is required");
            print_help();
            return ExitCode::from(2);
        }
    };

    let database_url = match env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("error: DATABASE_URL environment variable is not set");
            return ExitCode::from(2);
        }
    };

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: could not connect to database: {e}");
            return ExitCode::from(1);
        }
    };

    match seed_directory(&pool, &dir, with_dev_mock).await {
        Ok(reports) => {
            if reports.is_empty() {
                println!("no YAML files found under {}", dir.display());
                return ExitCode::SUCCESS;
            }
            println!("seeded {} service(s) from {}", reports.len(), dir.display());
            for report in &reports {
                let plans_info = if report.plans_upserted > 0 {
                    format!(", {} plan(s) upserted", report.plans_upserted)
                } else {
                    String::new()
                };
                let mock_info = if report.mock_version_inserted {
                    ", dev mock version inserted"
                } else {
                    ""
                };
                println!(
                    "  - {}{}{}",
                    report.service_slug, plans_info, mock_info,
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: seed failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn print_help() {
    eprintln!(
        "Usage: seed_managed --dir <path> [--with-dev-mock]\n\
         \n\
         Options:\n\
           --dir <path>       Directory containing one *.yaml file per managed service\n\
           --with-dev-mock    Also insert the optional dev_mock_version block from each file\n\
           -h, --help         Print this help and exit\n\
         \n\
         Environment:\n\
           DATABASE_URL       PostgreSQL connection string (required)"
    );
}
