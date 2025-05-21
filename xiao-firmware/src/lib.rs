#![feature(type_alias_impl_trait)]
#![feature(future_join)]
#![feature(impl_trait_in_assoc_type)]
#![no_std]

mod ap_mode;
mod web_server;

pub use crate::ap_mode::{STATUS_WATCHER, WifiStatus, networking_task};

// REMOVE ONCE WE HAVE STORAGE IMPLEMENTED
#[unsafe(link_section = ".kvs_section")]
#[used]
static _KVSSPACE: [u8; 0] = [];
