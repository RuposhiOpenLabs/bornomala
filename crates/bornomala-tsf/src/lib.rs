//! Windows Text Services Framework (TSF) in-process COM DLL and host adapter for Bornomala.
//!
//! This crate provides:
//! 1. `HostAdapter` abstraction for text hosts (Win32 edit controls, modern apps, TSF contexts).
//! 2. `SimulatedHostAdapter` for automated Linux-based unit/integration tests.
//! 3. `BornomalaTextService` with watchdog protection, inline composition, and scheme state.
//! 4. In-process COM entry points (`DllGetClassObject`, `DllCanUnloadNow`, `DllRegisterServer`, `DllUnregisterServer`)
//!    with panic-catch boundaries so no panic can ever cross FFI boundaries into host applications.

pub mod com;
pub mod host_adapter;
pub mod service;

pub use com::{CLSID_BORNOMALA, GUID, GUID_PROFILE};
pub use host_adapter::{HostAdapter, SimulatedHostAdapter};
pub use service::BornomalaTextService;
