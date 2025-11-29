use crate::*;
use std::{net::UdpSocket, time::Instant};

fn handle_header(buf: &[u8], region: &mut Region) {
    // deserialize and reset header
    let header_bytes = &buf[..size_of::<RegionHeader>()];
    let header = *bytemuck::from_bytes::<RegionHeader>(header_bytes);
    region.update(header);
}

fn init_frame_handler(
    tx_event: EventSender,
    [rx_frame, rx_render]: [Waiter; 2],
    [frame_og, frame_aux]: [Arc<Mutex<Region>>; 2],
    fps_upt: Arc<FpsUpdater>,
    limit: Arc<Limit>,
) -> JoinHandle<Result<()>> {
    spawn(move || {
        rx_frame.update_thread();
        rx_render.update_thread();

        let spin = SpinSleeper::default();
        let mut t = Instant::now();

        loop {
            // wait for net to wake
            rx_frame.wait();

            // obtain limit before waiting
            let required_delay = limit.get();

            // adhere to fps limit
            let rem = required_delay.saturating_sub(t.elapsed());
            if !rem.is_zero() {
                spin.sleep(rem);
            }
            t = Instant::now();

            // bring the main frame buffer up-to-date
            {
                let mut g = frame_og.lock();
                let mut h = frame_aux.lock();
                std::mem::swap(&mut *g, &mut *h);
            }

            // notify event handler of frame update
            tx_event.push_custom_event(UserEvent::Render).unwrap();

            // wait for the texture to update and render
            rx_render.wait();

            // increment the frame counter
            fps_upt.incr();
        }
    })
}

pub fn init_remote(
    cfg: Config,
    tx_event: EventSender,
    frame_og: Arc<Mutex<Region>>,
    rx_render: Waiter,
    fps_upt: Arc<FpsUpdater>,
    limit: Arc<Limit>,
) -> JoinHandle<Result<()>> {
    spawn(move || {
        let udp = UdpSocket::bind(cfg.local_udp_addr()).unwrap();
        udp.connect(cfg.remote_udp_addr())?;
        udp.send(&[0])?;

        // auxillary frame buffer
        let frame_aux: Arc<Mutex<Region>> = Default::default();

        // udp receiving
        let mut buf = vec![0; 2 + UDP_CHUNK_SIZE];

        // local header-info and frame buffer data
        let mut region = Region::default();

        // attempt to obtain and initialize an initial region
        loop {
            let n = udp.recv(&mut buf)?;
            if n != size_of::<RegionHeader>() {
                continue;
            }
            handle_header(&buf, &mut region);
            break;
        }

        // net & frame-handler communicator
        let (tx_frame, rx_frame) = pair();

        // event-handler region communicator
        let _frame_handler = init_frame_handler(
            tx_event,
            [rx_frame, rx_render],
            [frame_og, frame_aux.clone()],
            fps_upt,
            limit,
        );

        // let mut bandwidth = 0;

        loop {
            // let start = Instant::now();
            let n = udp.recv(&mut buf)?;
            // bandwidth += n;

            if n == size_of::<RegionHeader>() {
                // bring shared region up-to-date
                {
                    let mut g = frame_aux.lock();
                    std::mem::swap(&mut region, &mut g);
                }

                // wake the render handler
                tx_frame.wake();

                // reset local region
                handle_header(&buf, &mut region);

                continue;
            }

            // TODO - expand & document
            let idx = u16::from_le_bytes([buf[0], buf[1]]);
            let payload_compressed = &buf[2..n];
            let payload =
                lz4::block::decompress(payload_compressed, Some(UDP_CHUNK_SIZE as i32)).unwrap();
            let chunk_len = payload.len();
            let start = UDP_CHUNK_SIZE * idx as usize;
            let end = start + chunk_len;
            region.data_mut()[start..end].copy_from_slice(payload.as_slice());
        }
    })
}

// let mut out = std::io::stdout();
// let mut bandwidth = 0;

// out.write_all(
//     format!(
//         "({}, {}, {}, {}) len={} bandwidth={} -> {:?}\n",
//         cnt_header.x(),
//         cnt_header.y(),
//         cnt_header.w(),
//         cnt_header.h(),
//         cnt_header.l(),
//         remdes::util::bytes_to_mb_str(bandwidth),
//         start.elapsed()
//     )
//     .as_bytes(),
// )
// .unwrap();
// out.flush().unwrap();
// t = Instant::now();
// bandwidth = 0;

// unsafe {
//     std::ptr::copy_nonoverlapping(
//         payload.as_ptr(),
//         cnt_frame_buf.as_mut_ptr().add(start),
//         chunk_len,
//     );
// }
