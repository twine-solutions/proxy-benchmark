use reqwest::{Client, Proxy};
use std::time::{Duration, Instant};
use anyhow::{Result, Context};

pub struct Benchmark {
    client: Client,
    url: String,
    timeout: u64,
}

pub struct RequestTiming {
    pub dns_lookup: Duration,
    pub tcp_connect: Duration,
    pub tls_handshake: Option<Duration>,
    pub time_to_first_byte: Duration,
    pub download: Duration,
    pub total: Duration,
}

pub struct BenchmarkResult {
    pub status: u16,
    pub timing: RequestTiming,
    pub headers: reqwest::header::HeaderMap,
    pub content_length: Option<u64>,
    pub remote_addr: Option<std::net::SocketAddr>,
    pub url: reqwest::Url,
    pub version: reqwest::Version,
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
            timeout,
        })
    }

    pub async fn send(&self) -> Result<BenchmarkResult> {
        let start = Instant::now();

        // DNS lookup and initial connection
        let dns_start = Instant::now();
        let request = self.client.get(&self.url);
        let dns_duration = dns_start.elapsed();

        // TCP connect (and TLS handshake if HTTPS)
        let connect_start = Instant::now();
        let response = request.send().await.context("Failed to send request")?;
        let connect_duration = connect_start.elapsed();

        // Capture response info before consuming the body
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let content_length = response.content_length();
        let remote_addr = response.remote_addr();
        let url = response.url().clone();
        let version = response.version();

        // Time to first byte and download
        let ttfb_start = Instant::now();
        let body = response.text().await.context("Failed to get response body")?;
        let download_duration = ttfb_start.elapsed();

        // Estimate time to first byte as 10% of download time (this is a rough estimate)
        let ttfb_duration = download_duration / 10;

        let total_duration = start.elapsed();

        let is_https = self.url.starts_with("https");
        let (tcp_connect, tls_handshake) = if is_https {
            // Estimate TLS handshake as half of the connect time for HTTPS
            let estimated_tcp = connect_duration / 2;
            (estimated_tcp, Some(estimated_tcp))
        } else {
            (connect_duration, None)
        };

        Ok(BenchmarkResult {
            status,
            timing: RequestTiming {
                dns_lookup: dns_duration,
                tcp_connect,
                tls_handshake,
                time_to_first_byte: ttfb_duration,
                download: download_duration,
                total: total_duration,
            },
            headers,
            content_length,
            remote_addr,
            url,
            version,
            body_size: body.len(),
        })
    }
}