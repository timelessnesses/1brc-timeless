use crate::truncate::truncate;
use dashmap::{self, DashMap};
use rayon::prelude::*;
use std::time::Instant;
use std::{self, io::BufRead};

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
