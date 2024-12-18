use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, Error as ActixError};
use actix_cors::Cors;
use serde::Serialize;
use sysinfo::{CpuExt, DiskExt, System, SystemExt, ComponentExt, NetworksExt, ProcessExt, NetworkExt};
use std::sync::Arc;
use parking_lot::Mutex;
use log::{info, warn, error};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::env;
use std::net::TcpListener;

mod platform;

const TEMP_HISTORY_SIZE: usize = 60;
const TEMP_WARNING_THRESHOLD: f32 = 80.0;
const CACHE_DURATION_MS: u64 = 1000;
const DEFAULT_PORT: u16 = 9527;
const MAX_PORT_ATTEMPTS: u16 = 100;

#[derive(Serialize, Clone)]
struct SystemInfo {
    // CPU 相关信息
    cpu_usage: Vec<f32>,
    cpu_temp: Option<f32>,
    cpu_brand: String,
    cpu_frequency: Vec<u64>,
    cpu_cores: usize,
    cpu_physical_cores: usize,
    cpu_vendor_id: String,
    cpu_load_avg: LoadAverage,

    // GPU 相关信息
    gpu_info: Option<GpuInfo>,

    // 内存相关信息
    memory_total: u64,
    memory_used: u64,
    memory_free: u64,
    memory_available: u64,
    memory_usage: f32,
    swap_total: u64,
    swap_used: u64,
    swap_free: u64,
    swap_usage: f32,

    // 磁盘相关信息
    disks: Vec<DiskInfo>,
    total_disk_space: u64,
    total_disk_used: u64,
    total_disk_free: u64,
    disk_io_stats: Vec<DiskIoStats>,

    // 系统信息
    system_name: Option<String>,
    kernel_version: Option<String>,
    os_version: Option<String>,
    host_name: Option<String>,
    boot_time: u64,
    uptime: u64,
    load_average: LoadAverage,

    // 进程信息
    process_count: usize,
    thread_count: usize,
    running_process_count: usize,
    process_stats: ProcessStats,

    // 温度信息
    temperatures: Vec<TempInfo>,
    temp_warnings: Vec<String>,
    timestamp: DateTime<Utc>,

    // 网络信息
    networks: Vec<NetworkInfo>,
    total_rx_bytes: u64,
    total_tx_bytes: u64,
    network_stats: NetworkStats,

    // 电源信息
    power_info: PowerInfo,

    // 性能指标
    performance_metrics: PerformanceMetrics,
}

#[derive(Serialize, Clone)]
struct DiskInfo {
    name: String,
    mount_point: String,
    total_space: u64,
    available_space: u64,
    usage_percentage: f32,
}

#[derive(Serialize, Clone)]
struct TempInfo {
    label: String,
    temp: f32,
    status: TempStatus,
}

#[derive(Serialize, Clone)]
enum TempStatus {
    Normal,
    Warning,
    Critical,
}

#[derive(Serialize)]
struct HealthStatus {
    status: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize, Clone)]
struct NetworkInfo {
    interface: String,
    received_bytes: u64,
    transmitted_bytes: u64,
    received_packets: u64,
    transmitted_packets: u64,
    mac_address: Option<String>,
    ip_addresses: Vec<String>,
}

#[derive(Serialize, Clone)]
struct LoadAverage {
    one: f64,
    five: f64,
    fifteen: f64,
}

#[derive(Serialize, Clone)]
struct GpuInfo {
    vendor: String,
    model: String,
    usage: f32,
    memory_total: u64,
    memory_used: u64,
    temperature: Option<f32>,
    power_usage: Option<f32>,
}

#[derive(Serialize, Clone)]
struct DiskIoStats {
    device: String,
    reads: u64,
    writes: u64,
    read_bytes: u64,
    write_bytes: u64,
    read_time: u64,
    write_time: u64,
    io_in_progress: u64,
}

#[derive(Serialize, Clone)]
struct ProcessStats {
    zombie_count: usize,
    sleeping_count: usize,
    blocked_count: usize,
    total_cpu_usage: f32,
    total_memory_usage: u64,
}

#[derive(Serialize, Clone)]
struct NetworkStats {
    tcp_connections: usize,
    udp_connections: usize,
    tcp_listen_ports: Vec<u16>,
    udp_listen_ports: Vec<u16>,
    interface_stats: Vec<InterfaceStats>,
}

#[derive(Serialize, Clone)]
struct InterfaceStats {
    name: String,
    rx_errors: u64,
    tx_errors: u64,
    rx_dropped: u64,
    tx_dropped: u64,
    rx_bytes_sec: f64,
    tx_bytes_sec: f64,
    rx_packets_sec: f64,
    tx_packets_sec: f64,
}

