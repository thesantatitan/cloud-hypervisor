// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//
// Copyright © 2020, Microsoft Corporation
//
// Copyright 2018-2019 CrowdStrike, Inc.
//
//

//! A generic abstraction around hypervisor functionality
//!
//! This crate offers a trait abstraction for underlying hypervisors
//!
//! # Platform support
//!
//! - x86_64
//! - arm64
//!

#![allow(clippy::significant_drop_in_scrutinee)]

#[macro_use]
extern crate anyhow;
#[cfg(target_arch = "x86_64")]
#[macro_use]
extern crate log;

pub mod generic_x86_64;
/// Architecture specific definitions
#[macro_use]
pub mod arch;

#[cfg(feature = "kvm")]
/// KVM implementation module
pub mod kvm;

/// Microsoft Hypervisor implementation module
#[cfg(all(feature = "mshv", target_arch = "x86_64"))]
pub mod mshv;

/// Hypevisor related module
mod hypervisor;

/// Vm related module
mod vm;

/// CPU related module
mod cpu;

/// Device related module
mod device;

pub use cpu::{HypervisorCpuError, Vcpu, VmExit};
pub use device::{Device, DeviceAttr, HypervisorDeviceError};
pub use hypervisor::{user_memory_region_flags, Hypervisor, HypervisorError, UserMemoryRegion, IoEventAddress};
#[cfg(all(feature = "kvm", target_arch = "x86_64"))]
pub use kvm::x86_64;
#[cfg(all(feature = "kvm", target_arch = "aarch64"))]
pub use kvm::{aarch64, GicState, CpuState, DeviceAttr};
#[cfg(target_arch = "x86_64")]
pub use generic_x86_64::CpuState;
pub use crate::vm::{VmState, CreateDevice, IrqRoutingEntry};
// Aliased types exposed from both hypervisors
#[cfg(feature = "kvm")]
pub use kvm::{
    ClockData, DeviceFd,
};
#[cfg(all(feature = "mshv", target_arch = "x86_64"))]
pub use mshv::x86_64;
// Aliased types exposed from both hypervisors
#[cfg(all(feature = "mshv", target_arch = "x86_64"))]
pub use mshv::{
    DeviceFd,
};
use std::sync::Arc;
pub use vm::{
    DataMatch, HypervisorVmError, InterruptSourceConfig, LegacyIrqSourceConfig, MsiIrqSourceConfig,
    Vm, VmOps,
};

pub fn new() -> std::result::Result<Arc<dyn Hypervisor>, HypervisorError> {
    #[cfg(feature = "kvm")]
    let hv = kvm::KvmHypervisor::new()?;

    #[cfg(feature = "mshv")]
    let hv = mshv::MshvHypervisor::new()?;

    Ok(Arc::new(hv))
}

// Returns a `Vec<T>` with a size in bytes at least as large as `size_in_bytes`.
fn vec_with_size_in_bytes<T: Default>(size_in_bytes: usize) -> Vec<T> {
    let rounded_size = (size_in_bytes + size_of::<T>() - 1) / size_of::<T>();
    let mut v = Vec::with_capacity(rounded_size);
    v.resize_with(rounded_size, T::default);
    v
}

// The kvm API has many structs that resemble the following `Foo` structure:
//
// ```
// #[repr(C)]
// struct Foo {
//    some_data: u32
//    entries: __IncompleteArrayField<__u32>,
// }
// ```
//
// In order to allocate such a structure, `size_of::<Foo>()` would be too small because it would not
// include any space for `entries`. To make the allocation large enough while still being aligned
// for `Foo`, a `Vec<Foo>` is created. Only the first element of `Vec<Foo>` would actually be used
// as a `Foo`. The remaining memory in the `Vec<Foo>` is for `entries`, which must be contiguous
// with `Foo`. This function is used to make the `Vec<Foo>` with enough space for `count` entries.
use std::mem::size_of;
pub fn vec_with_array_field<T: Default, F>(count: usize) -> Vec<T> {
    let element_space = count * size_of::<F>();
    let vec_size_bytes = size_of::<T>() + element_space;
    vec_with_size_in_bytes(vec_size_bytes)
}
