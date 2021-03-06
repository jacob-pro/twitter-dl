mod download;
mod model;
mod twitter;

#[cfg(feature = "serve")]
mod serve;

use anyhow::bail;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Download tweets
    Download(DownloadArgs),
    /// Serve the downloaded tweet viewer
    Serve(ServeArgs),
}

#[derive(Parser, Debug)]
pub struct DownloadArgs {
    /// Path to the authentication details file
    #[clap(short, long, default_value = "./auth.json")]
    auth: PathBuf,
    /// Where to save downloaded media (a sub folder will be created for each username)
    #[clap(short, long, default_value = "./")]
    out: PathBuf,
    /// Username(s) to download from (comma seperated)
    #[clap(short, long)]
    users: Option<String>,
    /// File containing list of usernames to download from (one per line)
    #[clap(short, long)]
    list: Option<PathBuf>,
    /// Download photos
    #[clap(long)]
    photos: bool,
    /// Download videos
    #[clap(long)]
    videos: bool,
    /// Download gifs
    #[clap(long)]
    gifs: bool,
    /// Rescan tweets that have already been loaded
    #[clap(long)]
    rescan: bool,
    /// Continue even if an account fails to download
    #[clap(long)]
    continue_on_error: bool,
    /// Use Twitter API 2 (Warning: Does not support Video and Gif downloads)
    #[clap(long)]
    api_v2: bool,
    /// Number of downloads to do concurrently
    #[clap(long, default_value_t = 4)]
    concurrency: usize,
    #[clap(long, arg_enum, default_value_t = FileExistsPolicy::Warn)]
    file_exists_policy: FileExistsPolicy,
}

#[derive(clap::ArgEnum, Debug, Clone, Eq, PartialEq)]
pub enum FileExistsPolicy {
    /// The existing file will be overwritten with a new download
    Overwrite,
    /// The data file will be updated to include the already present file
    Adopt,
    /// A warning is printed to the console
    Warn,
}

#[derive(Parser, Debug, Clone)]
pub struct ServeArgs {
    /// Location of tweet folders to serve
    #[clap(default_value = "./")]
    dir: PathBuf,
    /// Socket to serve the server on
    #[clap(long, default_value = "127.0.0.1:7008")]
    socket: SocketAddr,
    /// Don't launch the web browser
    #[clap(long)]
    no_launch: bool,
    /// Don't use TLS/HTTP2
    #[clap(long)]
    no_tls: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args: Args = Args::parse();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    if let Err(e) = async {
        match args.command {
            Commands::Download(args) => crate::download::download(args).await?,
            Commands::Serve(args) => {
                if cfg!(feature = "serve") {
                    crate::serve::serve(args).await?
                } else {
                    bail!("Application must be built with the `serve` feature")
                }
            }
        };
        Ok::<_, anyhow::Error>(())
    }
    .await
    {
        log::error!("{:#}", e);
        std::process::exit(1);
    }
}
