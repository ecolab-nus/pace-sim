# PACE-Sim

# Basic Usage
Get the usage of the simulate binary with
```
cargo run --bin simulation --help
```

You can convert between format with
```
cargo run --bin convert <file> <file>
```
The file type recognization relies on the file extension:
.binprog for binary string
.prof for mnemonic (human readable and writeable)
.bin for actual binary

### Next TODOs:
- AGU
- Floating Point SIMD

### Further TODOs:
TODOs:
- Complete ISA
