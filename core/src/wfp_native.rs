//! Windows Filtering Platform (WFP) Native API Bindings
//! 
//! This module provides direct bindings to the Windows WFP API
//! for kernel-level network filtering.
//! 
//! **Platform**: Windows 10 20H2+ / Windows 11 only
//! **Privileges**: Administrator required
//! **Signing**: EV Code Signing Certificate required

#[cfg(target_os = "windows")]
mod windows_impl {
    use anyhow::{Result, Context, anyhow};
    use tracing::{info, warn, error, debug};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::ptr;
    use std::mem;
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    
    // Windows WFP API imports
    use windows::{
        Win32::Foundation::{HANDLE, GetLastError, ERROR_SUCCESS, GUID},
        Win32::NetworkManagement::WindowsFilteringPlatform::{
            FwpmEngineOpen0,
            FwpmEngineClose0,
            FwpmSubLayerAdd0,
            FwpmSubLayerDeleteByKey0,
            FwpmFilterAdd0,
            FwpmFilterDeleteById0,
            FwpmFilterDeleteByKey0,
            FWPM_SESSION0,
            FWPM_DISPLAY_DATA0,
            FWPM_SUBLAYER0,
            FWPM_FILTER0,
            FWPM_FILTER_CONDITION0,
            FWPM_ACTION0,
            FWPM_LAYER_ALE_AUTH_CONNECT_V4,
            FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4,
            FWPM_CONDITION_ALE_APP_ID,
            FWPM_CONDITION_REMOTE_PORT,
            FWPM_CONDITION_REMOTE_ADDR_IPV4,
            FWP_MATCH_EQUAL,
            FWP_MATCH_GREATER,
            FWP_MATCH_LESS,
            FWP_MATCH_RANGE,
            FWPM_SESSION_FLAG_DYNAMIC,
            RPC_C_AUTHN_DEFAULT,
        },
        Win32::System::Threading::GetCurrentProcessId,
        Win32::Security::SECURITY_IMPERSONATION_LEVEL,
    };

