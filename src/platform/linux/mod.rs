use crate::{GpuInfo, DiskIoStats, NetworkStats, ProcessStats, PowerInfo, PerformanceMetrics, InterfaceStats};
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, NetworksExt};
use nvml_wrapper::{Nvml, enum_wrappers::device::TemperatureSensor};
use procfs::net::TcpState;

pub fn get_gpu_info() -> Option<GpuInfo> {
    if let Ok(nvml) = Nvml::init() {
        if let Ok(device) = nvml.device_by_index(0) {
            if let (Ok(utilization), Ok(memory), Ok(temp), Ok(power)) = (
                device.utilization_rates(),
                device.memory_info(),
                device.temperature(TemperatureSensor::Gpu),
                device.power_usage()
            ) {
                return Some(GpuInfo {
                    vendor: "NVIDIA".to_string(),
                    model: device.name().unwrap_or_default(),
                    usage: utilization.gpu as f32,
                    memory_total: memory.total,
                    memory_used: memory.used,
                    temperature: Some(temp as f32),
                    power_usage: Some(power as f32 / 1000.0),
                });
            }
        }
    }
    None
}

pub fn get_disk_io_stats() -> Vec<DiskIoStats> {
    if let Ok(disks) = procfs::diskstats() {
        disks.iter().map(|disk| {
            DiskIoStats {
                device: disk.name.clone(),
                reads: disk.reads,
                writes: disk.writes,
                read_bytes: disk.sectors_read * 512,
                write_bytes: disk.sectors_written * 512,
                read_time: disk.time_reading,
                write_time: disk.time_writing,
                io_in_progress: disk.time_in_progress,
            }
        }).collect()
    } else {
        Vec::new()
    }
}

pub fn get_network_stats(sys: &System) -> NetworkStats {
    let tcp = procfs::net::tcp().unwrap_or_default();
    let udp = procfs::net::udp().unwrap_or_default();
    
    let tcp_connections = tcp.len();
    let udp_connections = udp.len();
    
    let tcp_listen_ports: Vec<u16> = tcp.iter()
        .filter(|conn| conn.state == TcpState::Listen)
        .map(|conn| conn.local_address.port())
        .collect();

    let udp_listen_ports: Vec<u16> = udp.iter()
        .map(|conn| conn.local_address.port())
        .collect();

    NetworkStats {
        tcp_connections,
        udp_connections,
        tcp_listen_ports,
        udp_listen_ports,
        interface_stats: get_interface_stats(sys),
    }
}

pub fn get_interface_stats(sys: &System) -> Vec<InterfaceStats> {
    sys.networks().iter().map(|(name, _data)| {
        InterfaceStats {
            name: name.clone(),
            rx_errors: 0,
            tx_errors: 0,
            rx_dropped: 0,
            tx_dropped: 0,
            rx_bytes_sec: 0.0,
            tx_bytes_sec: 0.0,
            rx_packets_sec: 0.0,
            tx_packets_sec: 0.0,
        }
    }).collect()
}

pub fn get_process_stats(sys: &System) -> ProcessStats {
    let mut stats = ProcessStats {
        zombie_count: 0,
        sleeping_count: 0,
        blocked_count: 0,
        total_cpu_usage: 0.0,
        total_memory_usage: 0,
    };

    for process in sys.processes().values() {
        match process.status() {
            sysinfo::ProcessStatus::Zombie => stats.zombie_count += 1,
            sysinfo::ProcessStatus::Sleep => stats.sleeping_count += 1,
            sysinfo::ProcessStatus::Stop => stats.blocked_count += 1,
            _ => {}
        }
        stats.total_cpu_usage += process.cpu_usage();
        stats.total_memory_usage += process.memory();
    }

    stats
}

pub fn get_power_info() -> PowerInfo {
    PowerInfo {
        ac_powered: true,
        battery_present: false,
        battery_percentage: None,
        battery_time_remaining: None,
        power_consumption: None,
    }
}

pub fn get_performance_metrics(sys: &System) -> PerformanceMetrics {
    if let Ok(stat) = procfs::process::Process::myself() {
        if let Ok(stat) = stat.stat() {
            return PerformanceMetrics {
                iowait_percentage: 0.0,
                steal_percentage: 0.0,
                system_percentage: stat.stime as f32,
                user_percentage: stat.utime as f32,
                nice_percentage: stat.priority as f32,
                irq_percentage: 0.0,
                softirq_percentage: 0.0,
                cpu_queue_length: sys.processes().len() as u64,
                context_switches: stat.num_threads as u64,
                interrupts: 0,
            };
        }
    }

    PerformanceMetrics {
        iowait_percentage: 0.0,
        steal_percentage: 0.0,
        system_percentage: sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32,
        user_percentage: 0.0,
        nice_percentage: 0.0,
        irq_percentage: 0.0,
        softirq_percentage: 0.0,
        cpu_queue_length: sys.processes().len() as u64,
        context_switches: 0,
        interrupts: 0,
    }
} 