| PE   | cycle 0 | cycle 1 | cycle 2 | cycle 3  | cycle 4 | cycle 5  | cycle 6 (next period) |
| ---- | ------- | ------- | ------- | -------- | ------- | -------- | --------------------- |
| Y0X0 | RST     | LD      | SEND    | NOP      | NOP     | NOP      | LD                    |
| Y1X0 | RST     | LD      | LD/SEND | SEND     | RECV    | STORE    | LD                    |
| Y0X1 | RST     | NOP     | RECV    | MULT+SEND| NOP     | NOP      | NOP                   |
| Y1X1 | RST     | NOP     | PASS    | RECV     | ADD/SEND| NOP      | NOP                   |


res = a*b + c
Y0X0: LOAD a, and SEND a to Y0X1
Y1X0: LOAD b, and SEND b to Y0X1 (via Y1X1); LOAD c and SEND c to Y1X1; RECV res from Y1X1; STORE res

Y0X1: RECV (from both Y0X0 and Y1X1) and computing MULT, SEND to Y1X1
Y1X1: BYPASS for b going to Y0X1; ADD a*b, c, SEND to Y1X0

Memory:
dm0[0-7 bytes]: a
dm0[8-15 bytes]: b
dm0[16-23 bytes]: c
dm0[24-31 bytes]: result