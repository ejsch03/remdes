mod base;
use base::*;

use parking_lot::Mutex;
use remdes::*;
use std::{
    net::UdpSocket,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8},
    },
};

fn main() -> anyhow::Result<()> {
    let cfg = Config::default();

    let udp = UdpSocket::bind(cfg.local_udp_addr())?;
    println!("UDP listening @ {:?}\n", udp.local_addr()?);

    let region: Arc<Mutex<Region>> = Default::default();
    let region_id: Arc<AtomicU8> = Default::default();
    let control = Arc::new(AtomicBool::new(true));

    let (tx, rx) = waitx::pair();

    let _handler = start_capturing(cfg, region.clone(), region_id.clone(), control, tx);

    loop {
        match udp.recv_from(&mut [0; 1]) {
            std::result::Result::Ok((_, addr)) => {
                udp.connect(addr)?;
                println!("{:?}", addr);

                if let Err(e) = init_remote(&udp, &region, &rx) {
                    eprintln!("\tclient: {}", e);
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