#[derive(Serialize, Clone)]
struct PowerInfo {
    ac_powered: bool,
    battery_present: bool,
    battery_percentage: Option<f32>,
    battery_time_remaining: Option<u64>,
    power_consumption: Option<f32>,
}

#[derive(Serialize, Clone)]
struct PerformanceMetrics {
    iowait_percentage: f32,
    steal_percentage: f32,
    system_percentage: f32,
    user_percentage: f32,
    nice_percentage: f32,
    irq_percentage: f32,
    softirq_percentage: f32,
    cpu_queue_length: u64,
    context_switches: u64,
    interrupts: u64,
}

struct AppState {
    sys: Mutex<System>,
    temp_history: Mutex<VecDeque<(DateTime<Utc>, Vec<TempInfo>)>>,
    last_update: Mutex<DateTime<Utc>>,
    cached_info: Mutex<Option<SystemInfo>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            sys: Mutex::new(System::new_all()),
            temp_history: Mutex::new(VecDeque::with_capacity(TEMP_HISTORY_SIZE)),
            last_update: Mutex::new(Utc::now()),
            cached_info: Mutex::new(None),
        }
    }
}

#[get("/api/system")]
async fn get_system_info(data: web::Data<Arc<AppState>>) -> Result<HttpResponse, ActixError> {
    let now = Utc::now();
    let mut cached_info = data.cached_info.lock();
    let last_update = *data.last_update.lock();

    if let Some(info) = cached_info.as_ref() {
        if (now - last_update).num_milliseconds() < CACHE_DURATION_MS as i64 {
            return Ok(HttpResponse::Ok()
                .insert_header(("X-Cache-Status", "Hit"))
                .json(info));
        }
    }

    let mut sys = data.sys.lock();
    sys.refresh_all();
    sys.refresh_components();
    sys.refresh_networks();
    sys.refresh_networks_list();
    sys.refresh_processes();

    // CPU信息
    let cpu_usage: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
    let cpu_frequency: Vec<u64> = sys.cpus().iter().map(|cpu| cpu.frequency()).collect();
    let cpu_brand = sys.cpus().first().map(|cpu| cpu.brand().to_string()).unwrap_or_default();
    let cpu_vendor_id = sys.cpus().first().map(|cpu| cpu.vendor_id().to_string()).unwrap_or_default();
    let cpu_cores = sys.cpus().len();
    let cpu_physical_cores = sys.physical_core_count().unwrap_or(0);

    // 温度信息和警告
    let mut temperatures = Vec::new();
    let mut temp_warnings = Vec::new();
    
    for component in sys.components() {
        let temp = component.temperature();
        let status = if temp >= TEMP_WARNING_THRESHOLD {
            temp_warnings.push(format!("{} temperature is too high: {:.1}°C", component.label(), temp));
            TempStatus::Critical
        } else if temp >= TEMP_WARNING_THRESHOLD - 10.0 {
            temp_warnings.push(format!("{} temperature is getting high: {:.1}°C", component.label(), temp));
            TempStatus::Warning
        } else {
            TempStatus::Normal
        };

        temperatures.push(TempInfo {
            label: component.label().to_string(),
            temp,
            status,
        });
    }

    // 记录温度警告
    if !temp_warnings.is_empty() {
        warn!("Temperature warnings: {:?}", &temp_warnings);
    }

    // 更新温度历史记录
    let mut temp_history = data.temp_history.lock();
    temp_history.push_back((now, temperatures.clone()));
    if temp_history.len() > TEMP_HISTORY_SIZE {
        temp_history.pop_front();
    }

    // CPU温度
    let cpu_temp = temperatures.iter()
        .find(|t| t.label.to_lowercase().contains("cpu"))
        .map(|t| t.temp);

    // 内存信息
    let memory_total = sys.total_memory();
    let memory_used = sys.used_memory();
    let memory_free = sys.free_memory();
    let memory_available = sys.available_memory();
    let memory_usage = (memory_used as f32 / memory_total as f32) * 100.0;
    let swap_total = sys.total_swap();
    let swap_used = sys.used_swap();
    let swap_free = sys.free_swap();
    let swap_usage = if swap_total > 0 {
        (swap_used as f32 / swap_total as f32) * 100.0
    } else {
        0.0
    };

    // 磁盘信息
    let mut total_disk_space = 0;
    let mut total_disk_used = 0;
    let mut total_disk_free = 0;
    let disks: Vec<DiskInfo> = sys.disks().iter().map(|disk| {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;
        total_disk_space += total;
        total_disk_used += used;
        total_disk_free += available;
        DiskInfo {
            name: disk.name().to_string_lossy().into_owned(),
            mount_point: disk.mount_point().to_string_lossy().into_owned(),
            total_space: total,
            available_space: available,
            usage_percentage: (used as f32 / total as f32) * 100.0,
        }
    }).collect();

    // 系统信息
    let system_info = SystemInfo {
        // CPU
        cpu_usage,
        cpu_temp,
        cpu_brand,
        cpu_frequency,
        cpu_cores,
        cpu_physical_cores,
        cpu_vendor_id,
        cpu_load_avg: LoadAverage {
            one: sys.load_average().one,
            five: sys.load_average().five,
            fifteen: sys.load_average().fifteen,
        },

        // GPU
        gpu_info: platform::get_gpu_info(),

        // 内存
        memory_total,
        memory_used,
        memory_free,
        memory_available,
        memory_usage,
        swap_total,
        swap_used,
        swap_free,
        swap_usage,

        // 磁盘
        disks,
        total_disk_space,
        total_disk_used,
        total_disk_free,
        disk_io_stats: platform::get_disk_io_stats(),

        // 系统
        system_name: sys.name(),
        kernel_version: sys.kernel_version(),
        os_version: sys.os_version(),
        host_name: sys.host_name(),
        boot_time: sys.boot_time(),
        uptime: sys.uptime(),
        load_average: LoadAverage {
            one: sys.load_average().one,
            five: sys.load_average().five,
            fifteen: sys.load_average().fifteen,
        },

        // 进程
        process_count: sys.processes().len(),
        thread_count: sys.processes().len(),
        running_process_count: sys.processes().values()
            .filter(|p| p.status() == sysinfo::ProcessStatus::Run)
            .count(),
        process_stats: platform::get_process_stats(&sys),

        // 温度
        temperatures,
        temp_warnings: temp_warnings.clone(),
        timestamp: now,

        // 网络
        networks: sys.networks().iter().map(|(name, data)| {
            NetworkInfo {
                interface: name.clone(),
                received_bytes: data.total_received(),
                transmitted_bytes: data.total_transmitted(),
                received_packets: data.total_packets_received(),
                transmitted_packets: data.total_packets_transmitted(),
                mac_address: Some(data.mac_address().to_string()),
                ip_addresses: Vec::new(),
            }
        }).collect(),
        total_rx_bytes: sys.networks().iter().map(|(_, data)| data.total_received()).sum(),
        total_tx_bytes: sys.networks().iter().map(|(_, data)| data.total_transmitted()).sum(),
        network_stats: platform::get_network_stats(&sys),

        // 电源
        power_info: platform::get_power_info(),

        // 性能指标
        performance_metrics: platform::get_performance_metrics(&sys),
    };

    // 更新缓存
    *cached_info = Some(system_info.clone());
    *data.last_update.lock() = now;

    Ok(HttpResponse::Ok()
        .insert_header(("X-Cache-Status", "Miss"))
        .json(system_info))
}

