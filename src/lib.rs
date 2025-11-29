pub mod net;
pub mod util;

pub use anyhow::*;

use bytemuck::{Pod, Zeroable};
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

// default dynamic ports (arbitrary, for now)
pub const UDP_PORT: u16 = 54287;

pub const UDP_CHUNK_SIZE: usize = 36_864;

pub const SECOND: Duration = Duration::from_secs(1);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct RegionHeader {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    l: u32,
}

impl RegionHeader {
    pub const fn x(&self) -> i32 {
        self.x as i32
    }

    pub const fn y(&self) -> i32 {
        self.y as i32
    }

    pub const fn w(&self) -> i32 {
        self.w as i32
    }

    pub const fn h(&self) -> i32 {
        self.h as i32
    }

    pub const fn l(&self) -> usize {
        self.l as usize
    }

    pub const fn set_x(&mut self, x: u16) {
        self.x = x
    }

    pub const fn set_y(&mut self, y: u16) {
        self.y = y
    }

    pub const fn set_w(&mut self, w: i32) {
        self.w = w as u16
    }

    pub const fn set_h(&mut self, h: i32) {
        self.h = h as u16
    }

    pub const fn set_l(&mut self, l: usize) {
        self.l = l as u32
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Region {
    header: RegionHeader,
    data: Vec<u8>,
}

impl Region {
    pub const fn header(&self) -> RegionHeader {
        self.header
    }

    #[inline]
    pub const fn header_mut(&mut self) -> &mut RegionHeader {
        &mut self.header
    }

    #[inline]
    pub const fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    #[inline]
    pub const fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    #[inline]
    pub fn update(&mut self, header: RegionHeader) {
        self.header = header;

        // ensure buffer has correct size and capacity
        let old_len = self.data.len();
        let new_len = self.header.l();
        if self.data.capacity() < new_len {
            self.data.reserve_exact(new_len.saturating_sub(old_len));
        }
        unsafe { self.data.set_len(new_len) }
    }
}

impl Deref for Region {
    type Target = RegionHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

impl DerefMut for Region {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.header
    }
}
