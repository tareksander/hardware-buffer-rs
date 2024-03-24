use std::{ffi::CStr, mem::MaybeUninit, os::{fd::{AsRawFd, BorrowedFd}, unix::net::UnixStream}, ptr::{null, null_mut, NonNull}};

use cstr::cstr;
use lazy_static::lazy_static;
use libc::{c_char, c_int, c_void, dlopen, dlsym, RTLD_LAZY, RTLD_LOCAL};


mod hb_raw {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    
    include!(concat!(env!("OUT_DIR"), "/hardware_buffer.rs"));
}

pub use hb_raw::*;


/// Singleton that holds the loaded libandroid functions and provides wrappers.
#[allow(non_snake_case, dead_code)]
pub struct HBHolder {
    /// The device API level. Some functions are only introduced in specific API levels.
    /// The wrappers will always check before calling and generate an error if the function isn't available.
    pub api_level: c_int,
    AHardwareBuffer_acquire: *const fn(buffer: NonNull<AHardwareBuffer>) -> c_void,
    AHardwareBuffer_allocate: *const fn(desc: NonNull<AHardwareBufferDesc>, outBuffer: NonNull<*mut AHardwareBuffer>) -> c_int,
    AHardwareBuffer_describe: *const fn(buffer: NonNull<AHardwareBuffer>, outDesc: NonNull<AHardwareBufferDesc>) -> c_void,
    AHardwareBuffer_getId: *const fn(buffer: NonNull<AHardwareBuffer>, outId: NonNull<u64>) -> c_int,
    AHardwareBuffer_isSupported: *const fn(desc: NonNull<AHardwareBufferDesc>) -> c_int,
    AHardwareBuffer_recvHandleFromUnixSocket: *const fn(fd: c_int, outBuffer: NonNull<*mut AHardwareBuffer>) -> c_int,
    AHardwareBuffer_lock: *const fn(buffer: NonNull<AHardwareBuffer>, usage: u64, fence: i32, rect: *const ARect, outAddr: NonNull<*mut c_void>) -> c_int,
    AHardwareBuffer_lockAndGetInfo: *const fn(buffer: NonNull<AHardwareBuffer>, usage: u64, fence: i32, rect: *const ARect, outAddr: NonNull<*mut c_void>, bpp: NonNull<i32>, bps: NonNull<i32>) -> c_int,
    AHardwareBuffer_lockPlanes: *const fn(buffer: NonNull<AHardwareBuffer>, usage: u64, fence: i32, rect: *const ARect, planes: NonNull<AHardwareBufferPlanes>) -> c_int,
    AHardwareBuffer_release: *const fn(buffer: NonNull<AHardwareBuffer>) -> c_void,
    AHardwareBuffer_sendHandleToUnixSocket: *const fn(buffer: NonNull<AHardwareBuffer>, fd: c_int) -> c_int,
    AHardwareBuffer_unlock: *const fn(buffer: NonNull<AHardwareBuffer>, fence: *mut i32) -> c_int,
}


/// Only written on initialization, so should still be OK.
/// Also the pointers are valid for all threads in the process.
unsafe impl Sync for HBHolder {}

lazy_static! {
    /// Singleton that holds the loaded libandroid functions and provides wrappers.
    pub static ref HB: HBHolder = HBHolder::new();
}


