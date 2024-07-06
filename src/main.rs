use dashmap::DashMap;
use memmap2;
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use serde_json;
use std::io::Write;
use std::sync::atomic::AtomicU64;
use std::time::Instant;

mod benches;
mod io;
mod truncate;

use crate::truncate::truncate;

fn main() {
    memmap()
}

const SHARD_AMOUNT: usize = 2048;

#[derive(serde::Serialize, Debug)]
struct Station {
    #[serde(skip_serializing)]
    sum: f32,
    #[serde(skip_serializing)]
    len: usize,

    min: f32,
    max: f32,
    avg: f32,
}

impl Default for Station {
    fn default() -> Self {
        Self {
            sum: 0.0,
            len: 0,
            min: 0.0,
            max: 0.0,
            avg: 0.0,
        }
    }
}

fn print_stat() {
    #[allow(non_upper_case_globals)]
    static shard_count_thing: OnceCell<usize> = OnceCell::new();
    println!(
        "Default shard count is actually {} shards but we're using {} shards instead",
        *shard_count_thing.get_or_init(|| {
            (std::thread::available_parallelism().map_or(0, usize::from) * 4).next_power_of_two()
        }),
        SHARD_AMOUNT
    );
}

/// Actual function (other are just tests)
fn memmap() {
    print_stat();
    let start_of_buffering = Instant::now();
    let f = std::fs::File::open("./measurements.txt").unwrap();
    let hashmap: DashMap<String, Station> = DashMap::with_shard_amount(SHARD_AMOUNT);
    let memmap_thing = unsafe {
        memmap2::MmapOptions::new()
            .stack()
            .populate()
            .huge(None)
            .map_copy_read_only(&f)
            .unwrap()
    };
    println!(
        "Took {} milliseconds to prepare read to buffer",
        start_of_buffering.elapsed().as_millis()
    );

    let parsed = unsafe { std::str::from_utf8_unchecked(&memmap_thing) };
    #[cfg(debug_assertions)]
    // let all = parsed.par_lines().count();
    #[cfg(debug_assertions)]
    let count = AtomicU64::new(0);
    let start_of_parsing = Instant::now();
    parsed.par_lines().for_each(|line| {
        #[cfg(debug_assertions)]
        {
            count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        if line == "" {
            return;
        }
        let mut splitted = line.split(";");
        let country = splitted.next().expect("Country not found").to_string();
        let temp = splitted
            .next()
            .expect("Temp not found")
            .parse::<f32>()
            .unwrap();
        let mut a = hashmap.entry(country).or_insert(Station::default());
        let st = a.pair_mut().1;
        st.len += 1;
        if temp > st.max {
            st.max = temp;
        } else if temp < st.min {
            st.min = temp;
        }
        st.sum += temp;
        /* #[cfg(debug_assertions)]
        {
            let c = count.load(std::sync::atomic::Ordering::Relaxed);
            if c % 1_000_000 == 0 {
                println!("Progress: {}% ({c}/{all})", truncate(c as f64 / all as f64 , 2));
            }
        } */
    });
    hashmap.par_iter_mut().for_each(|mut t| {
        let st = t.value_mut();
        st.avg = truncate(st.sum as f64 / st.len as f64, 1) as f32
    });
    println!(
        "Took {} milliseconds for parsing and processing",
        start_of_parsing.elapsed().as_millis()
    );
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("./out.json")
        .unwrap()
        .write_all(serde_json::to_string_pretty(&hashmap).unwrap().as_bytes())
        .unwrap();
    #[cfg(debug_assertions)]
    {
        for thing in hashmap.iter() {
            println!(
                "Station: {}, Temperature (Min/Mean/Max): {}/{}/{}",
                thing.key(),
                thing.value().min,
                thing.value().avg,
                thing.value().max
            );
        }
        println!("{:#?}", hashmap.get("Alexandria").unwrap().value());
    }
}
