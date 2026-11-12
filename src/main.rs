mod terrain;
use terrain::Terrain;

fn main() {
    let text = std::fs::read_to_string("../data/optifluxData/test_small.asc")
        .expect("Could not read the ASC file");

    let mut ncols: usize = 0;
    let mut nrows: usize = 0;
    let mut x_corner: f64 = 0.0;
    let mut y_corner: f64 = 0.0;
    let mut cell_size: f64 = 1.0;
    let mut nodata: f64 = -9999.0;
    let mut header_lines = 0;

    for line in text.lines() {
        let mut words = line.split_whitespace();
        let key = words.next().unwrap();

        // Header lines start with a letter. Data lines start with a number.
        if !key.chars().next().unwrap().is_alphabetic() {
            break;
        }

        header_lines += 1;

        let value: f64 = words.next().unwrap().parse().unwrap();

        match key.to_lowercase().as_str() {
            "ncols" => ncols = value as usize,
            "nrows" => nrows = value as usize,
            "xllcorner" => x_corner = value,
            "yllcorner" => y_corner = value,
            "cellsize" | "dx" => cell_size = value,
            "nodata_value" => nodata = value,
            _ => {} // any other keyword (like "dy"): ignore
        }
    }

    let mut heights: Vec<f32> = Vec::with_capacity(ncols * nrows);
    let mut nodata_count = 0;

    for line in text.lines().skip(header_lines) {
        for word in line.split_whitespace() {
            let h: f64 = word.parse().unwrap();
            if h == nodata {
                nodata_count += 1;
            }
            heights.push(h as f32);
        }
    }

    println!("Heights read: {}", heights.len());
    println!("No-data cells: {}", nodata_count);

    println!("Number of cells: {}", ncols * nrows);
    println!("Grid: {} x {} cells", ncols, nrows);
    println!("Bottom-left corner: ({}, {})", x_corner, y_corner);
    println!("Cell size: {} m", cell_size);
    println!("No data value: {}", nodata);
    println!("File size: {} characters", text.len());
}
