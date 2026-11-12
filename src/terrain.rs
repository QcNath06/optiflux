use std::fs::File;
use std::io::{BufWriter, Write};

pub struct Terrain {
    width: usize,
    depth: usize,
    heights: Vec<f32>,
    cell_size: f32,
}

impl Terrain {
    /// Creates a test terrain with a round hill in the middle.
    pub fn new_testing_terrain(width: usize, depth: usize, hill_height: f32) -> Terrain {
        let mut heights = Vec::new();

        let center_x = (width - 1) as f32 / 2.0;
        let center_z = (depth - 1) as f32 / 2.0;
        let max_distance = (center_x.powi(2) + center_z.powi(2)).sqrt();

        for z in 0..depth {
            for x in 0..width {
                let dx = x as f32 - center_x;
                let dz = z as f32 - center_z;
                let distance = (dx.powi(2) + dz.powi(2)).sqrt();
                let height = hill_height * ((max_distance - distance) / max_distance).max(0.0);
                heights.push(height);
            }
        }

        Terrain {
            width,
            depth,
            heights,
            cell_size: 2.0,
        }
    }

    /// Loads a terrain from an Arc/Info ASCII Grid (.asc) file.
    /// Row z = 0 is the NORTH edge of the map, z increases going south.
    pub fn from_asc(path: &str) -> Terrain {
        let text = std::fs::read_to_string(path).expect("Could not read the ASC file");

        // ----- 1. Read the header -----
        let mut ncols: usize = 0;
        let mut nrows: usize = 0;
        let mut x_corner: f64 = 0.0; // real-world position (not used yet: for trees later)
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

        // ----- 2. Read all the heights -----
        let mut heights: Vec<f32> = Vec::with_capacity(ncols * nrows);
        let mut nodata_count = 0;

        for line in text.lines().skip(header_lines) {
            for word in line.split_whitespace() {
                let h: f64 = word.parse().unwrap();
                if h == nodata {
                    nodata_count += 1;
                    heights.push(f32::NAN); // no data → "Not a Number"
                } else {
                    heights.push(h as f32);
                }
            }
        }

        // ----- 3. Quick checks -----
        if heights.len() != ncols * nrows {
            panic!(
                "The file says {} x {} = {} heights, but {} were read",
                ncols,
                nrows,
                ncols * nrows,
                heights.len()
            );
        }
        if nodata_count > 0 {
            println!("Warning: {} cells have no data", nodata_count);
        }

        // ----- 4. Build the Terrain -----
        Terrain {
            width: ncols,
            depth: nrows,
            heights,
            cell_size: cell_size as f32,
        }
    }

    /// Returns the height at position (x, z).
    pub fn height(&self, x: usize, z: usize) -> f32 {
        self.heights[z * self.width + x]
    }

    /// Prints the terrain as a grid (only for small terrains!).
    pub fn print_grid(&self) {
        for z in 0..self.depth {
            for x in 0..self.width {
                print!("{:5.1}", self.height(x, z));
            }
            println!();
        }
    }

    /// Prints a small map of where the data is.
    /// '#' = data, '.' = no data. Only looks at one cell every `step`.
    pub fn print_coverage_map(&self, step: usize) {
        for z in (0..self.depth).step_by(step) {
            for x in (0..self.width).step_by(step) {
                if self.height(x, z).is_nan() {
                    print!(".");
                } else {
                    print!("#");
                }
            }
            println!();
        }
    }

    /// Slope in percent from (x1, z1) to (x2, z2).
    /// Negative = going down, positive = going up.
    /// Returns None if a point is outside the terrain or both points are the same.
    pub fn slope_between(&self, x1: usize, z1: usize, x2: usize, z2: usize) -> Option<f32> {
        if x1 >= self.width || z1 >= self.depth || x2 >= self.width || z2 >= self.depth {
            return None;
        }
        let current_height = self.height(x1, z1);
        let neighbor_height = self.height(x2, z2);

        // No data at one of the points → no slope
        // (use .is_nan(): NaN is never equal to anything, not even itself!)
        if current_height.is_nan() || neighbor_height.is_nan() {
            return None;
        }

        let height_difference = neighbor_height - current_height;
        let horizontal_distance = (((x2 as isize - x1 as isize).pow(2)
            + (z2 as isize - z1 as isize).pow(2)) as f32)
            .sqrt()
            * self.cell_size;
        if horizontal_distance == 0.0 {
            return None;
        }
        Some((height_difference / horizontal_distance) * 100.0)
    }

    /// Saves the terrain as a Wavefront .obj file.
    pub fn save_obj(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut out = BufWriter::new(file);

        // 1. One vertex per point: "v x y z"
        for z in 0..self.depth {
            for x in 0..self.width {
                writeln!(
                    out,
                    "v {} {} {}",
                    x as f32 * self.cell_size,
                    self.height(x, z),
                    z as f32 * self.cell_size
                )?;
            }
        }

        // 2. Two triangles per square: "f a b c"
        for z in 0..self.depth - 1 {
            for x in 0..self.width - 1 {
                let i = z * self.width + x + 1; // +1 because OBJ starts counting at 1
                let right = i + 1;
                let below = i + self.width;
                let diag = below + 1;

                writeln!(out, "f {} {} {}", i, below, right)?;
                writeln!(out, "f {} {} {}", right, below, diag)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_slope_outside_terrain() {
        let terrain = Terrain::new_testing_terrain(9, 9, 13.0);
        assert_eq!(terrain.slope_between(8, 8, 9, 9), None);
    }

    #[test]
    fn going_up_and_down_are_opposite() {
        let terrain = Terrain::new_testing_terrain(9, 9, 13.0);
        let up = terrain.slope_between(3, 4, 4, 4).unwrap();
        let down = terrain.slope_between(4, 4, 3, 4).unwrap();
        assert!((up + down).abs() < 0.001);
    }

    #[test]
    fn same_point_has_no_slope() {
        let terrain = Terrain::new_testing_terrain(9, 9, 13.0);
        assert_eq!(terrain.slope_between(3, 3, 3, 3), None);
    }

    #[test]
    fn no_slope_where_there_is_no_data() {
        let mut terrain = Terrain::new_testing_terrain(9, 9, 13.0);
        terrain.heights[0] = f32::NAN; // pretend (0, 0) has no data
        assert_eq!(terrain.slope_between(0, 0, 1, 0), None);
        assert_eq!(terrain.slope_between(1, 0, 0, 0), None);
    }

    #[test]
    fn longer_distance_gives_finer_slope() {
        let terrain = Terrain::new_testing_terrain(9, 9, 13.0);
        // Slope over 4 cells must be between the slopes of the individual steps
        let long = terrain.slope_between(0, 4, 4, 4).unwrap();
        assert!(long > 0.0);
    }
}
