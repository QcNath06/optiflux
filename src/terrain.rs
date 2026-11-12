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

    /// Size of the terrain in metres: (width along x, depth along z).
    pub fn size_in_metres(&self) -> (f32, f32) {
        (
            (self.width - 1) as f32 * self.cell_size,
            (self.depth - 1) as f32 * self.cell_size,
        )
    }

    /// Lowest height that has data (ignores NaN).
    pub fn min_height(&self) -> f32 {
        let mut min_h = f32::MAX;
        for &h in &self.heights {
            if !h.is_nan() {
                min_h = min_h.min(h);
            }
        }
        min_h
    }

    /// Builds the 3D shape of the terrain: a list of points and a list of triangles.
    /// This has nothing to do with Bevy, so it can be tested and reused anywhere.
    /// - `step`: keep one point every `step` cells
    /// - `vertical_scale`: multiply heights to exaggerate the relief
    /// Heights are shifted so the lowest point is at y = 0.
    /// Returns (positions, indices): every 3 indices make one triangle.
    pub fn mesh_data(&self, step: usize, vertical_scale: f32) -> (Vec<[f32; 3]>, Vec<u32>) {
        let xs: Vec<usize> = (0..self.width).step_by(step).collect();
        let zs: Vec<usize> = (0..self.depth).step_by(step).collect();
        let cols = xs.len();
        let base = self.min_height();

        // 1. Points
        let mut positions = Vec::with_capacity(xs.len() * zs.len());
        for &z in &zs {
            for &x in &xs {
                let h = self.height(x, z);
                // No data → put it at the bottom. No triangle will use it anyway.
                let y = if h.is_nan() {
                    0.0
                } else {
                    (h - base) * vertical_scale
                };
                positions.push([x as f32 * self.cell_size, y, z as f32 * self.cell_size]);
            }
        }

        // 2. Triangles (same pattern as save_obj, but indices start at 0)
        let mut indices = Vec::new();
        for row in 0..zs.len() - 1 {
            for col in 0..cols - 1 {
                let corners = [
                    self.height(xs[col], zs[row]),
                    self.height(xs[col + 1], zs[row]),
                    self.height(xs[col], zs[row + 1]),
                    self.height(xs[col + 1], zs[row + 1]),
                ];
                if corners.iter().any(|h| h.is_nan()) {
                    continue;
                }

                let i = (row * cols + col) as u32;
                let right = i + 1;
                let below = i + cols as u32;
                let diag = below + 1;

                indices.extend_from_slice(&[i, below, right]);
                indices.extend_from_slice(&[right, below, diag]);
            }
        }

        (positions, indices)
    }

    /// Saves the terrain as a Wavefront .obj file.
    /// - `step`: keep one point every `step` cells (1 = all points, 4 = 16x smaller file)
    /// - `vertical_scale`: multiply heights to make the relief easier to see (1.0 = real)
    /// Cells with no data (NaN) become holes in the 3D model.
    pub fn save_obj(&self, path: &str, step: usize, vertical_scale: f32) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut out = BufWriter::new(file);

        // Which columns and rows we keep (0, step, 2*step, ...)
        let xs: Vec<usize> = (0..self.width).step_by(step).collect();
        let zs: Vec<usize> = (0..self.depth).step_by(step).collect();
        let cols = xs.len(); // number of points per row in the OBJ

        // 1. One vertex per kept point: "v x y z"
        for &z in &zs {
            for &x in &xs {
                let h = self.height(x, z);
                // A NaN would break 3D viewers. We write 0 instead,
                // and no triangle will use this point (see below).
                let y = if h.is_nan() { 0.0 } else { h * vertical_scale };
                writeln!(
                    out,
                    "v {} {} {}",
                    x as f32 * self.cell_size,
                    y,
                    z as f32 * self.cell_size
                )?;
            }
        }

        // 2. Two triangles per square, only where all 4 corners have data
        for row in 0..zs.len() - 1 {
            for col in 0..cols - 1 {
                let corners = [
                    self.height(xs[col], zs[row]),
                    self.height(xs[col + 1], zs[row]),
                    self.height(xs[col], zs[row + 1]),
                    self.height(xs[col + 1], zs[row + 1]),
                ];
                if corners.iter().any(|h| h.is_nan()) {
                    continue; // no data here → leave a hole
                }

                let i = row * cols + col + 1; // +1 because OBJ starts counting at 1
                let right = i + 1;
                let below = i + cols;
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
    fn obj_export_skips_no_data() {
        let mut terrain = Terrain::new_testing_terrain(5, 5, 4.0);
        terrain.heights[0] = f32::NAN; // (0, 0) has no data
        let path = std::env::temp_dir().join("optiflux_test.obj");
        let path = path.to_str().unwrap();
        terrain.save_obj(path, 1, 1.0).unwrap();

        let text = std::fs::read_to_string(path).unwrap();
        let vertices = text.lines().filter(|l| l.starts_with("v ")).count();
        let faces = text.lines().filter(|l| l.starts_with("f ")).count();

        assert_eq!(vertices, 25); // 5 x 5 points
        assert_eq!(faces, 30); // 16 squares x 2 triangles, minus the 2 touching (0, 0)
        assert!(!text.contains("NaN"));
    }

    #[test]
    fn mesh_data_has_right_counts() {
        let mut terrain = Terrain::new_testing_terrain(5, 5, 4.0);
        terrain.heights[0] = f32::NAN;
        let (positions, indices) = terrain.mesh_data(1, 1.0);
        assert_eq!(positions.len(), 25);
        assert_eq!(indices.len(), 30 * 3); // 30 triangles, 3 indices each
        // lowest valid point is at y = 0
        let lowest = positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        assert!(lowest.abs() < 0.0001);
    }
}
