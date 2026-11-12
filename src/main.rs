struct Terrain {
    width: usize,
    depth: usize,
    heights: Vec<f32>,
    cell_size: f32,
}

impl Terrain {
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
}

fn main() {
    let width = 9;
    let depth = 9;
    let mut heights = Vec::new();

    let hill_height: f32 = 10.0;

    let center_x = (width - 1) as f32 / 2.0;
    let center_z = (depth - 1) as f32 / 2.0;
    let max_distance = (center_x.powi(2) + center_z.powi(2)).sqrt();

    // Creation of test terrain with x and z coords
    for z in 0..depth {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dz = z as f32 - center_z;
            let distance = (dx.powi(2) + dz.powi(2)).sqrt();
            let height = hill_height * ((max_distance - distance) / max_distance).max(0.0);
            heights.push(height);
        }
    }

    let terrain = Terrain {
        width,
        depth,
        heights,
        cell_size: 2.0,
    };

    // Print the terrain as a grid
    terrain.print_grid();
}
