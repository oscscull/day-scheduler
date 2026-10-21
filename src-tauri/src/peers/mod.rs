//! Other machines' Bridges. Each machine serves its own data and tools over HTTP (the same tools
//! as the MCP server); the UI merges what peers serve, and edits to their records go back to them.

pub mod client;
pub mod server;

use std::io::Read as _;

use crate::error::AppResult;

const HEX: &[u8; 16] = b"0123456789abcdef";

/// A random token for `[sync] token`, from the system's random source.
pub fn new_token() -> AppResult<String> {
    let mut bytes = [0_u8; 24];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let hex = bytes.iter().fold(String::with_capacity(48), |mut out, b| {
        out.extend(
            [HEX.get(usize::from(b >> 4)), HEX.get(usize::from(b & 0xf))]
                .into_iter()
                .flatten()
                .map(|&c| char::from(c)),
        );
        out
    });
    Ok(hex)
}

#[cfg(test)]
mod tests {
    #[test]
    fn tokens_are_long_hex_and_differ() {
        let (a, b) = (super::new_token().unwrap(), super::new_token().unwrap());
        assert_eq!(a.len(), 48);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }
}
