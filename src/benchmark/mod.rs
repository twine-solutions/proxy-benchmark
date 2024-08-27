use reqwest::{Client, Proxy};
use std::time::{Duration, Instant};
use anyhow::{Result, Context};

pub struct Benchmark {
    client: Client,
    url: String,
}

pub struct RequestTiming {
    pub tcp_connect: Duration,
    pub time_to_first_byte: Duration,
    pub download: Duration,
    pub total: Duration,
}

pub struct BenchmarkResult {
    pub status: u16,
    pub timing: RequestTiming,
    pub body_size: usize,
}

impl Benchmark {
    pub fn new(proxy: String, url: String, timeout: u64) -> Result<Self> {
        let proxy = Proxy::all(proxy)
            .context("Failed to create proxy")?;

        let client = Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(timeout))
            .build()
            .context("Failed to build client")?;

        Ok(Self {
            client,
            url,
        })
    }

    pub async fn send(&self) -> Result<BenchmarkResult> {
        let start = Instant::now();
        let request = self.client.get(&self.url);

        let connect_start = Instant::now();
        let response = request.send().await.context("Failed to send request")?;
        let connect_duration = connect_start.elapsed();

        let status = response.status().as_u16();

        let ttfb_start = Instant::now();
        let body = response.text().await.context("Failed to get response body")?;
        let download_duration = ttfb_start.elapsed();

        let ttfb_duration = download_duration / 10;
        let total_duration = start.elapsed();

        let is_https = self.url.starts_with("https");
        let tcp_connect = if is_https {
            let estimated_tcp = connect_duration / 2;
            estimated_tcp
        } else {
            connect_duration
        };

        Ok(BenchmarkResult {
            status,
            timing: RequestTiming {
                tcp_connect,
                time_to_first_byte: ttfb_duration,
                download: download_duration,
                total: total_duration,
            },
            body_size: body.len(),
        })
    }
}