//! Lock-free event-based analytics for Python LlmRouter.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

#![cfg_attr(not(feature = "enabled"), allow(unused_variables, dead_code))]

pub mod event;
pub mod helpers;
mod stats;