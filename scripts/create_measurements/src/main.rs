use rand::seq::SliceRandom;
use rand_distr::{self, Distribution};
use rayon::{
    self,
    iter::{ParallelBridge, ParallelIterator},
    str::ParallelString,
};
use std::{
    io::{BufWriter, Write}, sync::{atomic::AtomicU64, Arc, RwLock}
};
use better_panic;
use indicatif::{ProgressBar, ProgressStyle};

struct Stations(Arc<RwLock<Vec<WeatherStation>>>);

struct WeatherStation {
    station: String,
    temp: f32,
}

impl Stations {
    fn new() -> Self {
        Stations(Arc::new(RwLock::new(Vec::new())))
    }

    fn add_station(&self, name: String, temp: f32) {
        let mut stations = self.0.write().unwrap();
        stations.push(WeatherStation { station: name, temp });
    }

    fn get_random_station(&self) -> WeatherStation {
        let stations = self.0.read().unwrap();
        stations
            .choose(&mut rand::thread_rng())
            .map(|s| WeatherStation {
                station: s.station.to_string(),
                temp: s.temp,
            })
            .unwrap()
    }
}

impl WeatherStation {
    fn measurement(&self) -> f32 {
        let r = rand_distr::Normal::new(self.temp, 10.0).unwrap();
        (r.sample(&mut rand::thread_rng()) * 10.0).round() / 10.0
    }
}

const WEATHER: &str = include_str!("../../weather_stations.csv");
const BATCH_SIZE: usize = 500_000;

fn predict_file_size(stations: &Stations, count: u128) -> u128 {
    let stations_read = stations.0.read().unwrap();
    let total_name_bytes: usize = stations_read
        .iter()
        .map(|s| s.station.as_bytes().len())
        .sum();
    let avg_name_bytes = total_name_bytes as f64 / stations_read.len() as f64;
    let avg_temp_bytes = 4.400200100050025;
    let avg_line_length = avg_name_bytes + avg_temp_bytes + 2.0;
    (count as f64 * avg_line_length) as u128
}

fn format_bytes(bytes: u128) -> String {
    let mut bytes = bytes as f64;
    let mut unit = String::from("bytes");

    if bytes > 1024.0 {
        bytes /= 1024.0;
        unit = String::from("KiB");
        if bytes > 1024.0 {
            bytes /= 1024.0;
            unit = String::from("MiB");
            if bytes > 1024.0 {
                bytes /= 1024.0;
                unit = String::from("GiB");
            }
        }
    }
    format!("{:.3} {}", bytes, unit)
}

fn main() {
    better_panic::Settings::new()
        .lineno_suffix(true)
        .verbosity(better_panic::Verbosity::Full)
        .install();
    let (tx, rx) = std::sync::mpsc::sync_channel::<String>(100);
    let stations = Stations::new();

    let total_lines = WEATHER.lines().count() as u64;
    let pb_read = ProgressBar::new(total_lines);
    pb_read.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [Reading CSV] [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let read_counter = AtomicU64::new(0);
    WEATHER.par_lines().for_each(|line| {
        pb_read.inc(1);
        if line.starts_with("#") {
            return;
        }
        let mut s = line.split(";");
        stations.add_station(
            s.next().unwrap().to_string(),
            s.next().unwrap().parse().unwrap(),
        );
        read_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    });
    pb_read.finish_with_message("Completed reading stations");
    println!("\nGot all stations!");

    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let count = args
        .next()
        .expect("Expected a parsable number\nUsage: {program} <number>")
        .parse::<u128>()
        .expect("Failed to parse the number");

    println!(
        "Predicted file size: {}",
        format_bytes(predict_file_size(&stations, count))
    );

    let pb_write = ProgressBar::new(count as u64);
    pb_write.set_style(
        ProgressStyle::default_bar()
            .template(&format!("{{spinner:.green}} [Writing Data (batched: {BATCH_SIZE})] [{{elapsed_precise}}] [{{bar:40.yellow/blue}}] {{pos}}/{{len}} lines ({{eta}}, {{per_sec}})"))
            .unwrap()
            .progress_chars("#>-")
    );
    let handle = std::thread::spawn({
        let pb_write = pb_write.clone();
        let batches_before_flushes = 10;
        let mut count = 0;
        move || {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("./measurements.txt")
                .expect("Unable to open file");
            let mut w = BufWriter::with_capacity(1024 * 1024 * 100, file);
            for batch in rx {
                w.write_all(batch.as_bytes()).unwrap();
                count += 1;
                if count % batches_before_flushes == 0 {
                    w.flush().expect("Failed to flush");
                }
                pb_write.inc(batch.lines().count() as u64);
            }
            pb_write.finish_with_message("Finished writing data");
            w.flush().expect("Failed to flush");
        }
    });

    println!("Spawned writing thread!");
    let write_counter = AtomicU64::new(0);
    (0..count)
        .step_by(BATCH_SIZE)
        .par_bridge()
        .for_each_with(tx, |tx, _| {
            let mut batch = Vec::with_capacity(BATCH_SIZE as usize);
            let mut i = 0;
            while i < BATCH_SIZE {
                let x = write_counter.load(std::sync::atomic::Ordering::Relaxed);
                if x as u128 >= count {
                    break;
                }
                let st = stations.get_random_station();
                batch.push(format!("{};{}\n", st.station, st.measurement()));
                write_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                i += 1;
            }
            tx.send(batch.join("")).expect("Failed to send message");
        });
    handle.join().expect("Writing thread mysteriously panicked");
    println!(
        "Success! Written {} lines",
        write_counter.load(std::sync::atomic::Ordering::Relaxed)
    );
}

#[inline]
pub fn truncate(b: f32, precision: usize) -> f32 {
    let factor = 10f32.powi(precision as i32);
    (b * factor).ceil() / factor
}
