use std::path::Path;
use std::process::Command;

fn main() {
    let data_ron_path = "assetgen/pokemon_data.ron";
    let sprites_tar_gz_path = "assetgen/sprites.tar.gz";

    let data_ron_exists = Path::new(data_ron_path).exists();
    let sprites_tar_gz_exists = Path::new(sprites_tar_gz_path).exists();

    if !data_ron_exists || !sprites_tar_gz_exists {
        println!("cargo:warning=Missing asset files, running assetgen...");

        // Execute assetgen with -a flag
        let output = Command::new("cargo")
            .args(["run", "--bin", "assetgen", "--", "-a"])
            .output()
            .expect("Failed to execute assetgen command");

        // Check if the command was successful
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("assetgen failed with error: {}", stderr);
        }

        println!("cargo:warning=Asset generation completed successfully");
    } else {
        println!("cargo:warning=Asset files found, skipping assetgen");
    }

    // tell cargo to rerun this script if the asset files are deleted
    println!("cargo:rerun-if-changed={}", data_ron_path);
    println!("cargo:rerun-if-changed={}", sprites_tar_gz_path);
}
