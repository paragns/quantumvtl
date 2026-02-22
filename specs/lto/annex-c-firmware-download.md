# Annex C. Firmware Download

This annex describes how to compare a firmware level binary to information returned by a drive and determine which firmware is acceptable in which drive.


## C.1. Identifying Level Hardware of Drive

The firmware that is loaded in the drive reports a LOAD ID and RU NAME in the IP 03h: Firmware Designation (see 6.3.2 on page 228) inquiry page. The LOAD ID and RU NAME are used to designate the Level Hardware (i.e., Product). The following table defines the LOAD ID and RU NAME values for each Level Hardware.

### Table C.1 -- Load ID and RU Name Designation for LTO-9+

| Product (Level Hardware) | Interface | LOAD ID | RU Name "EBCDIC" Hex | PRODUCT ID IBM | PRODUCT ID OEM |
|--------------------------|-----------|---------|----------------------|----------------|----------------|
| LTO-9 Full-High | FCP | A1800300 | "AJEFH700" 0xC1D1C5C6C8F7F0F0 | ULT3580-TD9 | ULTRIUM-TD9 |
| LTO-9 Full-High | SSP | A1800301 | "AJEFH701" 0xC1D1C5C6C8F7F0F1 | ULT3580-TD9 | ULTRIUM-TD9 |
| LTO-9 Half-High | FCP | A1800302 | "AJEFH702" 0xC1D1C5C6C8F7F0F2 | ULT3580-TD9 | ULTRIUM-TD9 |
| LTO-9 Half-High | SSP | A1800303 | "AJEFH703" 0xC1D1C5C6C8F7F0F3 | ULT3580-TD9 | ULTRIUM-TD9 |

- **FCP**: 8G Fibre Channel
- **FCA**: 16G Fibre Channel
- **FCB**: 32G Fibre Channel
- **SSP**: 12 Gbps SAS

### Table C.2 -- Load ID and RU Name Designation for LTO-5 through LTO-8

| Product (Level Hardware) | LOAD ID | RU Name "EBCDIC" Hex | PRODUCT ID IBM | PRODUCT ID OEM | PRODUCT ID eServer |
|--------------------------|---------|----------------------|----------------|----------------|--------------------|
| **LTO-8** | | | | | |
| FH 8GFC | A1700D8B | "AJEFAX8B" 0xC1D1C5C6C1E7F8C2 | ULT3580-TD8 | ULTRIUM-TD8 | - |
| HH 8GFC | A1700D8C | "AJEFAX8C" 0xC1D1C5C6C1E7F8C3 | ULT3580-HH8 | ULTRIUM-HH8 | HH LTO Gen 8 |
| HH SAS | A1700D8D | "AJEFAX8D" 0xC1D1C5C6C1E7F8C4 | ULT3580-HH8 | ULTRIUM-HH8 | HH LTO Gen 8 |
| **LTO-7** | | | | | |
| FH 8GFC | A1700D87 | "AJEFAX87" 0xC1D1C5C6C1E7F8F7 | ULT3580-TD7 | ULTRIUM-TD7 | - |
| FH SAS | A1700D88 | "AJEFAX88" 0xC1D1C5C6C1E7F8F8 | ULT3580-TD7 | ULTRIUM-TD7 | - |
| HH 8GFC | A1700D89 | "AJEFAX89" 0xC1D1C5C6C1E7F8F9 | ULT3580-HH7 | ULTRIUM-HH7 | HH LTO Gen 7 |
| HH SAS | A1700D8A | "AJEFAX8A" 0xC1D1C5C6C1E7F8C1 | ULT3580-HH7 | ULTRIUM-HH7 | HH LTO Gen 7 |
| **LTO-6** | | | | | |
| FH 8GFC | A1700D81 | "AJEFAX81" 0xC1D1C5C6C1E7F8F1 | ULT3580-TD6 | ULTRIUM-TD6 | - |
| FH SAS | A1700D82 | "AJEFAX82" 0xC1D1C5C6C1E7F8F2 | ULT3580-TD6 | ULTRIUM-TD6 | - |
| HH 8GFC | A1700D83 | "AJEFAX83" 0xC1D1C5C6C1E7F8F3 | ULT3580-HH6 | ULTRIUM-HH6 | HH LTO Gen 6 |
| HH SAS | A1700D84 | "AJEFAX84" 0xC1D1C5C6C1E7F8F4 | ULT3580-HH6 | ULTRIUM-HH6 | HH LTO Gen 6 |
| **LTO-5** | | | | | |
| FH 8GFC | A1700D74 | "AJEFAX74" 0xC1D1C5C6C1E7F7F4 | ULT3580-TD5 | ULTRIUM-TD5 | - |
| FH SAS | A1700D75 | "AJEFAX75" 0xC1D1C5C6C1E7F7F5 | ULT3580-TD5 | ULTRIUM-TD5 | - |
| HH 8GFC / HH 8GFC V2 | A1700D76 | "AJEFAX76" 0xC1D1C5C6C1E7F7F6 | ULT3580-HH5 | ULTRIUM-HH5 | HH LTO Gen 5 |
| HH SAS / HH SAS V2 | A1700D77 | "AJEFAX77" 0xC1D1C5C6C1E7F7F7 | ULT3580-HH5 | ULTRIUM-HH5 | HH LTO Gen 5 |

- `-` Not Applicable
- **FH** - Full-High
- **HH** - Half-High


## C.2. Identifying the product for which the firmware image is intended

The Firmware Image is defined in table C.3.

### Table C.3 -- Firmware Image

```
         Bit
Byte     7 msb    6       5       4       3       2       1    0 lsb
 0-3                         Not Specified
 4-7     (MSB)    FIRMWARE LENGTH + HEADER LENGTH (m+1)       (LSB)
 8-11    (MSB)    LOAD ID                                     (LSB)
                  (see 6.3.2--IP 03h: Firmware Designation)
12-15    (MSB)    FIRMWARE REVISION LEVEL                     (LSB)
                  (see 5.2.5.1--Standard Inquiry Data bytes 32 - 35)
16-23                        Reserved
24-31    (MSB)    RU NAME                                     (LSB)
                  (see 6.3.2--IP 03h: Firmware Designation)
32-m                         Not Specified
```

The LOAD ID and RU NAME fields in the Firmware Image are used to define the product (i.e., Level Hardware) for which the Firmware Image is intended.


## C.3. Download Process

Confirm the Level Hardware of the Firmware Image (see C.2.) to be loaded matches the Level Hardware of the drive (see C.1.).

Download the Firmware Image using the WRITE BUFFER - 3Bh (see 5.2.44 on page 181) command.
