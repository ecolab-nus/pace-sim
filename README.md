# PACE-Sim

# Basic Usage
Build:
```
cargo build
```

You can simulate with snapshot and memory dump.
Refer to
```
target/debug/simulation --help
```
for the details.

You can convert between format with
```
target/debug/convert <file> <file>
```
The file type recognization relies on the file extension:
.binprog for binary string
.prog for mnemonic (human readable and writeable)

### Next TODOs:
- AGU
- Floating Point SIMD

### Further TODOs:
TODOs:
- Complete ISA
