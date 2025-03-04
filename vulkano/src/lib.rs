// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

#![doc(html_logo_url = "https://raw.githubusercontent.com/vulkano-rs/vulkano/master/logo.png")]
//! Safe and rich Rust wrapper around the Vulkan API.
//!
//! # Brief summary of Vulkan
//!
//! - The [`Instance`](crate::instance::Instance) object is the API entry point. It is the
//!   first object you must create before starting to use Vulkan.
//!
//! - The [`PhysicalDevice`](crate::device::physical::PhysicalDevice) object represents an
//!   implementation of Vulkan available on the system (eg. a graphics card, a software
//!   implementation, etc.). Physical devices can be enumerated from an instance with
//!   [`PhysicalDevice::enumerate`](crate::device::physical::PhysicalDevice::enumerate).
//!
//! - Once you have chosen a physical device to use, you can create a
//!   [`Device`](crate::device::Device) object from it. The `Device` is the most important
//!   object of Vulkan, as it represents an open channel of communication with a physical device.
//!   You always need to have one before you can do interesting things with Vulkan.
//!
//! - [*Buffers*](crate::buffer) and [*images*](crate::image) can be used to store data on
//!   memory accessible by the GPU (or more generally by the Vulkan implementation). Buffers are
//!   usually used to store information about vertices, lights, etc. or arbitrary data, while
//!   images are used to store textures or multi-dimensional data.
//!
//! - In order to show something on the screen, you need a [`Swapchain`](crate::swapchain).
//!   A `Swapchain` contains special `Image`s that correspond to the content of the window or the
//!   monitor. When you *present* a swapchain, the content of one of these special images is shown
//!   on the screen.
//!
//! - In order to ask the GPU to do something, you must create a
//!   [*command buffer*](crate::command_buffer). A command buffer contains a list of commands
//!   that the GPU must perform. This can include copies between buffers and images, compute
//!   operations, or graphics operations. For the work to start, the command buffer must then be
//!   submitted to a [`Queue`](crate::device::Queue), which is obtained when you create the
//!   `Device`.
//!
//! - In order to be able to add a compute operation or a graphics operation to a command buffer,
//!   you need to have created a [`ComputePipeline` or a `GraphicsPipeline`
//!   object](crate::pipeline) that describes the operation you want. These objects are usually
//!   created during your program's initialization. `Shader`s are programs that the GPU will
//!   execute as part of a pipeline. [*Descriptor sets*](crate::descriptor_set) can be used to access
//!   the content of buffers or images from within shaders.
//!
//! - For graphical operations, [`RenderPass`es and `Framebuffer`s](crate::render_pass)
//!   describe on which images the implementation must draw upon.
//!
//! - Once you have built a *command buffer* that contains a list of commands, submitting it to the
//!   GPU will return an object that implements [the `GpuFuture` trait](crate::sync::GpuFuture).
//!   `GpuFuture`s allow you to chain multiple submissions together and are essential to performing
//!   multiple operations on multiple different GPU queues.
//!

//#![warn(missing_docs)]        // TODO: activate
#![allow(dead_code)] // TODO: remove
#![allow(unused_variables)] // TODO: remove

pub use ash::vk::Handle;
pub use half;
use std::{
    error, fmt,
    ops::Deref,
    sync::{Arc, MutexGuard},
};
pub use version::Version;

#[macro_use]
mod tests;
#[macro_use]
mod extensions;
pub mod buffer;
pub mod command_buffer;
pub mod descriptor_set;
pub mod device;
pub mod format;
mod version;
#[macro_use]
pub mod render_pass;
mod fns;
pub mod image;
pub mod instance;
pub mod memory;
pub mod pipeline;
pub mod query;
mod range_map;
pub mod range_set;
pub mod sampler;
pub mod shader;
pub mod swapchain;
pub mod sync;

/// Represents memory size and offset values on a Vulkan device.
/// Analogous to the Rust `usize` type on the host.
pub use ash::vk::DeviceSize;

