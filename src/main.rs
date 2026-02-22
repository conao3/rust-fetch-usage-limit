use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "fetch-usage-limit")]
#[command(about = "Usage limit utilities", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Claude 関連の処理
    Claude,
}

fn main() {
    let _cli = Cli::parse();
}