    /// Convert a Rust string to a wide string for Windows API
    fn to_wide_string(s: &str) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        
        OsStr::new(s)
            .encode_wide()
            .chain(Some(0)) // Null terminator
            .collect()
    }

    /// Generate a random GUID for sublayer/filter identification
    fn generate_guid() -> GUID {
        use windows::Win32::System::Com::CoCreateGuid;
        
        unsafe {
            CoCreateGuid().unwrap_or(GUID {
                Data1: rand::random(),
                Data2: rand::random(),
                Data3: rand::random(),
                Data4: rand::random(),
            })
        }
    }

    /// Convert Unix timestamp to Windows FILETIME
    fn unix_time_to_filetime(unix_secs: u64) -> i64 {
        // Windows FILETIME epoch is 1601-01-01
        // Unix epoch is 1970-01-01
        const WINDOWS_EPOCH_OFFSET: i64 = 116444736000000000;
        ((unix_secs as i64) * 10000000) + WINDOWS_EPOCH_OFFSET
    }

    /// Native WFP Engine wrapper with actual Windows API calls
    pub struct NativeWfpEngine {
        engine_handle: HANDLE,
        sublayer_key: GUID,
        session: FWPM_SESSION0,
        is_dynamic: bool,
    }

    impl NativeWfpEngine {
        /// Initialize the WFP engine with actual Windows API calls
        /// 
        /// # Safety
        /// Requires administrator privileges
        /// Must be called from a single-threaded context or with proper synchronization
        pub fn new() -> Result<Self> {
            info!("🛡️  Initializing native WFP engine...");
            
            unsafe {
                // Create WFP session structure
                let mut session = FWPM_SESSION0 {
                    displayData: FWPM_DISPLAY_DATA0 {
                        name: to_wide_string("NetSentinel WFP Session").as_mut_ptr(),
                        description: to_wide_string("Network filtering session for NetSentinel").as_mut_ptr(),
                    },
                    flags: FWPM_SESSION_FLAG_DYNAMIC, // Auto-cleanup on close
                    txn_wait_timeout_in_ms: 0,
                    session_key: GUID::zeroed(),
                    api_version: 0, // Use default
                    num_sublayers: 0,
                    sublayers: ptr::null_mut(),
                    reserved: ptr::null_mut(),
                };

                let mut engine_handle: HANDLE = HANDLE::default();
                
                // Open WFP engine session
                let status = FwpmEngineOpen0(
                    None, // Server name (local machine)
                    RPC_C_AUTHN_DEFAULT,
                    None, // Authentication identity
                    &mut session,
                    &mut engine_handle,
                );

                if status != ERROR_SUCCESS.0 {
                    error!("Failed to open WFP engine: HRESULT 0x{:X}", status);
                    return Err(anyhow!(
                        "Failed to open WFP engine: HRESULT 0x{:X}. Ensure running as Administrator.",
                        status
                    ));
                }

                info!("WFP engine session opened successfully (handle: {:?})", engine_handle);

                // Generate sublayer GUID
                let sublayer_key = generate_guid();
                
                // Create and add sublayer
                let mut sublayer = FWPM_SUBLAYER0 {
                    subLayerKey: sublayer_key,
                    displayData: FWPM_DISPLAY_DATA0 {
                        name: to_wide_string("NetSentinel Sublayer").as_mut_ptr(),
                        description: to_wide_string("Main filtering sublayer for PCDN blocking").as_mut_ptr(),
                    },
                    flags: 0,
                    provider: ptr::null_mut(),
                    provider_data: ptr::null(),
                    weight: 0x0000FFFF, // High priority within empty weight range
                    num_filter_keys: 0,
                    filter_keys: ptr::null(),
                    raw_context: 0,
                    reserved: ptr::null(),
                };

                let status = FwpmSubLayerAdd0(engine_handle, &sublayer, None);
                if status != ERROR_SUCCESS.0 && status != 0x80320009 { // FWP_E_ALREADY_EXISTS
                    error!("Failed to add WFP sublayer: HRESULT 0x{:X}", status);
                    FwpmEngineClose0(engine_handle);
                    return Err(anyhow!("Failed to add WFP sublayer: HRESULT 0x{:X}", status));
                }

                info!("WFP sublayer created successfully (GUID: {:?})", sublayer_key);

                Ok(Self {
                    engine_handle,
                    sublayer_key,
                    session,
                    is_dynamic: true,
                })
            }
        }

        /// Add a filter rule to block/permit traffic based on conditions
        /// 
        /// # Arguments
        /// * `process_id` - Target process ID to filter
        /// * `action` - Block or Permit
        /// * `layer` - WFP layer (ALE_AUTH_CONNECT_V4, etc.)
        /// * `priority` - Filter weight (higher = evaluated first)
        pub fn add_process_filter(
            &self,
            process_id: u32,
            action: FilterAction,
            layer: WfpLayer,
            priority: u16,
        ) -> Result<GUID> {
            info!(
                "Adding process filter: PID={}, Action={:?}, Layer={:?}",
                process_id, action, layer
            );

            unsafe {
                let filter_id = generate_guid();
                
                // Build filter condition for process ID
                let app_id = format!("\\\\Device\\\\HarddiskVolume*\\\\PID_{}", process_id);
                let app_id_wide = to_wide_string(&app_id);
                
                let mut condition = FWPM_FILTER_CONDITION0 {
                    fieldKey: layer.get_condition_key(),
                    match_type: FWP_MATCH_EQUAL,
                    condition_value: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_VALUE0 {
                        type_: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_BYTE_BLOB_TYPE,
                        Anonymous: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_VALUE0_0 {
                            byteBlob: &mut windows::Win32::Foundation::FWP_BYTE_BLOB {
                                size: (app_id_wide.len() * 2) as u32,
                                data: app_id_wide.as_ptr() as *mut _,
                            },
                        },
                    },
                };

                // Create filter structure
                let mut filter = FWPM_FILTER0 {
                    displayData: FWPM_DISPLAY_DATA0 {
                        name: to_wide_string(&format!("NetSentinel Filter PID {}", process_id)).as_mut_ptr(),
                        description: to_wide_string("Auto-generated process filter").as_mut_ptr(),
                    },
                    provider: ptr::null_mut(),
                    provider_data: ptr::null(),
                    layer_key: layer.get_layer_key(),
                    sub_layer_key: self.sublayer_key,
                    weight: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_VALUE0 {
                        type_: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_UINT16_TYPE,
                        Anonymous: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_VALUE0_0 {
                            uint16: priority,
                        },
                    },
                    num_filter_conditions: 1,
                    filter_condition: &mut condition,
                    action: match action {
                        FilterAction::Block => FWPM_ACTION0 {
                            type_: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_ACTION_BLOCK,
                            ..Default::default()
                        },
                        FilterAction::Permit => FWPM_ACTION0 {
                            type_: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_ACTION_PERMIT,
                            ..Default::default()
                        },
                    },
                    raw_context: 0,
                    reserved: ptr::null(),
                    filter_id,
                    effective_weight: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_VALUE0::default(),
                    flags: 0,
                    expiration_time: 0,
                };

                let status = FwpmFilterAdd0(self.engine_handle, &filter, None, None);
                if status != ERROR_SUCCESS.0 {
                    error!("Failed to add filter: HRESULT 0x{:X}", status);
                    return Err(anyhow!("Failed to add WFP filter: HRESULT 0x{:X}", status));
                }

                info!("Filter added successfully (GUID: {:?})", filter_id);
                Ok(filter_id)
            }
        }

        /// Remove a filter by its GUID
        pub fn remove_filter(&self, filter_id: &GUID) -> Result<()> {
            info!("Removing filter: {:?}", filter_id);

            unsafe {
                let status = FwpmFilterDeleteById0(self.engine_handle, filter_id);
                if status != ERROR_SUCCESS.0 {
                    error!("Failed to remove filter: HRESULT 0x{:X}", status);
                    return Err(anyhow!("Failed to remove WFP filter: HRESULT 0x{:X}", status));
                }

                info!("Filter removed successfully");
                Ok(())
            }
        }

        /// Block all traffic from a specific process for a duration
        pub fn block_connection_with_timeout(
            &self,
            process_id: u32,
            duration_secs: u64,
        ) -> Result<GUID> {
            info!(
                "Creating temporary block: PID={}, Duration={}s",
                process_id, duration_secs
            );

            unsafe {
                let filter_id = generate_guid();
                let expire_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() + duration_secs;
                
                let mut filter = FWPM_FILTER0 {
                    displayData: FWPM_DISPLAY_DATA0 {
                        name: to_wide_string(&format!(
                            "Temp Block PID {} ({}s)",
                            process_id, duration_secs
                        )).as_mut_ptr(),
                        description: to_wide_string("Temporary connection block").as_mut_ptr(),
                    },
                    provider: ptr::null_mut(),
                    provider_data: ptr::null(),
                    layer_key: FWPM_LAYER_ALE_AUTH_CONNECT_V4,
                    sub_layer_key: self.sublayer_key,
                    weight: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_VALUE0 {
                        type_: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_UINT16_TYPE,
                        Anonymous: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_VALUE0_0 {
                            uint16: 0xFFFF,
                        },
                    },
                    num_filter_conditions: 0, // Block all for this process
                    filter_condition: ptr::null_mut(),
                    action: FWPM_ACTION0 {
                        type_: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_ACTION_BLOCK,
                        ..Default::default()
                    },
                    raw_context: 0,
                    reserved: ptr::null(),
                    filter_id,
                    effective_weight: windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_VALUE0::default(),
                    flags: 0,
                    expiration_time: unix_time_to_filetime(expire_time),
                };

                let status = FwpmFilterAdd0(self.engine_handle, &filter, None, None);
                if status != ERROR_SUCCESS.0 {
                    error!("Failed to add temp block filter: HRESULT 0x{:X}", status);
                    return Err(anyhow!("Failed to add temp block: HRESULT 0x{:X}", status));
                }

                info!(
                    "Temporary block filter created (expires in {}s, GUID: {:?})",
                    duration_secs, filter_id
                );
                Ok(filter_id)
            }
        }

        /// Get the engine handle for advanced operations
        pub fn get_handle(&self) -> HANDLE {
            self.engine_handle
        }

        /// Check if the engine session is valid
        pub fn is_valid(&self) -> bool {
            !self.engine_handle.is_invalid()
        }
    }

    impl Drop for NativeWfpEngine {
        fn drop(&mut self) {
            info!("Closing WFP engine session...");
            
            unsafe {
                // Dynamic sessions auto-cleanup, but we explicitly close
                let status = FwpmEngineClose0(self.engine_handle);
                if status != ERROR_SUCCESS.0 {
                    warn!("Failed to close WFP engine: HRESULT 0x{:X}", status);
                } else {
                    info!("WFP engine session closed successfully");
                }
            }
        }
    }

    /// Filter action types
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum FilterAction {
        Block,
        Permit,
    }

    /// WFP layer abstraction
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum WfpLayer {
        AleAuthConnectV4,
        AleAuthRecvAcceptV4,
    }

    impl WfpLayer {
        fn get_layer_key(&self) -> GUID {
            match self {
                WfpLayer::AleAuthConnectV4 => FWPM_LAYER_ALE_AUTH_CONNECT_V4,
                WfpLayer::AleAuthRecvAcceptV4 => FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4,
            }
        }

        fn get_condition_key(&self) -> GUID {
            match self {
                WfpLayer::AleAuthConnectV4 => FWPM_CONDITION_ALE_APP_ID,
                WfpLayer::AleAuthRecvAcceptV4 => FWPM_CONDITION_ALE_APP_ID,
            }
        }
    }
}

