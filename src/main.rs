use clap::Parser;
use tracing_subscriber::util::SubscriberInitExt;

mod client;
mod daemon;
mod window;

fn main() {
    tracing_subscriber::registry().init();

    let args = Args::parse();
    match args {
        Args::Daemon => daemon::start(),
        Args::Activate { item } => client::activate(item),
    }
}

#[derive(Parser)]
enum Args {
    Daemon,
    Activate { item: client::Item },
}
