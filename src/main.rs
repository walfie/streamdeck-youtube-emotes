mod profile;

use anyhow::Result;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");
    Ok(())
}

#[derive(StructOpt)]
struct Args {}
