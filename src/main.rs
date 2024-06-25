#![feature(portable_simd)]

use dashmap::DashMap;
use memmap2;
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use serde_json;
use std::io::Write;
use std::simd::f32x4;
use std::simd::num::SimdFloat;
use std::sync::atomic::AtomicU64;
use std::time::Instant;
use voracious_radix_sort::RadixSort;

mod benches;
mod io;
mod truncate;

use crate::truncate::truncate;

fn main() {
    memmap()
}

const SHARD_AMOUNT: usize = 1024;

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
    rayon::ThreadPoolBuilder::new()
        .num_threads(18)
        .build_global()
        .expect("Failed");
    let start_of_buffering = Instant::now();
    let f = std::fs::File::open("./measurements.txt").unwrap();
    let hashmap: DashMap<String, Vec<f32>> = DashMap::with_shard_amount(SHARD_AMOUNT);
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
    let all = parsed.par_lines().count();
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
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);
        #[cfg(debug_assertions)]
        {
            let x = count.load(std::sync::atomic::Ordering::Relaxed);
            if x % 1_000_000 == 0 {
                print!("Parsing: {}%", truncate((x as f32 / all as f32) * 100.0, 2));
            }
        }
    });
    println!(
        "Took {} milliseconds for parsing",
        start_of_parsing.elapsed().as_millis()
    );
    let hashmap2: DashMap<String, (f32, f32, f32)> = DashMap::with_shard_amount(SHARD_AMOUNT);
    let start_conversion = Instant::now();
    hashmap.par_iter_mut().for_each(|mut thing| {
        let pair = thing.pair_mut();
        let key = pair.0;
        let val = pair.1;

        let simd_fuckery = f32x4::from_slice(&val);
        let mean = simd_fuckery.reduce_sum() / simd_fuckery.len() as f32;

        // Sort using the standard sort method
        val.voracious_mt_sort(
            std::thread::available_parallelism()
                .map_or(0, usize::from)
                .next_power_of_two(),
        );
        let min = val[0];
        let max = val[val.len() - 1];
        hashmap2.insert(key.to_owned(), (min, truncate(mean, 1), max));
    });
    println!(
        "Took {} milliseconds for conversion technology",
        start_conversion.elapsed().as_millis()
    );
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("./out.json")
        .unwrap()
        .write_all(serde_json::to_string_pretty(&hashmap2).unwrap().as_bytes())
        .unwrap();
    #[cfg(debug_assertions)]
    {
        for thing in hashmap2.iter() {
            println!(
                "Station: {}, Temperature (Min/Mean/Max): {}/{}/{}",
                thing.key(),
                thing.value().0,
                thing.value().1,
                thing.value().2
            );
        }
        println!("{:#?}", hashmap2.get("Alexandria").unwrap().value());
        assert_eq!(
            &(-27.4 as f32, 20.0 as f32, 68.4 as f32),
            hashmap2.get("Alexandria").unwrap().value()
        );
    }
}
