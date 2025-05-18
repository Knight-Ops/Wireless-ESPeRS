#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
#![no_std]

mod ap_mode;
mod web_server;

pub use ap_mode::ap_configure;
