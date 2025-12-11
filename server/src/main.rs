mod base;
use base::*;

use parking_lot::Mutex;
use remdes::*;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    thread::{JoinHandle, sleep, spawn},
    time::{Duration, Instant},
};
use waitx::*;

fn init_heartbeat(
    tcp: TcpListener,
    tx_tcp: Waker,
    is_running: Arc<AtomicBool>,
) -> JoinHandle<Result<()>> {
    spawn(move || {
        fn handle_stream(mut stream: &TcpStream) -> Result<()> {
            stream.write_all(&[0])?;
            stream.read_exact(&mut [0])?;
            Ok(())
        }

        for stream in tcp.incoming().filter_map(Result::ok) {
            // initial heartbeat
            if handle_stream(&stream).is_err() {
                continue;
            }
            println!("\tTCP {:?}", stream.peer_addr()?);

            tx_tcp.signal(); // notify main thread of a new conn

            loop {
                if handle_stream(&stream).is_err() {
                    is_running.store(false, Ordering::SeqCst);
                    break;
                }
                sleep(Duration::from_millis(250));
            }
        }
        Ok(())
    })
}

fn main() -> anyhow::Result<()> {
    let cfg = Config::default();

    // bind sockets
    let tcp = TcpListener::bind(cfg.local_tcp_addr())?;
    let udp = UdpSocket::bind(cfg.local_udp_addr())?;

    println!(
        "TCP listening @ {:?}\nUDP listening @ {:?}\n",
        tcp.local_addr()?,
        udp.local_addr()?
    );

    let (region, region_id, is_running): (Arc<Mutex<Region>>, Arc<AtomicU8>, Arc<AtomicBool>) =
        Default::default();

    let (tx_conn, rx_conn) = pair();
    let (tx_tcp, rx_tcp) = pair();
    let (tx_dist, rx_dist) = pair();

    let _handler = start_capturing(
        cfg,
        region.clone(),
        region_id.clone(),
        is_running.clone(),
        (tx_dist, rx_conn),
    );

    // TCP-based heartbeat thread
    let _heartbeat = init_heartbeat(tcp, tx_tcp, is_running.clone());

    loop {
        println!("Waiting for TCP connection...");
        rx_tcp.wait();

        // continuously attempt to obtain UDP message.
        // timeout after 1 second.
        let t = Instant::now();
        while t.elapsed() < SECOND {
            match udp.recv_from(&mut [0; 1]) {
                std::result::Result::Ok((_, addr)) => {
                    println!("\tUDP {:?}\n", addr);
                    tx_conn.signal();
                    _ = handle_client(&udp, addr, &region, is_running.as_ref(), &rx_dist);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                    sleep(Duration::from_millis(1));
                    continue;
                }
                Err(e) => eprintln!("Unexpected: {e:?}"),
            }
        }
    }
}