/// Alternative to the `Deref` trait. Contrary to `Deref`, must always return the same object.
pub unsafe trait SafeDeref: Deref {}
unsafe impl<'a, T: ?Sized> SafeDeref for &'a T {}
unsafe impl<T: ?Sized> SafeDeref for Arc<T> {}
unsafe impl<T: ?Sized> SafeDeref for Box<T> {}

/// Gives access to the internal identifier of an object.
pub unsafe trait VulkanObject {
    /// The type of the object.
    type Object: ash::vk::Handle;

    /// Returns a reference to the object.
    fn internal_object(&self) -> Self::Object;
}

/// Gives access to the internal identifier of an object.
// TODO: remove ; crappy design
pub unsafe trait SynchronizedVulkanObject {
    /// The type of the object.
    type Object: ash::vk::Handle;

    /// Returns a reference to the object.
    fn internal_object_guard(&self) -> MutexGuard<Self::Object>;
}

/// Error type returned by most Vulkan functions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OomError {
    /// There is no memory available on the host (ie. the CPU, RAM, etc.).
    OutOfHostMemory,
    /// There is no memory available on the device (ie. video memory).
    OutOfDeviceMemory,
}

impl error::Error for OomError {}

impl fmt::Display for OomError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match *self {
                OomError::OutOfHostMemory => "no memory available on the host",
                OomError::OutOfDeviceMemory => "no memory available on the graphical device",
            }
        )
    }
}

impl From<Error> for OomError {
    #[inline]
    fn from(err: Error) -> OomError {
        match err {
            Error::OutOfHostMemory => OomError::OutOfHostMemory,
            Error::OutOfDeviceMemory => OomError::OutOfDeviceMemory,
            _ => panic!("unexpected error: {:?}", err),
        }
    }
}

/// All possible success codes returned by any Vulkan function.
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
enum Success {
    Success = ash::vk::Result::SUCCESS.as_raw(),
    NotReady = ash::vk::Result::NOT_READY.as_raw(),
    Timeout = ash::vk::Result::TIMEOUT.as_raw(),
    EventSet = ash::vk::Result::EVENT_SET.as_raw(),
    EventReset = ash::vk::Result::EVENT_RESET.as_raw(),
    Incomplete = ash::vk::Result::INCOMPLETE.as_raw(),
    Suboptimal = ash::vk::Result::SUBOPTIMAL_KHR.as_raw(),
}

/// All possible errors returned by any Vulkan function.
///
/// This type is not public. Instead all public error types should implement `From<Error>` and
/// panic for error code that aren't supposed to happen.
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
// TODO: being pub is necessary because of the weird visibility rules in rustc
pub(crate) enum Error {
    OutOfHostMemory = ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY.as_raw(),
    OutOfDeviceMemory = ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY.as_raw(),
    InitializationFailed = ash::vk::Result::ERROR_INITIALIZATION_FAILED.as_raw(),
    DeviceLost = ash::vk::Result::ERROR_DEVICE_LOST.as_raw(),
    MemoryMapFailed = ash::vk::Result::ERROR_MEMORY_MAP_FAILED.as_raw(),
    LayerNotPresent = ash::vk::Result::ERROR_LAYER_NOT_PRESENT.as_raw(),
    ExtensionNotPresent = ash::vk::Result::ERROR_EXTENSION_NOT_PRESENT.as_raw(),
    FeatureNotPresent = ash::vk::Result::ERROR_FEATURE_NOT_PRESENT.as_raw(),
    IncompatibleDriver = ash::vk::Result::ERROR_INCOMPATIBLE_DRIVER.as_raw(),
    TooManyObjects = ash::vk::Result::ERROR_TOO_MANY_OBJECTS.as_raw(),
    FormatNotSupported = ash::vk::Result::ERROR_FORMAT_NOT_SUPPORTED.as_raw(),
    SurfaceLost = ash::vk::Result::ERROR_SURFACE_LOST_KHR.as_raw(),
    NativeWindowInUse = ash::vk::Result::ERROR_NATIVE_WINDOW_IN_USE_KHR.as_raw(),
    OutOfDate = ash::vk::Result::ERROR_OUT_OF_DATE_KHR.as_raw(),
    IncompatibleDisplay = ash::vk::Result::ERROR_INCOMPATIBLE_DISPLAY_KHR.as_raw(),
    ValidationFailed = ash::vk::Result::ERROR_VALIDATION_FAILED_EXT.as_raw(),
    OutOfPoolMemory = ash::vk::Result::ERROR_OUT_OF_POOL_MEMORY_KHR.as_raw(),
    InvalidExternalHandle = ash::vk::Result::ERROR_INVALID_EXTERNAL_HANDLE.as_raw(),
    FullScreenExclusiveLost = ash::vk::Result::ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT.as_raw(),
}