impl HBHolder {
    fn new() -> Self {
        let lib = unsafe { dlopen(cstr!("libandroid.so").as_ptr(), RTLD_LAZY | RTLD_LOCAL) };
        if lib == null_mut() {
            return Self {
                api_level: -1,
                AHardwareBuffer_release: null(),
                AHardwareBuffer_recvHandleFromUnixSocket: null(),
                AHardwareBuffer_acquire: null(),
                AHardwareBuffer_allocate: null(),
                AHardwareBuffer_describe: null(),
                AHardwareBuffer_getId: null(),
                AHardwareBuffer_isSupported: null(),
                AHardwareBuffer_lock: null(),
                AHardwareBuffer_lockAndGetInfo: null(),
                AHardwareBuffer_lockPlanes: null(),
                AHardwareBuffer_sendHandleToUnixSocket: null(),
                AHardwareBuffer_unlock: null(),
            };
        }
        
        let mut api_level = -1;
        {
            // copy the functionality of the inline version of android_get_device_api_level, because somehow it only became a function after a specific API level,
            // which means you'd have to check for the API level to check for the API level.
            let __system_property_get = unsafe { dlsym(lib, cstr!("__system_property_get").as_ptr()) } as *const fn(name: *const c_char, value: *mut c_char) -> c_int;
            if __system_property_get != null() {
                // buffer size taken from the NDK source code
                let mut buffer: [c_char; 92] = [0; 92];
                if (unsafe { *__system_property_get })(cstr!("ro.build.version.sdk").as_ptr(), buffer.as_mut_ptr()) >= 1 {
                    if let Ok(s) = CStr::from_bytes_until_nul(bytemuck::cast_slice(&buffer[..])) {
                        if let Ok(l) = s.to_str() {
                            if let Ok(l) = l.parse() {
                                api_level = l;
                            }
                        }
                    }
                }
            }
        }
        
        #[allow(non_snake_case)]
        {
            let AHardwareBuffer_acquire = unsafe { dlsym(lib, cstr!("AHardwareBuffer_acquire").as_ptr()) } as *const fn(buffer: NonNull<AHardwareBuffer>) -> c_void;
            let AHardwareBuffer_allocate = unsafe { dlsym(lib, cstr!("AHardwareBuffer_allocate").as_ptr()) } as *const fn(desc: NonNull<AHardwareBufferDesc>, outBuffer: NonNull<*mut AHardwareBuffer>) -> c_int;
            let AHardwareBuffer_describe = unsafe { dlsym(lib, cstr!("AHardwareBuffer_describe").as_ptr()) } as *const fn(buffer: NonNull<AHardwareBuffer>, outDesc: NonNull<AHardwareBufferDesc>) -> c_void;
            let AHardwareBuffer_getId = unsafe { dlsym(lib, cstr!("AHardwareBuffer_getId").as_ptr()) } as *const fn(buffer: NonNull<AHardwareBuffer>, outId: NonNull<u64>) -> c_int;
            let AHardwareBuffer_isSupported = unsafe { dlsym(lib, cstr!("AHardwareBuffer_isSupported").as_ptr()) } as *const fn(desc: NonNull<AHardwareBufferDesc>) -> c_int;
            let AHardwareBuffer_recvHandleFromUnixSocket = unsafe { dlsym(lib, cstr!("AHardwareBuffer_recvHandleFromUnixSocket").as_ptr()) } as *const fn(fd: c_int, outBuffer: NonNull<*mut AHardwareBuffer>) -> c_int;
            let AHardwareBuffer_lock = unsafe { dlsym(lib, cstr!("AHardwareBuffer_lock").as_ptr()) } as *const fn(buffer: NonNull<AHardwareBuffer>, usage: u64, fence: i32, rect: *const ARect, outAddr: NonNull<*mut c_void>) -> c_int;
            let AHardwareBuffer_lockAndGetInfo = unsafe { dlsym(lib, cstr!("AHardwareBuffer_lockAndGetInfo").as_ptr()) } as *const fn(buffer: NonNull<AHardwareBuffer>, usage: u64, fence: i32, rect: *const ARect, outAddr: NonNull<*mut c_void>, bpp: NonNull<i32>, bps: NonNull<i32>) -> c_int;
            let AHardwareBuffer_lockPlanes = unsafe { dlsym(lib, cstr!("AHardwareBuffer_lockPlanes").as_ptr()) } as *const fn(buffer: NonNull<AHardwareBuffer>, usage: u64, fence: i32, rect: *const ARect, planes: NonNull<AHardwareBufferPlanes>) -> c_int;
            let AHardwareBuffer_release = unsafe { dlsym(lib, cstr!("AHardwareBuffer_release").as_ptr()) } as *const fn(buffer: NonNull<AHardwareBuffer>) -> c_void;
            let AHardwareBuffer_sendHandleToUnixSocket = unsafe { dlsym(lib, cstr!("AHardwareBuffer_sendHandleToUnixSocket").as_ptr()) } as *const fn(buffer: NonNull<AHardwareBuffer>, fd: c_int) -> c_int;
            let AHardwareBuffer_unlock = unsafe { dlsym(lib, cstr!("AHardwareBuffer_unlock").as_ptr()) } as *const fn(buffer: NonNull<AHardwareBuffer>, fence: *mut i32) -> c_int;
            Self {
                api_level,
                AHardwareBuffer_acquire,
                AHardwareBuffer_release,
                AHardwareBuffer_allocate,
                AHardwareBuffer_describe,
                AHardwareBuffer_getId,
                AHardwareBuffer_isSupported,
                AHardwareBuffer_recvHandleFromUnixSocket,
                AHardwareBuffer_sendHandleToUnixSocket,
                AHardwareBuffer_lock,
                AHardwareBuffer_lockAndGetInfo,
                AHardwareBuffer_lockPlanes,
                AHardwareBuffer_unlock,
            }
        }
    }
    
    

