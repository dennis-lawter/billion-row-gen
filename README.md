# Rust One Billion Row Challenge Generator
Generates one billion rows of data for the 1BRC

[https://github.com/gunnarmorling/1brc](https://github.com/gunnarmorling/1brc)

## Installation

```shell
cargo install billion-row-gen
```

## Usage

Defaults:
- The row count defaults to 1,000,000,000.
- The tool will use the `./data/weather_stations.csv` file to generate
the weather station names.
- The output will be stored in `./data/measurements.txt`.

All of these options may be configured:
```shell
billion-row-gen \
    --rows 1000 \
    --weather-stations ./data/weather.csv \
    --output ./data/out.txt
```
