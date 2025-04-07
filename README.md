# Rerun Loader for E57

This project provides an external data loader for Rerun. It is designed to load E57 pointcloud files, offering an efficient way to handle 3D point cloud data.

## Project Overview

- **Main Logic**: All the core functionality is implemented in `main.rs`.
- **Purpose**: Load and process E57 pointcloud files for visualization and analysis in Rerun.

## Usage

1. **Build**: Compile the project using Cargo.
    ```
    cargo build --release
    ```
2. **Run**: Execute the loader, providing the path to your E57 file.
    ```
    export PATH=$PATH:`pwd`/target/release 
    rerun /path/to/your/file.e57
    ```
