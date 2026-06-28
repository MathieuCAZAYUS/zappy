mod agent;
mod client;
mod protocol;
mod state;
 
use agent::{load_config, load_genome, NeuralAgent};
use client::ZappyClient;
 
const DEFAULT_HOST: &str = "localhost";
 
fn print_usage() {
    eprintln!("USAGE: ./zappy_ai -p port -n name -h machine");
    eprintln!("  -p port     port number");
    eprintln!("  -n name     name of the team");
    eprintln!("  -h machine  hostname of the server (default: {DEFAULT_HOST})");
}
 
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
 
    if args.iter().any(|a| a == "--help") {
        print_usage();
        return Ok(());
    }
 
    let mut port: Option<u16> = None;
    let mut team: Option<String> = None;
    let mut host = String::from(DEFAULT_HOST);
 
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                i += 1;
                let raw = args.get(i).ok_or_else(|| anyhow::anyhow!("-p requires a value"))?;
                port = Some(raw.parse::<u16>().map_err(|_| anyhow::anyhow!("Invalid port: {raw}"))?);
            }
            "-n" => {
                i += 1;
                team = Some(args.get(i).ok_or_else(|| anyhow::anyhow!("-n requires a value"))?.clone());
            }
            "-h" => {
                i += 1;
                host = args.get(i).ok_or_else(|| anyhow::anyhow!("-h requires a value"))?.clone();
            }
            other => anyhow::bail!("Unknown argument: {other}"),
        }
        i += 1;
    }
 
    let port = port.ok_or_else(|| anyhow::anyhow!("-p port is required"))?;
    let team = team.ok_or_else(|| anyhow::anyhow!("-n name is required"))?;
 
    let config = load_config("arch.json")?;
    let genome = load_genome("best_genome.npy")?;
    let agent = NeuralAgent { config, genome };
 
    let mut client = ZappyClient::connect(&host, port, &team)?;
    client.run(&agent)?;
 
    Ok(())
}
