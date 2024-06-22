use dashmap::{self, DashMap};
use memmap2;
use rayon::prelude::*;
use serde_json;
use std::io::Write;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::Instant;

mod benches;
mod io;
mod truncate;

use crate::truncate::truncate;
// fn verify() {
//     let f = std::fs::File::open("./measurements.txt").unwrap();
//     let mut count: u128 = 0;
//     for line in std::io::BufReader::new(f).lines() {
//         count += 1;
//         if count % 10_000_000 < (count - 1) % 10_000_000 {
//             println!("count: {}", count);
//         }
//     }
//     println!("final: {}", count);
//     assert_eq!(count, 1_000_000_000);
// }

fn main() {
    memmap()
}

/// Actual function (other are just tests)
fn memmap() {
    let start_of_buffering = Instant::now();
    let f = std::fs::File::open("./measurements.txt").unwrap();
    let hashmap: DashMap<String, Vec<f32>> = DashMap::new();
    let memmap_thing = unsafe { memmap2::Mmap::map(&f).unwrap() };
    println!(
        "Took {} milliseconds to prepare read to buffer",
        start_of_buffering.elapsed().as_millis()
    );
    let count = AtomicU64::new(0);

    let last_second = Arc::new(Mutex::new(Instant::now()));
    let parsed = unsafe { std::str::from_utf8_unchecked(&memmap_thing) };
    let start_of_counting = Instant::now();
    let all = parsed.par_lines().count();
    println!(
        "Took {} milliseconds to count lines",
        start_of_counting.elapsed().as_millis()
    );
    let start_of_parsing = Instant::now();
    parsed.par_lines().for_each(|line| {
        // println!("{}", line);
        if line.is_empty() {
            return;
        }
        count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let splitted = line.split(";").collect::<Vec<&str>>();
        if splitted.len() != 2 {
            panic!("Splitted value is NOT 2 values! String: {}", line);
        }
        let country = splitted[0].to_string();
        let temp = splitted[1].parse::<f32>().unwrap();
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);

        if count.load(std::sync::atomic::Ordering::Relaxed) % 10000 == 0 {
            let mut ls = last_second.lock().unwrap();
            if ls.elapsed().as_secs() as f32 >= 0.5 {
                println!(
                    "Loading File and Parsing File Progress: {}%",
                    truncate(
                        (count.load(std::sync::atomic::Ordering::Relaxed) as f32 / all as f32)
                            * 100.0,
                        2
                    )
                );
                *ls = Instant::now(); // Update the last_second to the current time
            }
        }
    });
    println!(
        "Took {} milliseconds for parsing",
        start_of_parsing.elapsed().as_millis()
    );
    let hashmap2: DashMap<String, (f32, f32, f32)> = DashMap::new();
    let start_conversion = Instant::now();
    hashmap.par_iter_mut().for_each(|mut thing| {
        let pair = thing.pair_mut();
        let key = pair.0;
        let val = pair.1;
        let mean = truncate(val.iter().sum::<f32>() / val.len() as f32, 1);
        val.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min = val[0];
        let max = val[val.len() - 1];
        hashmap2.insert(key.to_owned(), (min, mean, max));
    });
    println!(
        "Took {} milliseconds for conversion technology",
        start_conversion.elapsed().as_millis()
    );
    std::fs::File::create_new("./out.json")
        .unwrap_or(std::fs::File::open("./out.json").expect("Failed to open ./out.json"))
        .write(serde_json::to_string_pretty(&hashmap2).unwrap().as_bytes())
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
