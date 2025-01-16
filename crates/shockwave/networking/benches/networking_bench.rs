#![feature(test)]

extern crate test;

#[cfg(test)]
mod bench_tests {
    use test::Bencher;
    use networking::udp::UdpNetwork;
    use networking::NetworkError;
    use tokio::net::UdpSocket;
    use std::net::SocketAddr;
    use tokio::sync::OnceCell;

    static client_network: OnceCell<UdpNetwork> = OnceCell::const_new();

    pub async fn get_client_network() -> &'static UdpNetwork {
        client_network.get_or_init(|| async {
            let addr1: SocketAddr = "127.0.0.1:0".parse().unwrap();

            let network1 = UdpNetwork::new(addr1).await.unwrap();

            return network1;
        })
        .await
    }

    static server_network: OnceCell<UdpNetwork> = OnceCell::const_new();

    pub async fn get_server_network() -> &'static UdpNetwork {
        server_network.get_or_init(|| async {
            let addr2: SocketAddr = "127.0.0.1:0".parse().unwrap();

            let network2 = UdpNetwork::new(addr2).await.unwrap();

            return network2;
        })
        .await
    }
    
    #[bench]
    fn bench_network_creation(b: &mut Bencher) {
        b.iter(|| async {
            get_client_network().await;
            get_server_network().await;
        })
    }

    

    #[bench]
    fn bench_handshake(b: &mut Bencher) {
        b.iter(|| async {
            let client_addr = get_client_network().await.socket.local_addr().unwrap();
            let server_addr = get_server_network().await.socket.local_addr().unwrap();

            let server = tokio::spawn(async move {
                let result = get_server_network().await.accept_handshake().await;
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), client_addr);
            });
        
            let client = tokio::spawn(async move {
                let result = get_client_network().await.handshake(server_addr).await;
                assert!(result.is_ok());
            });

            tokio::try_join!(server, client).unwrap();
        })
    }

    #[bench]
    fn bench_send_receive(b: &mut Bencher) {
        b.iter(|| async {
            let send_addr = get_client_network().await.socket.local_addr().unwrap();
            let recv_addr = get_server_network().await.socket.local_addr().unwrap();

            let sent = get_client_network().await.send(b"hello", recv_addr).await.unwrap();

            tokio::spawn(async move {
                let mut buf = [0; 10];
                let (size, addr) = get_server_network().await.receive(&mut buf).await.unwrap();
                assert_eq!(&buf[..size], b"hello");
                assert_eq!(addr, send_addr);
            });

            assert_eq!(sent, 5);
        })
    }
}


