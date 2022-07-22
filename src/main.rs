use std::{
    fs::{read_to_string, File},
    io::Write,
    process::{Command, self},
    thread::sleep,
    time::Duration,
};

use clap::Parser;
use cli::Args;
use once_cell::sync::OnceCell;
use regex::Regex;

mod cli;
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

fn get_fan_pwm_by_workload(work: f32) -> (i32, i32) {
    // let fan_pwm = [
    //     (255, 4945),
    //     (240, 4770),
    //     (225, 4485),
    //     (210, 4192),
    //     (195, 3890),
    //     (180, 3600),
    //     (165, 3308),
    //     (150, 2941),
    //     (135, 2621),
    //     (120, 2246),
    //     (105, 1862),
    //     (90, 1430),
    //     (75, 1055),
    //     (60, 1045),
    //     (45, 1041),
    //     (30, 1036),
    //     (28, 1042),
    //     (26, 1044),
    //     (24, 1043),
    //     (22, 1039),
    //     (20, 1038),
    //     (18, 1043),
    //     (16, 1043),
    //     (14, 1042),
    //     (12, 1040),
    //     (10, 1039),
    //     (8, 1044),
    //     (6, 1041),
    //     (4, 1044),
    //     (2, 1039),
    //     (0, 1039),
    // ];
    // pwm 范围 0-255
    // 风扇 75-255 是线性增长，转速对应 1000-5000
    // 1000 -> 0%
    // 5000 -> 100%
    let pwm_min = 75;
    let pwm_max = 255;
    let fan_min = 1000;
    let fan_max = 5000;
    let target_fan = (((fan_max - fan_min) as f32) * work) as i32 + fan_min;
    let target_pwm = (((pwm_max - pwm_min) as f32) * work) as i32 + pwm_min;
    (target_pwm, target_fan)
}

fn get_pwm_value_by_temp(temp: i32) -> i32 {
    // 温度 20 以下，pwm = 0, 50 以上 pwm = 255
    // 中间部分线性
    let temp_min = 20;
    let temp_max = 60;
    let pwm_min = 0;
    let pwm_max = 200;
    if temp <= temp_min {
        return pwm_min;
    }
    if temp >= temp_max {
        return pwm_max;
    }
    let work = (temp - temp_min) as f32 / temp_max as f32;
    let (pwm, speed) = get_fan_pwm_by_workload(work);
    println!(
        "set workload to {}, pwm: {}/255, expected fan speed: {}RPM",
        work, pwm, speed
    );
    pwm
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
    let Args {
        pwm_path,
        disks,
        fan_path,
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
        let pwm_value = get_pwm_value_by_temp(*max_temp);
        set_pwm(&pwm_path, pwm_value);
        println!("{}", "=".repeat(40));
        sleep(Duration::from_secs(10));
    }
}
