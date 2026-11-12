mod terrain;
use terrain::Terrain;

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
