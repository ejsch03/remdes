use crate::*;
use clap::Parser;
use std::net::{Ipv4Addr, SocketAddr};

#[derive(Parser, Clone, Copy, Debug)]
pub struct Config {
    /// Local UDP address.
    #[arg(long, default_value_t = remdes::net::get_socket_addr(
        pfrs::find_open_port(Ipv4Addr::LOCALHOST, pfrs::Protocol::Udp).expect("No available dynamic ports"),
    ))]
    lu: SocketAddr,

    /// Remote UDP IP address.
    #[arg(long, default_value_t = remdes::net::get_socket_addr(UDP_PORT))]
    ru: SocketAddr,

    /// Specify the FPS.
    #[arg(short, long, default_value_t = 120)]
    fps: u8,
}

impl Config {
    pub const fn local_udp_addr(&self) -> SocketAddr {
        self.lu
    }

    pub const fn remote_udp_addr(&self) -> SocketAddr {
        self.ru
    }

    pub const fn fps(&self) -> u8 {
        self.fps
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::parse()
    }
}
