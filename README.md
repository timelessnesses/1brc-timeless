# 1brc-timeless

This repository contains rust code that can be used with 1brc (One Billion Row Challenge) dataset. *how the fuck did java #1 did it in 1 seconds*

## Running

Get yourself a `measurements.txt` and just run `cargo run -r`. It will spit out progress on reading file and time it take to parse file and do computation and a `out.json` file

## Getting yourself a `measurements.txt`

Inside `scripts/` folders contains 2 versions of `create_measurements` scripts that you can use. Python implementation is stolen from [`1brc`](https://github.com/gunnarmorling/1brc) repository and there is Rust's port of the Python implementation with multi-threading for fast speeds.  
Executing them is simply just

```sh
python scripts/create_measurements.py 1000000000
# or
cd scripts/create_measurements
cargo run -r -- 1000000000
```

## Perf

ran on my laptop (AMD Ryzen 7 7730U with 16 Gigabytes of RAM with 8 Cores and 16 Logical Processors) on Windows
| stuff                	| my implementation in rust 	| #1 java 1brc                 	|
|----------------------	|---------------------------	|------------------------------	|
| RAM Usage (Peak)     	| 7 Megabytes            	| 80+ Megabytes         	|
| Time to process      	| 32 seconds (pretty close)          	| 56 seconds                   	|
| Peak Disk Activities 	| 460+ MB/s                 	| 280+ MB/s 	|
| Avg Disk Activites    | 230-420+ MB/S                            | 230-280+ MB/S

![](./perf.png)

## Special Thanks

- [0/0](https://github.com/tazz4843)
- acrimon (discord)
- erk (discord)
- and everyone else in Serenity Discord Bot Library Guild!
