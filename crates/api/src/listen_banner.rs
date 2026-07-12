use std::net::SocketAddr;

/// Human-readable URLs for a listening socket (maps unspecified bind to localhost).
pub fn format_listen_banner(env: &str, addr: SocketAddr) -> String {
    let host = display_host(addr);
    let port = addr.port();
    let base = format!("http://{host}:{port}");
    format!(
        "Indiebase API  env={env}\n  Local:   {base}/\n  Health:  {base}/health\n  Docs:    {base}/docs\n  OpenAPI: {base}/openapi.json"
    )
}

fn display_host(addr: SocketAddr) -> String {
    if addr.ip().is_unspecified() {
        "localhost".to_string()
    } else {
        addr.ip().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn maps_unspecified_bind_to_localhost_urls() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080);
        let banner = format_listen_banner("development", addr);
        assert!(banner.contains("env=development"));
        assert!(banner.contains("http://localhost:8080/"));
        assert!(banner.contains("http://localhost:8080/health"));
        assert!(banner.contains("http://localhost:8080/docs"));
        assert!(banner.contains("http://localhost:8080/openapi.json"));
    }

    #[test]
    fn keeps_explicit_host() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);
        let banner = format_listen_banner("production", addr);
        assert!(banner.contains("http://127.0.0.1:3000/"));
    }
}
