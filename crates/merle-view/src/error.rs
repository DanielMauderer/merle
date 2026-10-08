use std::error::Error as StdError;
use std::fmt;

use merle_photo::PhotoError;
use winit::error::{EventLoopError, OsError};

#[derive(Debug)]
pub enum Error {
    Usage,
    EventLoop(EventLoopError),
    CreateWindow(OsError),
    Gpu(GpuError),
    Photo(PhotoError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => f.write_str("usage: merle-view <image path>"),
            Self::EventLoop(_) => f.write_str("event loop failed"),
            Self::CreateWindow(_) => f.write_str("cannot create window"),
            Self::Gpu(err) => err.fmt(f),
            Self::Photo(err) => err.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Usage => None,
            Self::EventLoop(err) => Some(err),
            Self::CreateWindow(err) => Some(err),
            Self::Gpu(err) => err.source(),
            Self::Photo(err) => err.source(),
        }
    }
}

impl From<EventLoopError> for Error {
    fn from(err: EventLoopError) -> Self {
        Self::EventLoop(err)
    }
}

impl From<OsError> for Error {
    fn from(err: OsError) -> Self {
        Self::CreateWindow(err)
    }
}

impl From<GpuError> for Error {
    fn from(err: GpuError) -> Self {
        Self::Gpu(err)
    }
}

impl From<PhotoError> for Error {
    fn from(err: PhotoError) -> Self {
        Self::Photo(err)
    }
}

#[derive(Debug)]
pub enum GpuError {
    CreateSurface(wgpu::CreateSurfaceError),
    RequestAdapter(wgpu::RequestAdapterError),
    RequestDevice(wgpu::RequestDeviceError),
    SurfaceLost,
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateSurface(_) => f.write_str("cannot create surface"),
            Self::RequestAdapter(_) => f.write_str("no suitable GPU adapter"),
            Self::RequestDevice(_) => f.write_str("cannot create device"),
            Self::SurfaceLost => f.write_str("surface lost"),
        }
    }
}

impl StdError for GpuError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::CreateSurface(err) => Some(err),
            Self::RequestAdapter(err) => Some(err),
            Self::RequestDevice(err) => Some(err),
            Self::SurfaceLost => None,
        }
    }
}

impl From<wgpu::CreateSurfaceError> for GpuError {
    fn from(err: wgpu::CreateSurfaceError) -> Self {
        Self::CreateSurface(err)
    }
}

impl From<wgpu::RequestAdapterError> for GpuError {
    fn from(err: wgpu::RequestAdapterError) -> Self {
        Self::RequestAdapter(err)
    }
}

impl From<wgpu::RequestDeviceError> for GpuError {
    fn from(err: wgpu::RequestDeviceError) -> Self {
        Self::RequestDevice(err)
    }
}
