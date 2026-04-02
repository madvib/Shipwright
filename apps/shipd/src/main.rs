fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();
    let port = extract_u16_arg(&args, "--port").unwrap_or(9315);
    let host = extract_str_arg(&args, "--host").unwrap_or_else(|| "127.0.0.1".to_string());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(shipd::run_network(host, port))
}

fn extract_u16_arg(args: &[String], flag: &str) -> Option<u16> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse().ok())
}

fn extract_str_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}
