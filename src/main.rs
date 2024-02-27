use core::time;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

use clap::Parser;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

use color_eyre::eyre::Result;
use rand::{seq::SliceRandom, Rng};

/// Generates a large number of rows for the one billion row challenge
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of rows to generate
    #[arg(short, long, default_value_t = 1_000_000_000)]
    rows: u64,

    /// Path to the weather station examples
    #[arg(short, long, default_value_t = String::from("./data/weather_stations.csv"))]
    weather_stations: String,

    /// Path to the file to generate
    #[arg(short, long, default_value_t = String::from("./data/measurements.txt"))]
    output: String,
}

#[derive(Debug)]
struct WeatherStation {
    id: String,
}
impl TryFrom<&str> for WeatherStation {
    type Error = color_eyre::eyre::ErrReport;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut split = value.split(';');
        let id = split
            .next()
            .ok_or_else(|| color_eyre::eyre::eyre!("No id"))?
            .to_string();
        Ok(Self { id })
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let stations: Vec<WeatherStation> = load_weather_stations(args.weather_stations)?;
    generate_lines(&stations, args.rows, args.output)?;

    Ok(())
}

const MIN_TEMP: i32 = -999; // -99.9C
const MAX_TEMP: i32 = 999; // 99.9C
const CHUNK_SIZE: u64 = 10_000;

macro_rules! generate_line {
    ($stations:expr, $out_buf:expr) => {{
        let station = $stations
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| color_eyre::eyre::eyre!("No stations"))?;
        let measurement = rand::thread_rng().gen_range(MIN_TEMP..MAX_TEMP);
        let line = format!(
            "{};{}.{}\n",
            station.id,
            measurement / 10,
            if measurement < 0 {
                measurement * -1 % 10
            } else {
                measurement % 10
            }
        );
        $out_buf.push_str(&line);
    }};
}

fn generate_lines(stations: &Vec<WeatherStation>, rows: u64, output_path: String) -> Result<()> {
    let bar_style = ProgressStyle::with_template(
        "[{elapsed_precise} elapsed] [{eta_precise} remaining] [{percent:.2}%] {msg}\n{bar:80.cyan/blue} ",
    )
    .expect("Could not create progress bar style");
    let chunk_count = rows / CHUNK_SIZE;
    let bar = ProgressBar::new(chunk_count + 1).with_style(bar_style);
    bar.enable_steady_tick(time::Duration::from_millis(1000));
    let mut file = File::create(output_path)?;
    let mut out_buf;
    for _ in 0..chunk_count {
        out_buf = String::new();
        for _ in 0..CHUNK_SIZE {
            generate_line!(&stations, &mut out_buf);
        }
        file.write_all(out_buf.as_bytes())?;
        bar.inc(1);
    }

    // Extra chunk with remainder rows
    out_buf = String::new();
    for _ in 0..rows % CHUNK_SIZE {
        generate_line!(&stations, &mut out_buf);
    }

    file.write_all(out_buf.as_bytes())?;
    bar.inc(1);

    let size = file.metadata()?.len();
    bar.finish_with_message(format!(
        "Completed, final file size: {}",
        human_readable(size)
    ));

    Ok(())
}

const BYTE_POSTFIXES: [&str; 9] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
fn human_readable(value: u64) -> String {
    let mut value = value as f64;
    let mut i = 0;
    while value > 1024.0 && i < BYTE_POSTFIXES.len() {
        value /= 1024.0;
        i += 1;
    }

    format!("{:.2} {}", value, BYTE_POSTFIXES[i])
}

fn load_weather_stations(path: String) -> Result<Vec<WeatherStation>> {
    let file: File = load_weather_stations_file(path)?;
    let reader: BufReader<File> = BufReader::new(file);
    let mut stations = Vec::new();
    for line_result in reader.lines() {
        let line = line_result?;
        if line.starts_with('#') {
            continue;
        }
        stations.push(WeatherStation::try_from(line.as_str())?);
    }
    Ok(stations)
}

fn load_weather_stations_file(path: String) -> Result<File> {
    File::open(path).map_err(|_| color_eyre::eyre::eyre!("Could not open file"))
}
