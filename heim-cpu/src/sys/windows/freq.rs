use std::io;
use std::mem;
use std::ptr;

use winapi::shared::{minwindef, ntstatus};
use winapi::um::{powerbase, winnt};

use heim_common::prelude::*;
use heim_common::units::{frequency, Frequency};

use super::bindings::get_system_info;
use super::bindings::power::PROCESSOR_POWER_INFORMATION;

trait FreqStrategy: std::fmt::Debug {
    fn current(&self) -> Frequency;
    fn max(&self) -> Option<Frequency>;
    fn min(&self) -> Option<Frequency>;
}

#[derive(Debug)]
struct WinternlStrategy(PROCESSOR_POWER_INFORMATION);

impl WinternlStrategy {
    pub async fn new() -> Result<WinternlStrategy> {
        let processors = get_processors()?;

        processors
            .into_iter()
            .next()
            .map(Self)
            .ok_or_else(|| {
                let inner = io::Error::from(io::ErrorKind::InvalidData);
                Error::from(inner).with_message("No processors were found")
            })
    }

    fn get_processors() -> Result<Vec<PROCESSOR_POWER_INFORMATION>> {
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
}

impl FreqStrategy for WinternlStrategy { 
    fn current(&self) -> Frequency {
        Frequency::new::<frequency::megahertz>(self.0.CurrentMhz.into())
    }

    fn max(&self) -> Option<Frequency> {
        Some(Frequency::new::<frequency::megahertz>(self.0.MaxMhz.into()))
    }

    fn min(&self) -> Option<Frequency> {
        None
    }
}

#[derive(Debug)]
pub struct CpuFrequency {
    // We use trait object to represent the abstract strategy
    strategy: Box<dyn FreqStrategy>,
}

impl CpuFrequency {
    pub fn current(&self) -> Frequency {
        self.strategy.current()
    }

    pub fn max(&self) -> Option<Frequency> {
        self.strategy.max()
    }

    pub fn min(&self) -> Option<Frequency> {
        self.strategy.min()
    }
}

pub async fn frequency() -> Result<CpuFrequency> {
    let strategy = WinternlStrategy::new().await?;

    Ok(CpuFrequency { strategy: Box::new(strategy) })
}
