mod terrain;
mod viewer;

fn main() {
    // Open the 3D viewer on the real LiDAR terrain:
    //  - one point every 2 m
    //  - heights x 3 so the relief is easier to see
    viewer::run("../data/optifluxData/test_small.asc", 2, 3.0);
}
