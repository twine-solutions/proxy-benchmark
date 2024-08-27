mod benchmark;

use clap::Parser;
use anyhow::Result;

#[derive(Parser, Debug)]
#[clap(author = "dropout", version = "1.0.0", about = "Benchmark proxies.", long_about = None)]
struct Args {
    /// URL to send requests to.
    #[clap(short, long, default_value = "https://wtfismyip.com/text")]
    url: String,

    /// Number of requests to send.
    #[clap(short, long, default_value_t = 1_000)]
    requests: u32,

    /// Number of concurrent requests to send.
    #[clap(short, long, default_value_t = 100)]
    concurrency: u32,

    /// Timeout for each request.
    #[clap(short, long, default_value_t = 5)]
    timeout: u64,

    /// Proxy to use for requests.
    /// Supported formats: http://proxy-server:8080, https://proxy-server:8080, socks4://proxy-server:1080, socks5://proxy-server:1080
    #[clap(short, long)]
    proxy: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("{:?}", args);
    Ok(())
}