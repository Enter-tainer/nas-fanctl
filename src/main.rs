use std::{
    cmp::Ord,
    fs::{read_to_string, File},
    io::Write,
    marker::Copy,
    process::{self, Command},
    thread::sleep,
    time::Duration,
};

use clap::Parser;
use cli::Args;
use interpolation::Interpolator;
use itertools::Itertools;
use once_cell::sync::OnceCell;
use regex::Regex;

mod cli;
mod interpolation;
static DISKS: OnceCell<Vec<String>> = OnceCell::new();
fn get_temp() -> Vec<i32> {
    let output = String::from_utf8_lossy(
        &Command::new("hddtemp")
            .args(DISKS.get().unwrap())
            .output()
            .expect("read hdd temp failed")
            .stdout,
    )
    .to_string();
    let match_regex = Regex::new("(\\d+)°C").unwrap();
    output
        .lines()
        .map(|i| {
            let caps = match_regex.captures(i).unwrap();
            let temp = caps.get(1).unwrap().as_str();
            temp.parse::<i32>().unwrap()
        })
        .collect()
}

fn get_pwm_enable(path: &str) -> String {
    let pwm_enable_path = format!("{}_enable", path);
    let pwm_enable = read_to_string(&pwm_enable_path).unwrap();
    println!("{}: {}", pwm_enable_path, pwm_enable);
    pwm_enable
}

fn set_pwm_enable(path: &str, value: &str) {
    let pwm_enable_path = format!("{}_enable", path);
    let mut pwm_enable = File::options()
        .write(true)
        .read(false)
        .open(&pwm_enable_path)
        .unwrap();
    pwm_enable.write_all(value.as_bytes()).unwrap();
    pwm_enable.flush().unwrap();
    println!("set {} to {}", pwm_enable_path, value);
}

fn set_pwm_to_manual(path: &str) {
    set_pwm_enable(path, "1");
}

fn get_pwm_value_by_temp(pwm_to_speed: &Interpolator, temp: i32) -> (i32, i32) {
    // 温度 30 以下，pwm = 0, 60 以上 pwm = 255
    // 中间部分线性
    let temp_min = 30;
    let temp_max = 60;
    let pwm_min = 0;
    let pwm_max = 255;
    if temp <= temp_min {
        return (pwm_min, 1000);
    }
    if temp >= temp_max {
        return (pwm_max, 4800);
    }
    let work = (temp - temp_min) as f64 / (temp_max - temp_min) as f64;
    let speed = (3800.0 * work + 1000.0).clamp(1000.0, 4800.0);
    let pwm = pwm_to_speed.estimate_x(speed).clamp(0.0, 255.0);
    (pwm as i32, speed as i32)
}

fn get_fan_speed(path: &str) -> i32 {
    let fan_speed = read_to_string(&path).unwrap();
    fan_speed.trim().parse().unwrap()
}

fn set_pwm(path: &str, value: i32) {
    let pwm_path = path;
    let mut pwm = File::options()
        .write(true)
        .read(false)
        .open(&pwm_path)
        .unwrap();
    pwm.write_all(value.to_string().as_bytes()).unwrap();
    pwm.flush().unwrap();
    println!("set {} to {}", pwm_path, value);
}

fn main() {
    let pwm_to_speed = Interpolator::with_points(
        [
            (255, 4945),
            (240, 4770),
            (225, 4485),
            (210, 4192),
            (195, 3890),
            (180, 3600),
            (165, 3308),
            (150, 2941),
            (135, 2621),
            (120, 2246),
            (105, 1862),
            (90, 1430),
            (75, 1055),
            (0, 1039),
        ]
        .into_iter()
        .map(|(x, y)| (x as f64, y as f64))
        .collect_vec(),
    );
    println!("temp\t(pwm, speed)");
    for i in 30..60 {
        if i % 2 == 0 {
            println!("{}\t{:?}", i, get_pwm_value_by_temp(&pwm_to_speed, i));
        }
    }
    let Args {
        pwm_path,
        disks,
        fan_path,
        interval,
    } = cli::Args::parse();
    DISKS.get_or_init(move || disks);
    let pwm_enable = get_pwm_enable(&pwm_path);
    set_pwm_to_manual(&pwm_path);
    {
        let pwm_path = pwm_path.clone();
        ctrlc::set_handler(move || {
            set_pwm(&pwm_path, 150);
            set_pwm_enable(&pwm_path, &pwm_enable);
            process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }
    loop {
        println!("current fan speed: {}RPM", get_fan_speed(&fan_path));
        let temp = get_temp();
        println!("current dev temp: {:?}", temp);
        let max_temp = temp.iter().max().unwrap();
        let (pwm_value, estimated_speed) = get_pwm_value_by_temp(&pwm_to_speed, *max_temp);
        println!(
            "set pwm to: {}/255, estimated fan speed: {}RPM",
            pwm_value, estimated_speed
        );
        set_pwm(&pwm_path, pwm_value);
        println!("{}", "=".repeat(40));
        sleep(Duration::from_secs(interval.try_into().unwrap()));
    }
}
