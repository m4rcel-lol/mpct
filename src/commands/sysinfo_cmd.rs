use anyhow::Result;
use serde::Serialize;
use sysinfo::{Disks, System};

use crate::{cli::GlobalOptions, output};

#[derive(Debug, Serialize)]
struct DiskInfo {
    name: String,
    mount_point: String,
    total_space: u64,
    available_space: u64,
}

#[derive(Debug, Serialize)]
struct Info {
    os: Option<String>,
    os_version: Option<String>,
    arch: String,
    hostname: Option<String>,
    cpu_model: Option<String>,
    cpu_cores: usize,
    total_ram: u64,
    free_ram: u64,
    uptime_seconds: u64,
    disks: Vec<DiskInfo>,
}

pub fn run(global: &GlobalOptions) -> Result<()> {
    let system = System::new_all();
    let disks = Disks::new_with_refreshed_list();
    let info = Info {
        os: System::name(),
        os_version: System::os_version(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: System::host_name(),
        cpu_model: system.cpus().first().map(|cpu| cpu.brand().to_string()),
        cpu_cores: system.cpus().len(),
        total_ram: system.total_memory(),
        free_ram: system.free_memory(),
        uptime_seconds: System::uptime(),
        disks: disks
            .list()
            .iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().display().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
            })
            .collect(),
    };
    output::write_or_json(
        global,
        || {
            println!("os: {}", info.os.as_deref().unwrap_or("unknown"));
            println!(
                "os version: {}",
                info.os_version.as_deref().unwrap_or("unknown")
            );
            println!("arch: {}", info.arch);
            println!(
                "hostname: {}",
                info.hostname.as_deref().unwrap_or("unknown")
            );
            println!("cpu: {}", info.cpu_model.as_deref().unwrap_or("unknown"));
            println!("cpu cores: {}", info.cpu_cores);
            println!("ram total: {}", info.total_ram);
            println!("ram free: {}", info.free_ram);
            println!("uptime seconds: {}", info.uptime_seconds);
            for disk in &info.disks {
                println!(
                    "disk {} at {}: {} total, {} available",
                    disk.name, disk.mount_point, disk.total_space, disk.available_space
                );
            }
            Ok(())
        },
        &info,
    )
}
