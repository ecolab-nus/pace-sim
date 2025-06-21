| PE   | cycle 0 | cycle 1 | cycle 2 | cycle 3  | cycle 4 | cycle 5 (next period) |
| ---- | ------- | ------- | ------- | -------- | ------- | --------------------- |
| Y0X0 | RST     | LD      | SEND    | NOP      | NOP     | LD                    |
| Y1X0 | RST     | LD      | SEND    | RECV     | STORE   | LD                    |
| Y0X1 | RST     | NOP     | RECV    | ADD+SEND | NOP     | NOP                   |
| Y1X1 | RST     | NOP     | PASS    | PASS     | NOP     | NOP                   |


Y0X0: LOAD, and SEND the data to Y0X1
Y1X0: LOAD, and SEND the data to Y0X1 (via Y1X1); RECV the sum from Y0X1

Y0X1: RECV (from both Y0X0 and Y1X1) and computing ADD, SEND to Y1X0
Y1X1: BYPASS for Y1X0 -> Y0X1, BYPASS for Y0X1 -> Y0X0