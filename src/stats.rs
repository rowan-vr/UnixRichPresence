use std::borrow::Borrow;
use byte_unit::Byte;
use sysinfo::{DiskExt, SystemExt};

pub enum Statistic {
    RamUsage,
    ProcessCount,
    KernelVersion,
    DiskSpace
}

impl Statistic {
    pub fn show(&self) -> String{
        let mut system = sysinfo::System::new_all();
        match self{
            Statistic::RamUsage => format!("Using {0}/{1} ram", Byte::from(system.used_memory()).get_appropriate_unit(false), Byte::from(system.total_memory()).get_appropriate_unit(false)),
            Statistic::ProcessCount => format!("Running {0} processes", system.processes().len()),
            Statistic::KernelVersion => format!("Using kernel version {0}", system.kernel_version().expect("No kernal version found")),
            Statistic::DiskSpace => {
                let mut total = 0;
                let mut available = 0;
                for disk in system.disks(){
                    total += disk.total_space();
                    available += disk.available_space();
                }
                format!("Using {0}/{1} disk space", Byte::from(total-available).get_appropriate_unit(false), Byte::from(total).get_appropriate_unit(false))
            }
        }
    }
}



