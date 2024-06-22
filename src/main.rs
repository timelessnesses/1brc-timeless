use dashmap::{self, DashMap};
use memmap2;
use rayon::prelude::*;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{self, io::BufRead};

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

#[allow(dead_code)]
fn bench() {
    let f = std::fs::File::open("./measurements 10mil.txt").unwrap();
    let hashmap: DashMap<String, Vec<f32>> = DashMap::new();
    let memmap_thing = unsafe { memmap2::Mmap::map(&f).unwrap() };
    let count = AtomicU64::new(0);
    let last_second = Arc::new(Mutex::new(Instant::now()));
    let parsed = unsafe { std::str::from_utf8_unchecked(&memmap_thing) };
    let all = parsed.par_lines().count();
    let start_of_parsing = Instant::now();
    parsed.split("\n").par_bridge().for_each(|line| {
        // println!("{}", line);
        if line.is_empty() {
            return;
        }
        count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let l = line.to_owned();
        let splitted = l.split(";").collect::<Vec<&str>>();
        let country = splitted[0].to_string();
        let temp = splitted[1].parse::<f32>().unwrap();
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);

        let mut ls = last_second.lock().unwrap();

        if ls.elapsed().as_secs() as f32 >= 0.5 {
            println!(
                "Loading File and Parsing File Progress: {}%",
                truncate(
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f32 / all as f32) * 100.0,
                    2
                )
            );
            *ls = Instant::now(); // Update the last_second to the current time
        }
    });
    println!(
        "Took {} milliseconds for parsing (split newlines, rayon'd)",
        start_of_parsing.elapsed().as_millis()
    );
    let start_of_parsing = Instant::now();
    let count = AtomicU64::new(0);

    parsed.split("\n").for_each(|line| {
        // println!("{}", line);
        if line.is_empty() {
            return;
        }
        count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let l = line.to_owned();
        let splitted = l.split(";").collect::<Vec<&str>>();
        let country = splitted[0].to_string();
        let temp = splitted[1].parse::<f32>().unwrap();
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);

        let mut ls = last_second.lock().unwrap();

        if ls.elapsed().as_secs() as f32 >= 0.5 {
            println!(
                "Loading File and Parsing File Progress: {}%",
                truncate(
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f32 / all as f32) * 100.0,
                    2
                )
            );
            *ls = Instant::now(); // Update the last_second to the current time
        }
    });
    println!(
        "Took {} milliseconds for parsing (split newlines, sequential'd)",
        start_of_parsing.elapsed().as_millis()
    );
    let start_of_parsing = Instant::now();
    let count = AtomicU64::new(0);
    parsed.lines().par_bridge().for_each(|line| {
        // println!("{}", line);
        if line.is_empty() {
            return;
        }
        count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let l = line.to_owned();
        let splitted = l.split(";").collect::<Vec<&str>>();
        let country = splitted[0].to_string();
        let temp = splitted[1].parse::<f32>().unwrap();
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);

        let mut ls = last_second.lock().unwrap();

        if ls.elapsed().as_secs() as f32 >= 0.5 {
            println!(
                "Loading File and Parsing File Progress: {}%",
                truncate(
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f32 / all as f32) * 100.0,
                    2
                )
            );
            *ls = Instant::now(); // Update the last_second to the current time
        }
    });
    println!(
        "Took {} milliseconds for parsing (line iterator, rayon'd)",
        start_of_parsing.elapsed().as_millis()
    );
    let start_of_parsing = Instant::now();
    let count = AtomicU64::new(0);
    parsed.lines().for_each(|line| {
        // println!("{}", line);
        if line.is_empty() {
            return;
        }
        count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let l = line.to_owned();
        let splitted = l.split(";").collect::<Vec<&str>>();
        let country = splitted[0].to_string();
        let temp = splitted[1].parse::<f32>().unwrap();
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);

        let mut ls = last_second.lock().unwrap();

        if ls.elapsed().as_secs() as f32 >= 0.5 {
            println!(
                "Loading File and Parsing File Progress: {}%",
                truncate(
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f32 / all as f32) * 100.0,
                    2
                )
            );
            *ls = Instant::now(); // Update the last_second to the current time
        }
    });
    println!(
        "Took {} milliseconds for parsing (line iterator, sequential'd)",
        start_of_parsing.elapsed().as_millis()
    );
    let start_of_parsing = Instant::now();
    let count = AtomicU64::new(0);
    parsed.par_lines().for_each(|line| {
        // println!("{}", line);
        if line.is_empty() {
            return;
        }
        count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let l = line.to_owned();
        let splitted = l.split(";").collect::<Vec<&str>>();
        let country = splitted[0].to_string();
        let temp = splitted[1].parse::<f32>().unwrap();
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);

        let mut ls = last_second.lock().unwrap();

        if ls.elapsed().as_secs() as f32 >= 0.5 {
            println!(
                "Loading File and Parsing File Progress: {}%",
                truncate(
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f32 / all as f32) * 100.0,
                    2
                )
            );
            *ls = Instant::now(); // Update the last_second to the current time
        }
    });
    println!(
        "Took {} milliseconds for parsing (rayon included loop through lines, rayon'd)",
        start_of_parsing.elapsed().as_millis()
    );
}

/// Actual function (other are just tests)
fn memmap() {
    let start_of_buffering = Instant::now();
    let f = std::fs::File::open("./measurements 100mil.txt").unwrap();
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
        let l = line.to_owned();
        let splitted = l.split(";").collect::<Vec<&str>>();
        let country = splitted[0].to_string();
        let temp = splitted[1].parse::<f32>().unwrap();
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);

        let mut ls = last_second.lock().unwrap();

        if ls.elapsed().as_secs() as f32 >= 0.5 {
            println!(
                "Loading File and Parsing File Progress: {}%",
                truncate(
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f32 / all as f32) * 100.0,
                    2
                )
            );
            *ls = Instant::now(); // Update the last_second to the current time
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

#[allow(dead_code)]
fn io() {
    let f = std::fs::File::open("./measurements 100mil.txt").unwrap();
    let hashmap: DashMap<String, Vec<f32>> = DashMap::new();
    let start_of_parsing = Instant::now();
    std::io::BufReader::with_capacity(512000, f)
        .lines()
        .for_each(|line| {
            let l = line.unwrap().to_owned();
            let splitted = l.split(";").collect::<Vec<&str>>();
            let country = splitted[0].to_string();
            let temp = splitted[1].parse::<f32>().unwrap();
            if country == "Alexandria" && country.contains("-") {
                println!("{} {}", temp, splitted[1]);
            }
            hashmap.entry(country).or_insert_with(Vec::new).push(temp);
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
    for thing in hashmap2.iter() {
        println!(
            "Station: {}, Temperature (Min/Mean/Max): {}/{}/{}",
            thing.key(),
            thing.value().0,
            thing.value().1,
            thing.value().2
        );
    }
    #[cfg(debug_assertions)]
    {
        println!("{:#?}", hashmap2.get("Alexandria").unwrap().value());
        assert_eq!(
            &(-27.4 as f32, 20.0 as f32, 68.4 as f32),
            hashmap2.get("Alexandria").unwrap().value()
        );
    }
}

/// truncating float thanks chatgpt
fn truncate(b: f32, precision: usize) -> f32 {
    let factor = 10f32.powi(precision as i32);
    (b * factor).ceil() / factor
}
