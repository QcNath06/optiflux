use std::fs::File;
use std::io::{BufWriter, Write};

struct Terrain {
    width: usize,
    depth: usize,
    heights: Vec<f32>,
    cell_size: f32,
}

impl Terrain {
    fn new_testing_terrain(width: usize, depth: usize, hill_height: f32) -> Terrain {
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

    fn print_grid(&self) {
        for z in 0..self.depth {
            for x in 0..self.width {
                print!("{:5.1}", self.height(x, z));
            }
            println!();
        }
    }

    /// Slope in percent from (x, z) to its right neighbor (x + 1, z).
    /// Negative = going down, positive = going up.
    fn slope_right(&self, x: usize, z: usize) -> Option<f32> {
        if x + 1 >= self.width || z >= self.depth {
            return None;
        }
        // get the height at (x, z)
        let current_height = self.height(x, z);
        // 2. get the height at (x + 1, z)
        let neighbor_height = self.height(x + 1, z);
        // 3. calculate the height difference (neighbor - current)
        let height_difference = neighbor_height - current_height;
        // 4. divide by the horizontal distance, then multiply by 100
        Some((height_difference / self.cell_size) * 100.0)
    }

    fn slope_between(&self, x1: usize, z1: usize, x2: usize, z2: usize) -> Option<f32> {
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
        Some((height_difference / horizontal_distance) * 100.0)
    }

    fn slope_down(&self, x: usize, z: usize) -> Option<f32> {
        if x >= self.width || z + 1 >= self.depth {
            return None;
        }
        let current_height = self.height(x, z);
        let neighbor_height = self.height(x, z + 1);
        let height_difference = neighbor_height - current_height;
        Some((height_difference / self.cell_size) * 100.0)
    }

    fn save_obj(&self, path: &str) -> std::io::Result<()> {
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

fn main() {
    // Create a testing terrain with a hill in the center
    let terrain = Terrain::new_testing_terrain(9, 9, 13.0);
    // Print the terrain as a grid
    terrain.print_grid();

    // Straight neighbors (should give the same results as before)
    match terrain.slope_between(0, 4, 1, 4) {
        Some(slope) => println!("(0,4) → (1,4): {:.1}%", slope),
        None => println!("(0,4) → (1,4): outside the terrain"),
    }

    // Diagonal
    match terrain.slope_between(0, 0, 1, 1) {
        Some(slope) => println!("(0,0) → (1,1): {:.1}%", slope),
        None => println!("(0,0) → (1,1): outside the terrain"),
    }

    // Along the edge (not toward the top of the hill)
    match terrain.slope_between(0, 0, 1, 0) {
        Some(slope) => println!("(0,0) → (1,0): {:.1}%", slope),
        None => println!("(0,0) → (1,0): outside the terrain"),
    }

    // Outside the terrain
    match terrain.slope_between(8, 8, 9, 9) {
        Some(slope) => println!("(8,8) → (9,9): {:.1}%", slope),
        None => println!("(8,8) → (9,9): outside the terrain"),
    }

    //Export 3D model to OBJ file
    terrain
        .save_obj("terrain.obj")
        .expect("Unable to save OBJ file");
}
