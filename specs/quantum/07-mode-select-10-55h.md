# Mode Select (10) - 55h

## What the Library Does With This Command

The library does not support any changeable parameters, and this command is supported for
compatibility only. This command can only be issued to the controller device logical unit.


## Command Usage

A MODE SENSE command with the PC field set to 1h and the Page Code field set to 3Fh can be issued
before the MODE SELECT command is issued to determine which mode parameters are supported,
which mode parameters are changeable, and the supported length of each page. Since the library does
not support any changeable parameters, use of MODE SELECT provides limited value. For a list of
available mode pages, see Mode Select (10) - 55h above.


## Mode Select (10) CDB Format

The ten-byte MODE SELECT CDB format is shown in the following table.

**Table 1: MODE SELECT CDB format**

```
              Bit        7           6              5        4             3      2       1   0
  Byte
         0                                                  Op Code (55h)
         1                      Reserved                    PF                 Reserved       SP
         2                                                       Reserved
         3                                                       Reserved
         4                                                       Reserved
         5                                                       Reserved
         6                                                       Reserved
         7
                                                        Parameter List Length
         8
         9                                                       Control
```

| Field | Description |
|---|---|
| Page Format (PF) | This bit indicates that the data sent by the initiator after the MODE SELECT header and block descriptors complies with the definition of pages in the SCSI standard. The value must be set to 1. |
| Save Parameters (SP) | Savable pages are not supported and this field must be set to 0. |
| Parameter List Length | This field specifies the number of bytes that will be transferred for the MODE SELECT parameter list, and should be equal to the length of a single Parameter List Header plus the lengths of all pages to be transferred. A length of zero indicates that no data is transferred. This is not considered to be an error. |

> **Note:** The SP bit is not tested for validity if the Parameter List Length field
> indicates that saveable parameter data is not sent.


## Mode Parameter Header

Following the MODE SELECT CDB, a single Mode Parameter Header should be sent as shown in the
following table. For the controller device, none of the fields are actually used however, and should all be
set to zero.

**Table 2: Mode Parameter Header format for Mode Select (10)**

```
          Bit      7            6            5           4              3         2            1           0
  Byte
      0
                                                             Reserved
      1
      2                                                      Reserved
      3                                                      Reserved
      4
                                                             Reserved
      5
      6
                                                             Reserved
      7
```
