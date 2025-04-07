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
2. **Run 1**: Execute the loader, providing the path to your E57 file.
    ```
    export PATH=$PATH:`pwd`/target/release 
    rerun /path/to/your/file.e57
    ```
    You can also run rerun, ensure that the built binary is on the PATH and then drag and drop an E57 file in rerun. 

3. **Run 2**: To limit the number of of scans, use the RERUN_E57_DISPLAY_SCANS environment var:

    ```
    export PATH=$PATH:`pwd`/target/release 
    RERUN_E57_DISPLAY_SCANS=0,1,5,10 rerun /path/to/your/file.e57
    ```