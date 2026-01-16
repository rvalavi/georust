# Testing and exploring GDAL bindings in Rust

### Compiling on HPC
To complie the Rust bindings to GDAL, you need the following modules on a HPC Linux system (or your local system):

```bash
module load rust/1.92.0
module load gdal/3.12.1
module load llvm/18.1.8
```

```bash
cargo build
```