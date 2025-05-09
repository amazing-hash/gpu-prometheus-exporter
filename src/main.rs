use lazy_static::lazy_static;
use prometheus::{opts, register_gauge_vec, register_int_gauge_vec, GaugeVec, IntGaugeVec};
use prometheus::{Encoder, TextEncoder};

use axum::{routing::get, Router};
use std::collections::HashMap;
use std::{net::SocketAddr, str::FromStr};

lazy_static! {
    pub static ref NVIDIA_SMI_GPU_INFO: GaugeVec =
        register_gauge_vec!(opts!("nvidia_smi_gpu_info", "GPU common information."),
        &["index", "name", "driver_version", "vbios_version"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_TOTAL_MEMORY: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_total_memory", "Total installed GPU memory."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_USED_MEMORY: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_used_memory", "Total memory allocated by active contexts."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_UTILIZATION_MEMORY: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_utilization_memory", "Percent of time over the past sample period during which global (device) memory was being read or written."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_UTILIZATION: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_utilization", "Percent of time over the past sample period during which one or more kernels was executing on the GPU."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_CUDA_SPECIFICATION: GaugeVec =
        register_gauge_vec!(opts!("nvidia_smi_gpu_cuda_specification", "The CUDA Compute Capability."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_TEMPERATURE: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_temperature", "Core GPU temperature. in degrees C."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_FAN_SPEED: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_fan_speed", "The fan speed value is the percent of the product's maximum noise tolerance fan speed that the device's fan is currently intended to run at."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_PERFORMANCE_STATE: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_performance_state", "The current performance state for the GPU. States range from P0 (maximum performance) to P12 (minimum performance)."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_POWER_LIMIT: GaugeVec =
        register_gauge_vec!(opts!("nvidia_smi_gpu_power_limit", "The software power limit in watts."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_POWER_DRAW: GaugeVec =
        register_gauge_vec!(opts!("nvidia_smi_gpu_power_draw", "The last measured power draw for the entire board, in watts. Only available if power management is supported. This reading is accurate to within +/- 5 watts."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_CLOCK_GRAPHICS: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_clock_graphics", "Current frequency of graphics (shader) clock."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_CLOCK_MAX_GRAPHICS: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_clock_max_graphics", "Maximum frequency of graphics (shader) clock"),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_CLOCK_MEMORY: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_clock_memory", "Current frequency of memory clock."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_CLOCK_MAX_MEMORY: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_clock_max_memory", "Maximum frequency of memory clock."),
        &["index"]
    ).expect("Can't create a metric");
    pub static ref NVIDIA_SMI_GPU_CLOCK_VIDEO: IntGaugeVec =
        register_int_gauge_vec!(opts!("nvidia_smi_gpu_clock_video", "Current frequency of video encoder/decoder clock."),
        &["index"]
    ).expect("Can't create a metric");
}

use std::process::Command;

const PARAMS: &[&str] = &[
    "index",
    "name",
    "driver_version",
    "vbios_version",
    "memory.total",
    "memory.used",
    "utilization.memory",
    "compute_cap",
    "temperature.gpu",
    "utilization.gpu",
    "fan.speed",
    "pstate",
    "power.limit",
    "power.draw",
    "clocks.current.graphics",
    "clocks.current.memory",
    "clocks.current.video",
    "clocks.max.graphics",
    "clocks.max.memory",
];

fn main() {
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Runtime::new().unwrap_or_else(|err| {
        tracing::error!("{:?}", err);
        std::process::exit(1)
    });
    rt.block_on(async {
        let app = Router::new().route("/metrics", get(metrics));
        let addr = SocketAddr::from_str("0.0.0.0:9835").unwrap_or_else(|err| {
            tracing::error!("{}. Fatal error. The app will be stopped", err);
            std::process::exit(1);
        });
        tracing::info!("HTTP server listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap_or_else(|err| {
                tracing::error!("{:?}", err);
                std::process::exit(1)
            });
    });
}

pub async fn metrics() -> String {
    let query = format!("--query-gpu={}", PARAMS.join(","));
    let res = String::from_utf8(
        Command::new("nvidia-smi")
            .arg(query)
            .arg("--format=csv,noheader,nounits")
            .output()
            .expect("failed o execute process")
            .stdout,
    )
    .unwrap();
    for res in res.trim().split('\n') {
        let mut map = HashMap::new();
        for (idx, token) in res.split(',').enumerate() {
            let token = token.trim();
            match idx {
                0 => {
                    map.insert("index", token);
                }
                1 => {
                    map.insert("name", token);
                }
                2 => {
                    map.insert("driver_version", token);
                }
                3 => NVIDIA_SMI_GPU_INFO
                    .with_label_values(&[
                        *map.get("index").unwrap(),
                        *map.get("name").unwrap(),
                        *map.get("driver_version").unwrap(),
                        token,
                    ])
                    .set(1_f64),
                4 => NVIDIA_SMI_GPU_TOTAL_MEMORY
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                5 => NVIDIA_SMI_GPU_USED_MEMORY
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                6 => NVIDIA_SMI_GPU_UTILIZATION_MEMORY
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                7 => NVIDIA_SMI_GPU_CUDA_SPECIFICATION
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<f64>().unwrap()),
                8 => NVIDIA_SMI_GPU_TEMPERATURE
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                9 => NVIDIA_SMI_GPU_UTILIZATION
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                10 => NVIDIA_SMI_GPU_FAN_SPEED
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                11 => NVIDIA_SMI_GPU_PERFORMANCE_STATE
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token[1..].parse::<i64>().unwrap()),
                12 => NVIDIA_SMI_GPU_POWER_LIMIT
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<f64>().unwrap()),
                13 => NVIDIA_SMI_GPU_POWER_DRAW
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<f64>().unwrap_or_default()),
                14 => NVIDIA_SMI_GPU_CLOCK_GRAPHICS
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                15 => NVIDIA_SMI_GPU_CLOCK_MEMORY
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                16 => NVIDIA_SMI_GPU_CLOCK_VIDEO
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                17 => NVIDIA_SMI_GPU_CLOCK_MAX_GRAPHICS
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                18 => NVIDIA_SMI_GPU_CLOCK_MAX_MEMORY
                    .with_label_values(&[*map.get("index").unwrap()])
                    .set(token.parse::<i64>().unwrap()),
                _ => unimplemented!(),
            }
        }
    }

    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder
        .encode(&prometheus::gather(), &mut buffer)
        .expect("Failed to encode metrics");

    let response = String::from_utf8(buffer.clone()).expect("Failed to convert bytes to string");
    buffer.clear();
    response
}
