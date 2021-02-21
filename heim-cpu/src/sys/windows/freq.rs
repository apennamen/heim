use std::io;
use std::fmt;

use heim_common::prelude::*;
use heim_common::units::{frequency, Frequency};

use wmi::{COMLibrary, WMIConnection};

use super::bindings::power::{self, PROCESSOR_POWER_INFORMATION};

trait FreqStrategy: std::fmt::Debug {
    fn current(&self) -> Frequency;
    fn max(&self) -> Option<Frequency>;
    fn min(&self) -> Option<Frequency>;
}

#[derive(Debug)]
struct WinternlStrategy(PROCESSOR_POWER_INFORMATION);

impl WinternlStrategy {
    pub async fn new() -> Result<WinternlStrategy> {
        let processors = power::query_processor_information()?;

        processors
            .into_iter()
            .next()
            .map(Self)
            .ok_or_else(|| {
                let inner = io::Error::from(io::ErrorKind::InvalidData);
                Error::from(inner).with_message("No processors were found")
            })
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

struct WMIStrategy {
    con: WMIConnection,
}

impl fmt::Debug for WMIStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WMIStrategy")
         .finish()
    }
}

impl WMIStrategy {
    pub fn new() -> Result<WMIStrategy> {
        let com_con = COMLibrary::new()?;
        let con = WMIConnection::new(com_con.into())?;
        Ok(WMIStrategy { con })
    }
}

impl FreqStrategy for WMIStrategy { 
    fn current(&self) -> Frequency {
        Frequency::new::<frequency::megahertz>(0)
    }

    fn max(&self) -> Option<Frequency> {
        None
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

    /* Replace with this
    let strategy = WMIStrategy::new();

    let strategy = match strategy {
        Ok(s) => s,
        Err(e) => WinternlStrategy::new().await?,
    };
    */

    Ok(CpuFrequency { strategy: Box::new(strategy) })
}
