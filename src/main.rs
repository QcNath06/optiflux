struct Terrain {
    width: usize,
    depth: usize,
    heights: Vec<f32>,
    cell_size: f32,
}

fn main() {
    let width = 9;
    let depth = 9;
    let mut heights = Vec::new();

    let center_x = (width - 1) as f32 / 2.0;
    let center_z = (depth - 1) as f32 / 2.0;
    let max_distance = center_x + center_z;

    // Creation of test terrain with x and z coords
    for z in 0..depth {
        for x in 0..width {
            let distance = (x as f32 - center_x).abs() + (z as f32 - center_z).abs();
            let height = max_distance - distance;
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
    for z in 0..terrain.depth {
        for x in 0..terrain.width {
            let index = z * terrain.width + x;
            print!("{:5.1}", terrain.heights[index]);
        }
        println!();
    }
}
