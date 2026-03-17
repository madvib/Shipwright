#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    if args.iter().any(|a| a == "--http") {
        let port = parse_port(&args).unwrap_or(3000);
        mcp::run_http_server(port).await
    } else {
        mcp::run().await
    }
}

fn parse_port(args: &[String]) -> Option<u16> {
    args.windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse().ok())
}

fn print_help() {
    println!(
        "ship-mcp — Ship MCP server

USAGE:
    ship-mcp [OPTIONS]

OPTIONS:
    --http          Serve MCP over HTTP (default: stdio)
    --port <PORT>   HTTP port (default: 3000, requires --http)
    -h, --help      Print this help"
    );
}