/// Checks whether the result returned correctly.
fn check_errors(result: ash::vk::Result) -> Result<Success, Error> {
    match result {
        ash::vk::Result::SUCCESS => Ok(Success::Success),
        ash::vk::Result::NOT_READY => Ok(Success::NotReady),
        ash::vk::Result::TIMEOUT => Ok(Success::Timeout),
        ash::vk::Result::EVENT_SET => Ok(Success::EventSet),
        ash::vk::Result::EVENT_RESET => Ok(Success::EventReset),
        ash::vk::Result::INCOMPLETE => Ok(Success::Incomplete),
        ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(Error::OutOfHostMemory),
        ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(Error::OutOfDeviceMemory),
        ash::vk::Result::ERROR_INITIALIZATION_FAILED => Err(Error::InitializationFailed),
        ash::vk::Result::ERROR_DEVICE_LOST => Err(Error::DeviceLost),
        ash::vk::Result::ERROR_MEMORY_MAP_FAILED => Err(Error::MemoryMapFailed),
        ash::vk::Result::ERROR_LAYER_NOT_PRESENT => Err(Error::LayerNotPresent),
        ash::vk::Result::ERROR_EXTENSION_NOT_PRESENT => Err(Error::ExtensionNotPresent),
        ash::vk::Result::ERROR_FEATURE_NOT_PRESENT => Err(Error::FeatureNotPresent),
        ash::vk::Result::ERROR_INCOMPATIBLE_DRIVER => Err(Error::IncompatibleDriver),
        ash::vk::Result::ERROR_TOO_MANY_OBJECTS => Err(Error::TooManyObjects),
        ash::vk::Result::ERROR_FORMAT_NOT_SUPPORTED => Err(Error::FormatNotSupported),
        ash::vk::Result::ERROR_SURFACE_LOST_KHR => Err(Error::SurfaceLost),
        ash::vk::Result::ERROR_NATIVE_WINDOW_IN_USE_KHR => Err(Error::NativeWindowInUse),
        ash::vk::Result::SUBOPTIMAL_KHR => Ok(Success::Suboptimal),
        ash::vk::Result::ERROR_OUT_OF_DATE_KHR => Err(Error::OutOfDate),
        ash::vk::Result::ERROR_INCOMPATIBLE_DISPLAY_KHR => Err(Error::IncompatibleDisplay),
        ash::vk::Result::ERROR_VALIDATION_FAILED_EXT => Err(Error::ValidationFailed),
        ash::vk::Result::ERROR_OUT_OF_POOL_MEMORY_KHR => Err(Error::OutOfPoolMemory),
        ash::vk::Result::ERROR_INVALID_EXTERNAL_HANDLE => Err(Error::InvalidExternalHandle),
        ash::vk::Result::ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT => {
            Err(Error::FullScreenExclusiveLost)
        }
        ash::vk::Result::ERROR_INVALID_SHADER_NV => panic!(
            "Vulkan function returned \
                                               VK_ERROR_INVALID_SHADER_NV"
        ),
        c => unreachable!("Unexpected error code returned by Vulkan: {}", c),
    }
}

/// A helper type for non-exhaustive structs.
///
/// This type cannot be constructed outside Vulkano. Structures with a field of this type can only
/// be constructed by calling a constructor function or `Default::default()`. The effect is similar
/// to the standard Rust `#[non_exhaustive]` attribute, except that it does not prevent update
/// syntax from being used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] // add traits as needed
pub struct NonExhaustive(pub(crate) ());
