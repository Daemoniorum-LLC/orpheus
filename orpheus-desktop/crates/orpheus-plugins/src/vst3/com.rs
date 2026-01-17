//! VST3 COM Foundation
//!
//! Implements COM (Component Object Model) fundamentals required for VST3.
//! This includes IUnknown interface, GUID handling, and reference counting.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::c_void;
use std::ptr;

/// COM result type (HRESULT equivalent)
pub type tresult = i32;

/// Result codes
pub const K_RESULT_OK: tresult = 0;
pub const K_RESULT_FALSE: tresult = 1;
pub const K_NO_INTERFACE: tresult = -1;
pub const K_RESULT_TRUE: tresult = K_RESULT_OK;
pub const K_INVALID_ARGUMENT: tresult = -2;
pub const K_NOT_IMPLEMENTED: tresult = -3;
pub const K_INTERNAL_ERROR: tresult = -4;
pub const K_NOT_INITIALIZED: tresult = -5;
pub const K_OUT_OF_MEMORY: tresult = -6;

/// 128-bit type UID (GUID equivalent)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TUID {
    pub data: [u8; 16],
}

impl TUID {
    /// Create a new TUID from bytes
    pub const fn new(data: [u8; 16]) -> Self {
        Self { data }
    }

    /// Create a null/empty TUID
    pub const fn null() -> Self {
        Self { data: [0; 16] }
    }

    /// Check if this TUID is null
    pub fn is_null(&self) -> bool {
        self.data == [0; 16]
    }

    /// Create TUID from 4 u32 values (VST3 SDK style)
    pub const fn from_u32(a: u32, b: u32, c: u32, d: u32) -> Self {
        Self {
            data: [
                (a >> 24) as u8,
                (a >> 16) as u8,
                (a >> 8) as u8,
                a as u8,
                (b >> 24) as u8,
                (b >> 16) as u8,
                (b >> 8) as u8,
                b as u8,
                (c >> 24) as u8,
                (c >> 16) as u8,
                (c >> 8) as u8,
                c as u8,
                (d >> 24) as u8,
                (d >> 16) as u8,
                (d >> 8) as u8,
                d as u8,
            ],
        }
    }
}

impl Default for TUID {
    fn default() -> Self {
        Self::null()
    }
}

/// FUnknown interface ID (COM IUnknown equivalent)
pub const FUNKNOWN_IID: TUID = TUID::from_u32(0x00000000, 0x00000000, 0xC0000000, 0x00000046);

/// IPluginBase interface ID
pub const IPLUGIN_BASE_IID: TUID = TUID::from_u32(0x22888DDB, 0x156E45AE, 0x8358B348, 0x08190625);

/// IPluginFactory interface ID
pub const IPLUGIN_FACTORY_IID: TUID = TUID::from_u32(0x7A4D811C, 0x5211A04A, 0x96CF3E39, 0xE18BFC54);

/// IPluginFactory2 interface ID
pub const IPLUGIN_FACTORY2_IID: TUID = TUID::from_u32(0x0007B650, 0xF24B4C0B, 0xA464EDB9, 0xF00B2ABB);

/// IPluginFactory3 interface ID
pub const IPLUGIN_FACTORY3_IID: TUID = TUID::from_u32(0x4555A2AB, 0xC1234E57, 0x91DD4B99, 0xC618F4B7);

/// IComponent interface ID
pub const ICOMPONENT_IID: TUID = TUID::from_u32(0xE831FF31, 0xF2D54301, 0x928EBBEE, 0x25697802);

/// IAudioProcessor interface ID
pub const IAUDIO_PROCESSOR_IID: TUID = TUID::from_u32(0x42043F99, 0xB7DA453C, 0xA569E79D, 0x9AAEC33D);

/// IEditController interface ID
pub const IEDIT_CONTROLLER_IID: TUID = TUID::from_u32(0xDCD7BBE3, 0x7742448D, 0xA874AACC, 0x979C759E);

/// IConnectionPoint interface ID
pub const ICONNECTION_POINT_IID: TUID = TUID::from_u32(0x70A4156F, 0x6E6E4026, 0x989148BF, 0xAA60D8D1);

/// IHostApplication interface ID
pub const IHOST_APPLICATION_IID: TUID = TUID::from_u32(0x58E595CC, 0xDB2D4969, 0x8B6AAF8C, 0x36A664E5);