    /// Acquire a reference on the given AHardwareBuffer object.  
    /// This prevents the object from being deleted until the last reference is removed.  
    /// Available since API level 26.
    pub unsafe fn acquire(&self, buffer: NonNull<AHardwareBuffer>) {
        if self.api_level < 26 {
            return;
        }
        (*self.AHardwareBuffer_acquire)(buffer);
    }
    
    

    /// Remove a reference that was previously acquired with AHardwareBuffer_acquire() or AHardwareBuffer_allocate().  
    /// Available since API level 26.
    pub unsafe fn release(&self, buffer: NonNull<AHardwareBuffer>) {
        if self.api_level < 26 {
            return;
        }
        (*self.AHardwareBuffer_release)(buffer);
    }
    
    /// Allocates a buffer that matches the passed AHardwareBuffer_Desc.  
    /// If allocation succeeds, the buffer can be used according to the usage flags specified in its description.
    /// If a buffer is used in ways not compatible with its usage flags, the results are undefined and may include program termination.  
    /// Available since API level 26.
    pub fn allocate(&self, desc: AHardwareBufferDesc) -> Option<HBRef> {
        if self.api_level < 26 {
            return None;
        }
        let mut desc = desc;
        let mut buffer: *mut AHardwareBuffer = null_mut();
        unsafe {
            if (*self.AHardwareBuffer_allocate)(NonNull::new_unchecked(&mut desc), NonNull::new_unchecked(&mut buffer as *mut *mut AHardwareBuffer)) != 0 {
                buffer = null_mut();
            }
        }
        if buffer == null_mut() {
            return None;
        }
        return Some(HBRef { data: unsafe { NonNull::new_unchecked(buffer) } });
    }
    
    /// Return a description of the AHardwareBuffer.  
    /// Available since API level 26.
    pub fn describe(&self, buffer: &HBRef) -> Option<AHardwareBufferDesc> {
        if self.api_level < 26 {
            return None;
        }
        let mut desc: MaybeUninit<AHardwareBufferDesc> = MaybeUninit::uninit();
        unsafe {
            (*self.AHardwareBuffer_describe)(buffer.get(), NonNull::new_unchecked(desc.as_mut_ptr()));
            return Some(desc.assume_init());
        }
    }
    
    /// Get the system wide unique id for an AHardwareBuffer.  
    /// Available since API level 31.
    pub fn get_id(&self, buffer: &HBRef) -> Option<u64> {
        if self.api_level < 31 {
            return None;
        }
        let mut id: u64 = 0;
        unsafe {
            if (*self.AHardwareBuffer_getId)(buffer.get(), NonNull::new_unchecked(&mut id)) != 0 {
                None
            } else {
                Some(id)
            }
        }
    }
    
    /// Test whether the given format and usage flag combination is allocatable.  
    /// If this function returns true, it means that a buffer with the given description can be allocated on this implementation,
    /// unless resource exhaustion occurs. If this function returns false, it means that the allocation of the given description will never succeed.  
    /// The return value of this function may depend on all fields in the description, except stride, which is always ignored.
    /// For example, some implementations have implementation-defined limits on texture size and layer count.  
    /// Available since API level 29.
    pub fn is_supported(&self, desc: &AHardwareBufferDesc) -> bool {
        if self.api_level < 29 {
            return false;
        }
        if (unsafe { *self.AHardwareBuffer_isSupported })(desc.into()) == 1 {
            true
        } else {
            false
        }
    }
    
    /// Receive an AHardwareBuffer from an AF_UNIX socket. 
    /// Available since API level 26.
    pub fn recv(&self, socket: &UnixStream) -> Option<HBRef> {
        if self.api_level < 26 {
            return None;
        }
        let mut buffer: *mut AHardwareBuffer = null_mut();
        unsafe {
            if (*self.AHardwareBuffer_recvHandleFromUnixSocket)(socket.as_raw_fd(), NonNull::new_unchecked(&mut buffer as *mut *mut AHardwareBuffer)) != 0 {
                buffer = null_mut();
            }
        }
        if buffer == null_mut() {
            return None;
        }
        return Some(HBRef { data: unsafe { NonNull::new_unchecked(buffer) } });
    }
    
