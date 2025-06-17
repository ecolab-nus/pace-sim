| PE   | cycle 0 | cycle 1 | cycle 2 | cycle 3  | cycle 4 | cycle 5 |
| ---- | ------- | ------- | ------- | -------- | ------- | ------- |
| Y0X0 | RST     | LD      | SEND    | NOP      | LD      | SEND    |
| Y1X0 | RST     | LD      | SEND    | NOP      | LD      | SEND    |
| Y0X1 | RST     | NOP     | RECV    | ADD+SEND | NOP     | RECV    |
| Y1X1 | RST     | NOP     | PASS    | RECV     | STORE   | PASS    |


Y0X0: LOAD, and SEND the data to Y0X1
Y1X0: LOAD, and SEND the data to Y0X1 (via Y1X1)

Y0X1: RECV and computing ADD
Y1X1: BYPASS for Y1X0 -> Y0X1, RECV sum from Y0X1, STORE