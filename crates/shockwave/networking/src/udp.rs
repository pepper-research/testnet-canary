use super::NetworkError;
use tokio::net::UdpSocket;
use std::net::SocketAddr;
use tokio::time::{timeout, Duration};
use tokio::test;

pub struct UdpNetwork {
    pub socket: UdpSocket,
}

impl UdpNetwork {
    pub async fn new(bind_addr: SocketAddr) -> Result<Self, NetworkError> {
        let socket = UdpSocket::bind(bind_addr).await?;
        Ok(Self { socket })
    }

    pub async fn send(&self, data: &[u8], addr: SocketAddr) -> Result<usize, NetworkError> {
        Ok(self.socket.send_to(data, addr).await?)
    }

    pub async fn receive(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), NetworkError> {
        Ok(self.socket.recv_from(buf).await?)
    }

    pub async fn handshake(&self, addr: SocketAddr) -> Result<(), NetworkError> {
        // Send SYN
        self.send(b"SYN", addr).await?;

        // Wait for SYN-ACK
        let mut buf = [0; 10];
        let timeout_duration = Duration::from_secs(5);
        let (_, recv_addr) = timeout(timeout_duration, self.receive(&mut buf))
            .await
            .map_err(|_| NetworkError::HandshakeError("Handshake timed out".into()))?
            ?;

        if recv_addr != addr {
            return Err(NetworkError::HandshakeError("Received response from unexpected address".into()));
        }

        if &buf[..7] != b"SYN-ACK" {
            return Err(NetworkError::HandshakeError("Invalid SYN-ACK received".into()));
        }

        // Send ACK
        self.send(b"ACK", addr).await?;

        Ok(())
    }

    pub async fn accept_handshake(&self) -> Result<SocketAddr, NetworkError> {
        let mut buf = [0; 10];
        let timeout_duration = Duration::from_secs(5);

        // Wait for SYN
        let (_, addr) = timeout(timeout_duration, self.receive(&mut buf))
            .await
            .map_err(|_| NetworkError::HandshakeError("Handshake timed out".into()))?
            ?;

        if &buf[..3] != b"SYN" {
            return Err(NetworkError::HandshakeError("Invalid SYN received".into()));
        }

        // Send SYN-ACK
        self.send(b"SYN-ACK", addr).await?;

        // Wait for ACK
        let (_, recv_addr) = timeout(timeout_duration, self.receive(&mut buf))
            .await
            .map_err(|_| NetworkError::HandshakeError("Handshake timed out".into()))?
            ?;

        if recv_addr != addr {
            return Err(NetworkError::HandshakeError("Received ACK from unexpected address".into()));
        }

        if &buf[..3] != b"ACK" {
            return Err(NetworkError::HandshakeError("Invalid ACK received".into()));
        }

        Ok(addr)
    }
}


#[test]
async fn test_udp_network_creation() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let network = UdpNetwork::new(addr).await;
    assert!(network.is_ok());
}

#[test]
async fn test_udp_send_receive() {
    let addr1: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let network1 = UdpNetwork::new(addr1).await.unwrap();
    let network2 = UdpNetwork::new(addr2).await.unwrap();

    let send_addr = network1.socket.local_addr().unwrap();
    let recv_addr = network2.socket.local_addr().unwrap();

    tokio::spawn(async move {
        let mut buf = [0; 10];
        let (size, addr) = network2.receive(&mut buf).await.unwrap();
        assert_eq!(&buf[..size], b"hello");
        assert_eq!(addr, send_addr);
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let sent = network1.send(b"hello", recv_addr).await.unwrap();
    assert_eq!(sent, 5);
}


#[test]
async fn test_handshake() {
    let addr1: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let network1 = UdpNetwork::new(addr1).await.unwrap();
    let network2 = UdpNetwork::new(addr2).await.unwrap();

    let client_addr = network1.socket.local_addr().unwrap();
    let server_addr = network2.socket.local_addr().unwrap();
    
    let server = tokio::spawn(async move {
        let result = network2.accept_handshake().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), client_addr);
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = tokio::spawn(async move {
        let result = network1.handshake(server_addr).await;
        assert!(result.is_ok());
    });

    tokio::try_join!(server, client).unwrap();
}

#[test]
async fn test_handshake_timeout() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let network = UdpNetwork::new(addr).await.unwrap();

    let non_existent_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

    let result = network.handshake(non_existent_addr).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NetworkError::HandshakeError(_)));
}

#[test]
async fn test_invalid_handshake_response() {
    let addr1: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let network1 = UdpNetwork::new(addr1).await.unwrap();
    let network2 = UdpNetwork::new(addr2).await.unwrap();

    let server_addr = network2.socket.local_addr().unwrap();

    tokio::spawn(async move {
        let mut buf = [0; 10];
        let (_, addr) = network2.receive(&mut buf).await.unwrap();
        // Send invalid response
        network2.send(b"INVALID", addr).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let result = network1.handshake(server_addr).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NetworkError::HandshakeError(_)));
}