    /// Send the AHardwareBuffer to an AF_UNIX socket. 
    /// Available since API level 26.
    pub fn send(&self, socket: &UnixStream, buffer: &HBRef) -> bool {
        if self.api_level < 26 {
            return false;
        }
        if (unsafe { *self.AHardwareBuffer_sendHandleToUnixSocket })(buffer.get(), socket.as_raw_fd()) == 0 {
            true
        } else {
            false
        }
    }
    
    /// Lock the AHardwareBuffer for direct CPU access.
    /// Available since API level 26.
    pub fn lock(&self, buffer: &HBRef, usage: u64, fence: Option<BorrowedFd>, rect: Option<&ARect>) -> *mut c_void {
        if self.api_level < 26 {
            return null_mut();
        }
        let mut out: *mut c_void = null_mut();
        if (unsafe { *self.AHardwareBuffer_lock })(buffer.get(), usage, fence.and_then(|f| Some(f.as_raw_fd())).unwrap_or(-1), rect.and_then(|r| Some(r as *const ARect)).unwrap_or(null()), unsafe { NonNull::new_unchecked(&mut out) }) != 0 {
            out = null_mut();
        }
        return out;
    }
    
    /// Lock an AHardwareBuffer for direct CPU access. 
    /// Available since API level 29. 
    pub fn lock_info(&self, buffer: &HBRef, usage: u64, fence: Option<BorrowedFd>, rect: Option<&ARect>) -> Option<LockInfo> {
        if self.api_level < 29 {
            return None;
        }
        let mut bpp: i32 = 0;
        let mut bps: i32 = 0;
        let mut out: *mut c_void = null_mut();
        if (unsafe { *self.AHardwareBuffer_lockAndGetInfo })(buffer.get(), usage, fence.and_then(|f| Some(f.as_raw_fd())).unwrap_or(-1), rect.and_then(|r| Some(r as *const ARect)).unwrap_or(null()), unsafe { NonNull::new_unchecked(&mut out) }, (&mut bpp).into(), (&mut bps).into()) != 0 {
            return None;
        }
        return Some(LockInfo { address: out, bytes_per_pixel: bpp, bytes_per_stride: bps });
    }
    
    /// Lock an AHardwareBuffer for direct CPU access. 
    /// Available since API level 29. 
    pub fn lock_planes(&self, buffer: &HBRef, usage: u64, fence: Option<BorrowedFd>, rect: Option<&ARect>) -> Option<AHardwareBufferPlanes> {
        if self.api_level < 29 {
            return None;
        }
        let mut out: MaybeUninit<AHardwareBufferPlanes> = MaybeUninit::uninit();
        if (unsafe { *self.AHardwareBuffer_lockPlanes })(buffer.get(), usage, fence.and_then(|f| Some(f.as_raw_fd())).unwrap_or(-1), rect.and_then(|r| Some(r as *const ARect)).unwrap_or(null()), unsafe { NonNull::new_unchecked(out.as_mut_ptr()) }) != 0 {
            return None;
        }
        return Some(unsafe { out.assume_init() });
    }
    
    /// Unlock the AHardwareBuffer from direct CPU access. 
    /// Available since API level 26.
    pub fn unlock(&self, buffer: &HBRef) -> bool {
        if self.api_level < 26 {
            return false;
        }
        if (unsafe { *self.AHardwareBuffer_unlock })(buffer.get(), null_mut()) == 0 {
            true
        } else {
            false
        }
    }
}

/// Container for a mapped AHardwareBuffer and additional information.
pub struct LockInfo {
    /// The address of the mapped buffer.
    pub address: *mut c_void,
    /// The bytes per pixel.
    pub bytes_per_pixel: i32,
    /// The bytes per stride.
    pub bytes_per_stride: i32,
}

/// A reference-counting pointer to an AHardwareBuffer.
#[repr(transparent)]
pub struct HBRef {
    data: NonNull<AHardwareBuffer>
}

impl HBRef {
    /// Gets the underlying AHardwareBuffer pointer, without modifying the reference count.
    pub fn get(&self) -> NonNull<AHardwareBuffer> {
        self.data
    }
}

impl Clone for HBRef {
    fn clone(&self) -> Self {
        unsafe { (*HB.AHardwareBuffer_acquire)(self.data) };
        Self { data: self.data.clone() }
    }
}


impl Drop for HBRef {
    fn drop(&mut self) {
        unsafe { (*HB.AHardwareBuffer_release)(self.data) };
    }
}
