use std::fs::File;
use std::io::{BufWriter, Write};

pub struct Terrain {
    width: usize,
    depth: usize,
    heights: Vec<f32>,
    cell_size: f32,
}

impl Terrain {
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

    fn height(&self, x: usize, z: usize) -> f32 {
        self.heights[z * self.width + x]
    }

    pub fn print_grid(&self) {
        for z in 0..self.depth {
            for x in 0..self.width {
                print!("{:5.1}", self.height(x, z));
            }
            println!();
        }
    }

    pub fn slope_between(&self, x1: usize, z1: usize, x2: usize, z2: usize) -> Option<f32> {
        if x1 >= self.width || z1 >= self.depth || x2 >= self.width || z2 >= self.depth {
            return None;
        }
        let current_height = self.height(x1, z1);
        let neighbor_height = self.height(x2, z2);
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
}
