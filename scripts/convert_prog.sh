# convert all the prog files in a folder to binprog files
for file in *.prog; do ~/pace-sim/target/debug/convert "$file" "${file%.prog}"; done