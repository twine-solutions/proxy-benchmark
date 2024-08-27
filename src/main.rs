mod benchmark;

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use env_logger::Builder;
use futures::future::join_all;
use log::{info, LevelFilter};

#[derive(Parser)]
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

#[tokio::main]
async fn main() -> Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();
    let args = Args::parse();

    info!("Starting benchmark on {} @ {} concurrents.\n", args.proxy, args.concurrency);
    let benchmark = Arc::new(benchmark::Benchmark::new(
        args.proxy,
        args.url,
        args.timeout,
    )?);

    let semaphore = Arc::new(tokio::sync::Semaphore::new(args.concurrency as usize));
    let mut tasks = Vec::with_capacity(args.requests as usize);

    for _ in 0..args.requests {
        let benchmark = Arc::clone(&benchmark);
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
        tasks.push(tokio::spawn(async move {
            let result = benchmark.send().await;
            drop(permit);
            result
        }));
    }

    let results: Vec<_> = join_all(tasks)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .filter_map(|r| r.ok())
        .collect();

    let total_requests = results.len();
    let successful_requests = results.iter().filter(|r| r.status == 200).count();
    let total_duration: Duration = results.iter().map(|r| r.timing.total).sum();
    let avg_duration = total_duration / total_requests as u32;

    let total_tcp_connect: Duration = results.iter().map(|r| r.timing.tcp_connect).sum();
    let avg_tcp_connect = total_tcp_connect / total_requests as u32;

    let total_ttfb: Duration = results.iter().map(|r| r.timing.time_to_first_byte).sum();
    let avg_ttfb = total_ttfb / total_requests as u32;

    let total_download: Duration = results.iter().map(|r| r.timing.download).sum();
    let avg_download = total_download / total_requests as u32;

    let total_body_size: usize = results.iter().map(|r| r.body_size).sum();
    let status_codes: HashSet<_> = results.iter().map(|r| r.status).collect();

    info!("Total requests: {}/{}", total_requests, args.requests);
    info!("Successful requests: {}", successful_requests);
    info!("Average total duration: {:?}", avg_duration);
    info!("Average TCP connect time: {:?}", avg_tcp_connect);
    info!("Average time to first byte: {:?}", avg_ttfb);
    info!("Average download time: {:?}", avg_download);
    info!("Total body size: {} bytes", total_body_size);
    info!("Status codes: {:?}", status_codes);

    Ok(())
}
