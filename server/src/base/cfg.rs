use crate::*;
use clap::Parser;
use std::{net::SocketAddr, time::Duration};

/// Calculates the duration of a single game tick.
fn parse_tps(s: &str) -> Result<Duration> {
    let tps = s.parse::<f32>()?;

    if tps == 0.0 {
        bail!("TPS must be greater than zero.")
    } else if tps > 1024.0 {
        bail!("TPS must be less than or equal to 256.")
    }
    Ok(remdes::util::tick_dur(tps))
}

#[derive(Parser, Debug)]
pub struct Config {
    /// Target window whose title contains the given substring.
    #[arg(short, long)]
    window: String,

    /// Local UDP IP address.
    #[arg(long, default_value_t = remdes::net::get_socket_addr(UDP_PORT))]
    lu: SocketAddr,

    /// Server ticks/sec.
    #[arg(short, long, default_value = "128", value_parser = parse_tps)]
    tps: Duration,
}

impl Config {
    pub const fn window(&self) -> &str {
        self.window.as_str()
    }

    pub const fn local_udp_addr(&self) -> SocketAddr {
        self.lu
    }

    pub const fn tps(&self) -> Duration {
        self.tps
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::parse()
    }
}