#[get("/api/health")]
async fn health_check() -> impl Responder {
    let health = HealthStatus {
        status: "OK".to_string(),
        timestamp: Utc::now(),
    };
    HttpResponse::Ok().json(health)
}

#[get("/api/temperature/history")]
async fn get_temp_history(data: web::Data<Arc<AppState>>) -> impl Responder {
    let temp_history = data.temp_history.lock();
    HttpResponse::Ok().json(&*temp_history)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Initializing hardware monitoring service...");

    let app_state = Arc::new(AppState::new());
    let app_state = web::Data::new(app_state);

    // 获取环境变量中的端口，如果没有则使用默认端口
    let start_port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    // 尝试绑定端口
    let mut port = start_port;
    let mut listener = None;
    let mut attempts = 0;

    while attempts < MAX_PORT_ATTEMPTS {
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(l) => {
                listener = Some(l);
                info!("Successfully bound to port {}", port);
                break;
            }
            Err(_) => {
                warn!("Port {} is in use, trying next port", port);
                port += 1;
                attempts += 1;
            }
        }
    }

    let listener = listener.ok_or_else(|| {
        error!("Failed to bind to any port after {} attempts", MAX_PORT_ATTEMPTS);
        std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            format!("Failed to bind to any port between {} and {}", start_port, start_port + MAX_PORT_ATTEMPTS),
        )
    })?;

    info!("Server running at http://127.0.0.1:{}", port);
    
    HttpServer::new(move || {
        // 置CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(get_system_info)
            .service(health_check)
            .service(get_temp_history)
    })
    .listen(listener)?
    .run()
    .await
}
