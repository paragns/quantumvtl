# Release Element (10) - 57h

## What the Library Does With This Command

The library releases any outstanding reservation that had previously been made by the same initiator via
the RESERVE command. Only whole logical unit reservations are allowed; individual element
reservations are not supported.


## Command Usage

This command should be used to release the library from any reservations previously made.


## Release Element CDB Format

The RELEASE ELEMENT CDB format is shown in the following table.

**Table 1: RELEASE ELEMENT CDB format**

```
             Bit    7           6         5           4             3         2            1       0
  Byte
         0                                                Op Code (57h)
         1                   Reserved               3rdPrty         Reserved             LongID   Rsvd
         2                                                 Reserved
         3                                            Third Party Device ID
         4
         :                                                    Reserved
         6
         7         MSB
         :                                           Parameter List Length
         8                                                                                          LSB
         9                                                    Control
```

| Field | Description |
|-------|-------------|
| 3rdPrty | This field is not supported, and must be set to 0. |
| LongID | This field is not supported, and must be set to 0. |
| Third Party Device ID | This field is not supported, and must be set to 0. |
| Parameter List Length | This field is not supported, and must be set to 0. |
