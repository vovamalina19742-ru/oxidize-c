//! Oxidize-C Transpiled Module
//! Source: socket_client.c
//! Pattern: Cross-Platform I/O Safety RAII (OwnedFd / OwnedSocket)

#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, FromRawSocket, OwnedSocket, RawSocket};

/// Safe Cross-Platform Socket Wrapper using Rust standard library I/O safety.
#[derive(Debug)]
pub struct SafeSocket {
    #[cfg(unix)]
    inner: OwnedFd,
    #[cfg(windows)]
    inner: OwnedSocket,
}

impl SafeSocket {
    /// Creates a safe RAII socket from a raw OS descriptor.
    ///
    /// # Safety
    /// Caller must guarantee that the raw descriptor is valid and exclusively owned.
    #[cfg(unix)]
    pub unsafe fn from_raw(fd: RawFd) -> Self {
        Self {
            inner: OwnedFd::from_raw_fd(fd),
        }
    }

    /// Creates a safe RAII socket from a raw OS descriptor.
    ///
    /// # Safety
    /// Caller must guarantee that the raw descriptor is valid and exclusively owned.
    #[cfg(windows)]
    pub unsafe fn from_raw(sock: RawSocket) -> Self {
        Self {
            inner: OwnedSocket::from_raw_socket(sock),
        }
    }
}

/// Network Client replacing C NetClient with safe connection management.
#[derive(Debug)]
pub struct NetClient {
    socket: Option<SafeSocket>,
    host: String,
    port: u16,
}

impl NetClient {
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            socket: None,
            host: host.into(),
            port,
        }
    }

    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.socket.is_some()
    }

    pub fn close(&mut self) {
        // Dropping the SafeSocket automatically closes the OS descriptor with no-panic guarantee.
        self.socket = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_lifecycle() {
        let mut client = NetClient::new("127.0.0.1", 8080);
        assert_eq!(client.host(), "127.0.0.1");
        assert_eq!(client.port(), 8080);
        assert!(!client.is_connected());

        client.close();
        assert!(!client.is_connected());
    }
}
