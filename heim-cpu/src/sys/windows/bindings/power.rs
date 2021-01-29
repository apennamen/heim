#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

use std::io;
use std::mem;
use std::ptr;

use winapi::shared::{minwindef, ntstatus};
use winapi::um::{powerbase, winnt};

use heim_common::prelude::{Error, Result};

use super::get_system_info;

#[repr(C)]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct PROCESSOR_POWER_INFORMATION {
    pub Number: minwindef::ULONG,
    pub MaxMhz: minwindef::ULONG,
    pub CurrentMhz: minwindef::ULONG,
    pub MhzLimit: minwindef::ULONG,
    pub MaxIdleState: minwindef::ULONG,
    pub CurrentIdleState: minwindef::ULONG,
}


// Safe wrapper around the `CallNtPowerInformation`
pub fn query_processor_information() -> Result<Vec<PROCESSOR_POWER_INFORMATION>> {
    let info = get_system_info();
    if info.dwNumberOfProcessors == 0 {
        let inner = io::Error::from(io::ErrorKind::InvalidData);
        return Err(Error::from(inner).with_message("No processors were found"));
    }

    let proc_amount = info.dwNumberOfProcessors as usize;
    let mut processors = Vec::<PROCESSOR_POWER_INFORMATION>::with_capacity(proc_amount);
    let buffer_length = proc_amount * mem::size_of::<PROCESSOR_POWER_INFORMATION>();

    let result = unsafe {
        powerbase::CallNtPowerInformation(
            winnt::ProcessorInformation,
            ptr::null_mut(),
            0,
            processors.as_mut_ptr() as *mut _,
            buffer_length as minwindef::ULONG,
        )
    };

    if result == ntstatus::STATUS_SUCCESS {
        unsafe {
            processors.set_len(proc_amount);
        }

        Ok(processors)
    } else {
        Err(Error::last_os_error().with_ffi("CallNtPowerInformation"))
    }
}
