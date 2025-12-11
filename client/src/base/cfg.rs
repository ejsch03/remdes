use crate::*;
use clap::Parser;
use remdes::util::get_socket_addr;
use std::net::{Ipv4Addr, SocketAddr};

#[derive(Parser, Clone, Copy, Debug)]
pub struct Config {
    /// Remote TCP address.
    #[arg(long, default_value_t = get_socket_addr(TCP_PORT))]
    rt: SocketAddr,

    /// Local UDP address.
    #[arg(long, default_value_t = get_socket_addr(
        pfrs::find_open_port(Ipv4Addr::LOCALHOST, pfrs::Protocol::Udp).expect("No available dynamic ports"),
    ))]
    lu: SocketAddr,

    /// Remote UDP address.
    #[arg(long, default_value_t = get_socket_addr(UDP_PORT))]
    ru: SocketAddr,

    /// Specify the FPS.
    #[arg(short, long, default_value_t = 120)]
    fps: u8,
}

impl Config {
    pub const fn remote_tcp_addr(&self) -> SocketAddr {
        self.rt
    }

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
