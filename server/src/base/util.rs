use parking_lot::Mutex;
use remdes::*;
use std::io::{Write, stdout};
use std::net::UdpSocket;
use std::sync::Arc;

pub fn init_remote(udp: &UdpSocket, region: &Arc<Mutex<Region>>, rx: &waitx::Waiter) -> Result<()> {
    let mut out = stdout();

    let mut buf = [0u8; 2 + UDP_CHUNK_SIZE];
    let mut current_region = Region::default();

    loop {
        let t = std::time::Instant::now();

        // wait for notification
        rx.wait();

        // bring the current region up-to-date
        {
            let mut g = region.lock();
            std::mem::swap(&mut current_region, &mut *g);
        }
        let header = current_region.header();
        let data = current_region.data();

        /////////////////////////////////////////////

        // send the frame to the client
        udp.send(bytemuck::bytes_of(&header))?;

        // distribute each chunk
        for (i, chunk) in data.chunks(UDP_CHUNK_SIZE).enumerate() {
            let chunk_compressed = lz4::block::compress(chunk, None, false).unwrap();
            let chunk_len = chunk_compressed.len();

            let idx_bytes = (i as u16).to_le_bytes();
            buf[..2].copy_from_slice(&idx_bytes);

            // Copy chunk into buffer
            buf[2..2 + chunk_len].copy_from_slice(chunk_compressed.as_slice());

            // Send header + chunk bytes
            udp.send(&buf[..2 + chunk_len])?;
        }

        // Print timing info
        out.write_all(
            format!(
                "({}, {}, {}, {}) len={} [{}] -> {:?}\n",
                header.x(),
                header.y(),
                header.w(),
                header.h(),
                header.l(),
                remdes::util::bytes_to_mb_str(header.l()),
                t.elapsed(),
            )
            .as_bytes(),
        )?;
        out.flush()?;
    }
}
