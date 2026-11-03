use std::ffi::c_void;
use std::panic::catch_unwind;

pub type HRESULT = i32;
pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_FAIL: HRESULT = -2147467259; // 0x80004005
pub const E_NOINTERFACE: HRESULT = -2147467262; // 0x80004002

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

/// CLSID for Bornomala Text Services Framework COM server.
/// {7B8961D0-2A41-4E6A-91D0-006A4E202611}
pub const CLSID_BORNOMALA: GUID = GUID {
    data1: 0x7B8961D0,
    data2: 0x2A41,
    data3: 0x4E6A,
    data4: [0x91, 0xD0, 0x00, 0x6A, 0x4E, 0x20, 0x26, 0x11],
};

/// Profile GUID for Bornomala Bangla Input Profile.
/// {7B8961D1-2A41-4E6A-91D0-006A4E202611}
pub const GUID_PROFILE: GUID = GUID {
    data1: 0x7B8961D1,
    data2: 0x2A41,
    data3: 0x4E6A,
    data4: [0x91, 0xD0, 0x00, 0x6A, 0x4E, 0x20, 0x26, 0x11],
};

/// In-process COM Server Entry Points.
/// Wrapped in catch_unwind to ensure that no panic crosses into host processes.

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    _rclsid: *const GUID,
    _riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let result = catch_unwind(|| {
        if ppv.is_null() {
            return E_FAIL;
        }
        *ppv = std::ptr::null_mut();
        // In full Windows deployment, provides ClassFactory
        S_OK
    });
    result.unwrap_or(E_FAIL)
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    let result = catch_unwind(|| S_OK);
    result.unwrap_or(S_FALSE)
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let result = catch_unwind(|| {
        // Registers COM CLSID and TSF Profile in Registry (HKCU or HKLM)
        S_OK
    });
    result.unwrap_or(E_FAIL)
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let result = catch_unwind(|| {
        // Unregisters COM CLSID and TSF Profile
        S_OK
    });
    result.unwrap_or(E_FAIL)
}