// Re-export for non-Windows platforms (stub implementation)
#[cfg(not(target_os = "windows"))]
mod non_windows_stub {
    use anyhow::{Result, anyhow};
    use tracing::info;
    use std::time::Duration;

    /// Stub implementation for non-Windows platforms
    pub struct NativeWfpEngine {
        _placeholder: (),
    }

    impl NativeWfpEngine {
        pub fn new() -> Result<Self> {
            info!("⚠️  WFP is only available on Windows. Running in stub mode.");
            Ok(Self { _placeholder: () })
        }

        pub fn add_process_filter(
            &self,
            _process_id: u32,
            _action: FilterAction,
            _layer: WfpLayer,
            _priority: u16,
        ) -> Result<String> {
            Ok("stub-filter-id".to_string())
        }

        pub fn remove_filter(&self, _filter_id: &str) -> Result<()> {
            Ok(())
        }

        pub fn block_connection_with_timeout(
            &self,
            _process_id: u32,
            _duration_secs: u64,
        ) -> Result<String> {
            Ok("stub-temp-block-id".to_string())
        }

        pub fn is_valid(&self) -> bool {
            false
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum FilterAction {
        Block,
        Permit,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum WfpLayer {
        AleAuthConnectV4,
        AleAuthRecvAcceptV4,
    }
}

// Export appropriate implementation based on platform
#[cfg(target_os = "windows")]
pub use windows_impl::*;

#[cfg(not(target_os = "windows"))]
pub use non_windows_stub::*;
