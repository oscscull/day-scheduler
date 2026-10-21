//! Where the sync server listens and who may connect: this machine only, or also the local
//! network. Never the open internet; a token is still needed either way.

use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Listen {
    pub port: u16,
    /// Also accept other machines on the home or office network.
    pub lan: bool,
}

impl Listen {
    /// On the network, both IPv4 and IPv6 (macOS's `::` takes both). IPv6 matters: link-local
    /// `fe80::` addresses usually stay on the home network even when a VPN claims its IPv4 range.
    pub const fn host(self) -> &'static str {
        if self.lan { "::" } else { "127.0.0.1" }
    }
}

/// Loopback, or a private, link-local or carrier-grade NAT address: somewhere on this network.
const fn local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || (a == 100 && b & 0xc0 == 64)
        }
        IpAddr::V6(v6) => {
            let first = v6.segments()[0];
            v6.is_loopback() || first & 0xfe00 == 0xfc00 || first & 0xffc0 == 0xfe80
        }
    }
}

/// Whether a request from `from` may be answered at all.
pub fn allowed(from: Option<&SocketAddr>) -> bool {
    from.is_some_and(|a| local(a.ip().to_canonical()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(ip: &str) -> SocketAddr {
        SocketAddr::new(ip.parse().unwrap(), 1)
    }

    #[test]
    fn only_this_network_may_connect() {
        for ip in [
            "127.0.0.1",
            "192.168.1.20",
            "10.0.0.5",
            "::ffff:10.32.1.53",
            "172.20.1.1",
            "100.96.0.3",
            "fe80::1",
            "fd7a::2",
            "::1",
        ] {
            assert!(allowed(Some(&at(ip))), "{ip}");
        }
        for ip in ["8.8.8.8", "104.30.160.44", "::ffff:8.8.8.8", "2606:4700::1"] {
            assert!(!allowed(Some(&at(ip))), "{ip}");
        }
        assert!(!allowed(None));
    }
}
