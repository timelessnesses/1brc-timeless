use crate::truncate::truncate;
use dashmap::DashMap;
use rayon::prelude::*;
use std::sync::{atomic::AtomicU64, Arc, Mutex};
use std::time::Instant;
#[allow(dead_code)]
fn bench() {
    let f = std::fs::File::open("./measurements.txt").unwrap();
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
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f64 / all as f64) * 100.0,
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
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f64 / all as f64) * 100.0,
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
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f64 / all as f64) * 100.0,
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
        let mut splitted = l.split(";");
        let country = splitted.next().unwrap().to_string();
        let temp = splitted.next().unwrap().parse::<f32>().unwrap();
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);

        let mut ls = last_second.lock().unwrap();

        if count.load(std::sync::atomic::Ordering::Relaxed) % 10000 == 0 {
            println!(
                "Loading File and Parsing File Progress: {}%",
                truncate(
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f64 / all as f64) * 100.0,
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
        let splitted = line.split(";").collect::<Vec<&str>>();
        let country = splitted[0].to_string();
        let temp = splitted[1].parse::<f32>().unwrap();
        hashmap.entry(country).or_insert_with(Vec::new).push(temp);

        let mut ls = last_second.lock().unwrap();

        if ls.elapsed().as_secs() as f32 >= 0.5 {
            println!(
                "Loading File and Parsing File Progress: {}%",
                truncate(
                    (count.load(std::sync::atomic::Ordering::Relaxed) as f64 / all as f64) * 100.0,
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
