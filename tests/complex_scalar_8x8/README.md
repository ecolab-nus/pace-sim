```
    let y0x0 = load(i); // period starts at cycle 2
    let y0x1 = LS(y0x0, #5); // period starts at cycle 3
    let y0x2 = ARS(y0x1, #6); // period starts at cycle 4
    let y0x3 = MUL(y0x2, #7); // period starts at cycle 5
    let y1x3 = DIV(y0x3, #6); // period starts at cycle 6
    let y1x2 = XOR(y1x3, 0b10101010101010101); // period starts at cycle 7
    let y1x1 = SUB(y1x2, 255); // period starts at cycle 8
    let y1x0 = STORE(y1x1, 512) // period starts at cycle 9
```
NOP before the period start to synchronize.
Last NOP before period start should do the routing