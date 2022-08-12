mod payloads;

use lazy_static::lazy_static;
use payloads::{Control, Monitor};
use prometheus::{Encoder, Gauge, GaugeVec, Opts, Registry, TextEncoder};
use serde_json::{Deserializer, Value};
use tokio_serial::SerialPort;
use tracing::{debug, info, trace};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use warp::{Filter, Rejection, Reply};
//  use tokio_stream::{self as stream, StreamExt};

// Prometheus Setup
lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref BOARD_TEMP: Gauge =
        Gauge::new("board_temp", "FEM Board Temperature (C)").expect("metric can be created");
    pub static ref IF_POW: GaugeVec = GaugeVec::new(
        Opts::new("if_power", "IF Output Power (dBm) for FEM Channels"),
        &["ch"]
    )
    .expect("metric can be created");
    pub static ref VOLTAGES: GaugeVec =
        GaugeVec::new(Opts::new("voltages", "FEM Rail Voltages"), &["rail"])
            .expect("metric can be created");
    pub static ref CURRENTS: GaugeVec =
        GaugeVec::new(Opts::new("currents", "FEM Rail Currents"), &["rail"])
            .expect("metric can be created");
}

fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(IF_POW.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(BOARD_TEMP.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(VOLTAGES.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(CURRENTS.clone()))
        .expect("collector can be registered");
}

// Prometheus endpoint
async fn metrics_handler() -> Result<impl Reply, Rejection> {
    trace!("New metrics query");
    let encoder = TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        eprintln!("could not encode custom metrics: {}", e);
    };
    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    Ok(res)
}

async fn monitor(mut port: Box<dyn SerialPort>) {
    loop {
        // Somehow switch this to the tokio streams?
        let stream = Deserializer::from_reader(&mut port).into_iter::<Value>();
        for value in stream {
            match value {
                Ok(v) => {
                    let monitor: Monitor = match serde_json::from_value(v) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("{e}");
                            continue;
                        }
                    };
                    trace!("New monitor data - {:#?}", monitor);
                    // Send to prometheus
                    BOARD_TEMP.set(monitor.board_temp as f64);
                    // IF Powers
                    IF_POW
                        .with_label_values(&["ch1"])
                        .set(monitor.if_power.channel_one as f64);
                    IF_POW
                        .with_label_values(&["ch2"])
                        .set(monitor.if_power.channel_two as f64);
                    // Voltages
                    VOLTAGES
                        .with_label_values(&["input"])
                        .set(monitor.voltages.raw_input as f64);
                    VOLTAGES
                        .with_label_values(&["analog"])
                        .set(monitor.voltages.analog as f64);
                    VOLTAGES
                        .with_label_values(&["ch1"])
                        .set(monitor.voltages.lna_one as f64);
                    VOLTAGES
                        .with_label_values(&["ch2"])
                        .set(monitor.voltages.lna_two as f64);
                    // Currents
                    CURRENTS
                        .with_label_values(&["input"])
                        .set(monitor.currents.raw_input as f64);
                    CURRENTS
                        .with_label_values(&["analog"])
                        .set(monitor.currents.analog as f64);
                    CURRENTS
                        .with_label_values(&["ch1"])
                        .set(monitor.currents.lna_one as f64);
                    CURRENTS
                        .with_label_values(&["ch2"])
                        .set(monitor.currents.lna_two as f64);
                }
                Err(_) => (),
            }
        }
    }
}

// Grab serial port from clap
// Set prometheus port from clap
// Incoming control payloads??

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var or default to info.
    let filter_layer = EnvFilter::try_from_default_env().unwrap();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter_layer)
        .init();
    // Open serial port
    let mut port = tokio_serial::new("/dev/ttyUSB0", 115200)
        .open()
        .expect("Couldn't open serial port");
    debug!("Logging started!");
    // Send a single control payload
    let control = Control {
        cal_one: false,
        cal_two: false,
        lna_one_powered: true,
        lna_two_powered: true,
        attenuation_level: 3,
        if_power_threshold: -10f32,
    };
    port.write_all(serde_json::to_string(&control).unwrap().as_bytes())
        .unwrap();
    debug!("Sent initial control payload");
    // Setup prometheus registry
    register_custom_metrics();
    // Webserver setup
    let metrics_route = warp::path!("metrics").and_then(metrics_handler);
    // Startup
    tokio::task::spawn(monitor(port));
    warp::serve(metrics_route).run(([0, 0, 0, 0], 8080)).await;
}
