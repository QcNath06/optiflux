mod terrain;
use terrain::Terrain;

fn main() {
    // Load the real LiDAR terrain (MNT) exported from QGIS
    let terrain = Terrain::from_asc("../data/optifluxData/test_small.asc");

    // Small map: '#' = data, '.' = no data (one character every 40 m)
    terrain.print_coverage_map(40);

    println!("Height at (0, 0): {:.2} m", terrain.height(0, 0));

    // Investigation: print the raw heights with 3 decimals
    println!("Height at (1000,500): {:.3} m", terrain.height(1000, 500));
    println!("Height at (1001,500): {:.3} m", terrain.height(1001, 500));

    // Slope over a longer distance (10 m)
    match terrain.slope_between(1000, 500, 1010, 500) {
        Some(slope) => println!("Slope over 10 m: {:.3}%", slope),
        None => println!("Slope over 10 m: no data"),
    }

    // Try a few other spots
    for (x, z) in [(600, 200), (900, 800), (1500, 300), (1700, 900)] {
        match terrain.slope_between(x, z, x + 1, z) {
            Some(slope) => println!("Slope at ({},{}): {:.3}%", x, z, slope),
            None => println!("Slope at ({},{}): no data", x, z),
        }
    }
}
