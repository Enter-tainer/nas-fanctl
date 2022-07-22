use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Args {
    #[clap(short, long, help("disk paths. e.g. /dev/sda"))]
    pub disks: Vec<String>,
    #[clap(short, long, help("pwm path. e.g. /sys/class/hwmon/hwmon2/pwm1"))]
    pub pwm_path: String,
    #[clap(short, long, help("fan path. e.g. /sys/class/hwmon/hwmon2/fan1_input"))]
    pub fan_path: String,
}
