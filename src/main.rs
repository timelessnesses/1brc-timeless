use dashmap::DashMap;
use memmap2;
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use serde_json;
use std::io::Write;
use std::time::Instant;

mod benches;
mod io;
mod truncate;

use crate::truncate::truncate;

fn main() {
    memmap()
}

const SHARD_AMOUNT: usize = 2048;

#[derive(serde::Serialize, Debug, Default)]
struct Station {
    #[serde(skip_serializing)]
    sum: f32,
    #[serde(skip_serializing)]
    len: usize,

    min: f32,
    max: f32,
    avg: f32,
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
    rayon::ThreadPoolBuilder::new().num_threads(18).use_current_thread().thread_name(|i| format!("1brc-timeless::memmap::processing_{i}")).build_global().expect("Failed to build threadpool");
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
    let start_of_parsing = Instant::now();
    parsed.par_lines().for_each(|line| {
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
        let st = a.value_mut();
        st.len += 1;
        if temp > st.max {
            st.max = temp;
        } else if temp < st.min {
            st.min = temp;
        }
        st.sum += temp;
        st.avg = truncate(st.sum / st.len as f32, 1)
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