/// FUnknown VTable (COM IUnknown equivalent)
#[repr(C)]
pub struct FUnknownVtbl {
    /// Query for an interface
    pub queryInterface:
        unsafe extern "system" fn(this: *mut c_void, iid: *const TUID, obj: *mut *mut c_void) -> tresult,
    /// Add a reference
    pub addRef: unsafe extern "system" fn(this: *mut c_void) -> u32,
    /// Release a reference
    pub release: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

/// FUnknown interface (COM IUnknown equivalent)
#[repr(C)]
pub struct FUnknown {
    pub vtbl: *const FUnknownVtbl,
}

impl FUnknown {
    /// Query for an interface
    ///
    /// # Safety
    /// Caller must ensure this points to a valid FUnknown instance.
    pub unsafe fn query_interface(&self, iid: &TUID) -> Option<*mut c_void> {
        let mut obj: *mut c_void = ptr::null_mut();
        let result = ((*self.vtbl).queryInterface)(
            self as *const _ as *mut c_void,
            iid as *const TUID,
            &mut obj,
        );
        if result == K_RESULT_OK && !obj.is_null() {
            Some(obj)
        } else {
            None
        }
    }

    /// Add a reference
    ///
    /// # Safety
    /// Caller must ensure this points to a valid FUnknown instance.
    pub unsafe fn add_ref(&self) -> u32 {
        ((*self.vtbl).addRef)(self as *const _ as *mut c_void)
    }

    /// Release a reference
    ///
    /// # Safety
    /// Caller must ensure this points to a valid FUnknown instance.
    pub unsafe fn release(&self) -> u32 {
        ((*self.vtbl).release)(self as *const _ as *mut c_void)
    }
}

/// Raw COM pointer wrapper (no automatic reference counting)
///
/// Use this for manually managing COM object lifetimes.
/// Call `release()` when done with the pointer.
pub struct ComPtr<T> {
    ptr: *mut T,
}

impl<T> ComPtr<T> {
    /// Create a new ComPtr from a raw pointer (takes ownership, does NOT addRef)
    ///
    /// # Safety
    /// The pointer must be valid and the caller is transferring ownership.
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    /// Get a reference to the inner value
    ///
    /// # Safety
    /// Caller must ensure the pointer is still valid.
    pub unsafe fn as_ref(&self) -> &T {
        &*self.ptr
    }

    /// Check if the pointer is null
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Release the COM object and set pointer to null
    ///
    /// # Safety
    /// Must be called on a valid COM object. Should only be called once.
    pub unsafe fn release(&mut self)
    where
        T: AsRef<FUnknown>,
    {
        if !self.ptr.is_null() {
            (*self.ptr).as_ref().release();
            self.ptr = ptr::null_mut();
        }
    }

