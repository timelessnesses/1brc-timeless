use rand::{seq::SliceRandom, Rng};
use rand_distr::{self, Distribution};
use rayon::{
    self,
    iter::{IntoParallelIterator, ParallelBridge, ParallelIterator},
    str::ParallelString,
};
use std::{
    io::{BufWriter, Write},
    sync::{atomic::AtomicU64, mpsc::sync_channel, Arc, RwLock},
};

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
        stations.push(WeatherStation {
            station: name,
            temp,
        });
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
        return (r.sample(&mut rand::thread_rng()) * 10.0).round() / 10.0;
    }
}

fn main() {
    let (tx, rx) = sync_channel::<Vec<String>>(100);
    let stations = Stations::new();

    let WEATHER = include_str!("../../weather_stations.csv");
    let c = AtomicU64::new(0);
    let all = WEATHER.par_lines().count();
    WEATHER.par_lines().for_each(|line| {
        if line.starts_with("#") {
            return;
        }
        let mut s = line.split(";");
        stations.add_station(
            s.next().unwrap().to_string(),
            s.next().unwrap().parse().unwrap(),
        );
        c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let x = c.load(std::sync::atomic::Ordering::Relaxed);
        if x % 1000 == 0 {
            println!(
                "Progress: {}% ({}/{})",
                truncate((x as f32 / all as f32) * 100 as f32, 2),
                x,
                all
            )
        }
    });
    println!("Got all stations!");
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let count = args
        .next()
        .expect("Expected a parsable number\nUsage: {program} <number>")
        .parse::<u128>()
        .expect("Failed to parse the number");
    let handle = std::thread::spawn(move || {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("./measurements.txt")
            .expect("Unable to open file");
        let mut w = BufWriter::with_capacity(1024 * 1024 * 30, file);
        for batch in rx {
            for r in batch {
                w.write_all(r.as_bytes())
                    .expect("Failed to write this buffer");
                drop(r)
            }
            w.flush().expect("Failed to flush");
        }
    });
    println!("Spawned writing thread!");
    let c = AtomicU64::new(0);
    let BATCH_SIZE = 500000;
    (0..count)
        .step_by(BATCH_SIZE)
        .par_bridge()
        .for_each_with(tx, |tx, _| {
            let mut batch = Vec::with_capacity(BATCH_SIZE as usize);
            let mut i = 0;
            while i < BATCH_SIZE {
                let x = c.load(std::sync::atomic::Ordering::Relaxed);
                if x as u128 >= count {
                    println!("Reached required count, stopping");
                    break;
                }
                let st = stations.get_random_station();
                batch.push(format!("{};{}\n", st.station, st.measurement()));
                drop(st);
                c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                i += 1;
                if x % 1_000_000 == 0 {
                    println!(
                        "Progress: {}% ({}/{})",
                        truncate((x as f32 / count as f32) * 100 as f32, 2),
                        x,
                        count
                    );
                }
            }
            tx.send(batch).expect("Failed to send message");
        });
    handle.join().expect("Writing thread mysteriously panicked");
    println!(
        "Success! Written {} lines",
        c.load(std::sync::atomic::Ordering::Relaxed)
    );
}

#[inline]
pub fn truncate(b: f32, precision: usize) -> f32 {
    let factor = 10f32.powi(precision as i32);
    (b * factor).ceil() / factor
}
