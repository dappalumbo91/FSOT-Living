//! FSOT body layers — shared by bare-metal kernel and host probe.
//!
//! - [`interoception`] — vagus analogue (self-sensing ring)
//! - [`mind_abi`] — MOTOR / MIND wire protocol
//! - [`organs`] — organ membrane / VFS stub
//! - [`soma`] — homeostasis tick integrating fabric + sensors + organs
//! - [`scale`] — goldfish rule: grow into available RAM/GPU tank
//! - [`memory`] — RAM working set vs disk LTM (sleep/wake)

#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]

extern crate alloc;

pub mod dream;
pub mod ignition;
pub mod interoception;
pub mod memory;
pub mod mind_abi;
pub mod organs;
pub mod principal;
pub mod scale;
pub mod soma;

pub use ignition::{Ignition, ServiceStage};
pub use interoception::{Channel, Interoception, Sample, SensorInputs, CHANNEL_COUNT};
pub use memory::SomaSnapshot;
pub use mind_abi::{CommandRx, MotorVerb};
pub use organs::{Membrane, OrganId, OrganNode};
pub use principal::{authorize, Capabilities, Principal};
pub use scale::{ScaleProfile, ScaleTier};
pub use soma::Soma;