    /// Query for another interface
    ///
    /// # Safety
    /// Must be called on a valid COM object.
    pub unsafe fn query_interface<U>(&self, iid: &TUID) -> Option<ComPtr<U>>
    where
        T: AsRef<FUnknown>,
    {
        let unknown = (*self.ptr).as_ref();
        unknown.query_interface(iid).map(|p| ComPtr { ptr: p as *mut U })
    }
}

/// Plugin class info structure
#[repr(C)]
pub struct PClassInfo {
    /// Class ID
    pub cid: TUID,
    /// Cardinality (number of instances allowed)
    pub cardinality: i32,
    /// Category (e.g., "Audio Module Class")
    pub category: [u8; 32],
    /// Class name
    pub name: [u8; 64],
}

impl Default for PClassInfo {
    fn default() -> Self {
        Self {
            cid: TUID::null(),
            cardinality: 0,
            category: [0; 32],
            name: [0; 64],
        }
    }
}

/// Plugin class info 2 structure (extended)
#[repr(C)]
pub struct PClassInfo2 {
    /// Class ID
    pub cid: TUID,
    /// Cardinality
    pub cardinality: i32,
    /// Category
    pub category: [u8; 32],
    /// Class name
    pub name: [u8; 64],
    /// Class flags
    pub classFlags: u32,
    /// Subcategories
    pub subCategories: [u8; 128],
    /// Vendor name
    pub vendor: [u8; 64],
    /// Version string
    pub version: [u8; 64],
    /// SDK version
    pub sdkVersion: [u8; 64],
}

impl Default for PClassInfo2 {
    fn default() -> Self {
        Self {
            cid: TUID::null(),
            cardinality: 0,
            category: [0; 32],
            name: [0; 64],
            classFlags: 0,
            subCategories: [0; 128],
            vendor: [0; 64],
            version: [0; 64],
            sdkVersion: [0; 64],
        }
    }
}

/// Plugin class info with Unicode support
#[repr(C)]
pub struct PClassInfoW {
    /// Class ID
    pub cid: TUID,
    /// Cardinality
    pub cardinality: i32,
    /// Category
    pub category: [u8; 32],
    /// Class name (UTF-16)
    pub name: [u16; 64],
    /// Class flags
    pub classFlags: u32,
    /// Subcategories
    pub subCategories: [u8; 128],
    /// Vendor name (UTF-16)
    pub vendor: [u16; 64],
    /// Version string (UTF-16)
    pub version: [u16; 64],
    /// SDK version (UTF-16)
    pub sdkVersion: [u16; 64],
}

impl Default for PClassInfoW {
    fn default() -> Self {
        Self {
            cid: TUID::null(),
            cardinality: 0,
            category: [0; 32],
            name: [0; 64],
            classFlags: 0,
            subCategories: [0; 128],
            vendor: [0; 64],
            version: [0; 64],
            sdkVersion: [0; 64],
        }
    }
}

/// Factory info structure
#[repr(C)]
pub struct PFactoryInfo {
    /// Vendor name
    pub vendor: [u8; 64],
    /// Vendor URL
    pub url: [u8; 256],
    /// Vendor email
    pub email: [u8; 128],
    /// Factory flags
    pub flags: i32,
}

impl Default for PFactoryInfo {
    fn default() -> Self {
        Self {
            vendor: [0; 64],
            url: [0; 256],
            email: [0; 128],
            flags: 0,
        }
    }
}

/// IPluginFactory VTable
#[repr(C)]
pub struct IPluginFactoryVtbl {
    /// Base FUnknown vtable
    pub unknown: FUnknownVtbl,
    /// Get factory info
    pub getFactoryInfo:
        unsafe extern "system" fn(this: *mut c_void, info: *mut PFactoryInfo) -> tresult,
    /// Count classes in factory
    pub countClasses: unsafe extern "system" fn(this: *mut c_void) -> i32,
    /// Get class info by index
    pub getClassInfo:
        unsafe extern "system" fn(this: *mut c_void, index: i32, info: *mut PClassInfo) -> tresult,
    /// Create class instance
    pub createInstance: unsafe extern "system" fn(
        this: *mut c_void,
        cid: *const TUID,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> tresult,
}

/// IPluginFactory interface
#[repr(C)]
pub struct IPluginFactory {
    pub vtbl: *const IPluginFactoryVtbl,
}

impl AsRef<FUnknown> for IPluginFactory {
    fn as_ref(&self) -> &FUnknown {
        unsafe { &*(self as *const _ as *const FUnknown) }
    }
}

/// IPluginFactory2 VTable (extends IPluginFactory)
#[repr(C)]
pub struct IPluginFactory2Vtbl {
    /// Base IPluginFactory vtable
    pub factory: IPluginFactoryVtbl,
    /// Get class info 2 by index
    pub getClassInfo2:
        unsafe extern "system" fn(this: *mut c_void, index: i32, info: *mut PClassInfo2) -> tresult,
}

/// IPluginFactory2 interface
#[repr(C)]
pub struct IPluginFactory2 {
    pub vtbl: *const IPluginFactory2Vtbl,
}

impl AsRef<FUnknown> for IPluginFactory2 {
    fn as_ref(&self) -> &FUnknown {
        unsafe { &*(self as *const _ as *const FUnknown) }
    }
}

/// IPluginFactory3 VTable (extends IPluginFactory2)
#[repr(C)]
pub struct IPluginFactory3Vtbl {
    /// Base IPluginFactory2 vtable
    pub factory2: IPluginFactory2Vtbl,
    /// Get class info with Unicode by index
    pub getClassInfoUnicode:
        unsafe extern "system" fn(this: *mut c_void, index: i32, info: *mut PClassInfoW) -> tresult,
    /// Set host context
    pub setHostContext:
        unsafe extern "system" fn(this: *mut c_void, context: *mut c_void) -> tresult,
}

/// IPluginFactory3 interface
#[repr(C)]
pub struct IPluginFactory3 {
    pub vtbl: *const IPluginFactory3Vtbl,
}

impl AsRef<FUnknown> for IPluginFactory3 {
    fn as_ref(&self) -> &FUnknown {
        unsafe { &*(self as *const _ as *const FUnknown) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuid_creation() {
        let tuid = TUID::from_u32(0x12345678, 0x9ABCDEF0, 0x11223344, 0x55667788);
        assert_eq!(tuid.data[0], 0x12);
        assert_eq!(tuid.data[4], 0x9A);
        assert!(!tuid.is_null());
    }

    #[test]
    fn test_tuid_null() {
        let tuid = TUID::null();
        assert!(tuid.is_null());
        assert_eq!(tuid.data, [0; 16]);
    }

    #[test]
    fn test_result_codes() {
        assert_eq!(K_RESULT_OK, 0);
        assert!(K_NO_INTERFACE < 0);
        assert!(K_INVALID_ARGUMENT < 0);
    }

    #[test]
    fn test_pclass_info_default() {
        let info = PClassInfo::default();
        assert!(info.cid.is_null());
        assert_eq!(info.cardinality, 0);
    }

    #[test]
    fn test_pfactory_info_default() {
        let info = PFactoryInfo::default();
        assert_eq!(info.flags, 0);
    }

    #[test]
    fn test_interface_ids() {
        // Verify IIDs are not null
        assert!(!FUNKNOWN_IID.is_null());
        assert!(!IPLUGIN_FACTORY_IID.is_null());
        assert!(!ICOMPONENT_IID.is_null());
        assert!(!IAUDIO_PROCESSOR_IID.is_null());
    }
}
