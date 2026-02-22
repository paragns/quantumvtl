# Release Element (6) - 17h

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
              Bit     7          6          5            4               3      2         1      0
  Byte
          0                                               Op Code (17h)
          1                 Reserved                 3rdPrty         Third Party Device ID    Element
          2                                              Reservation ID
          3
                                                             Reserved
          4
          5                                                    Control
```

| Field | Description |
|-------|-------------|
| 3rdPrty | This field is not supported, and must be set to 0. |
| Third Party Device ID | This field is not supported, and must be set to 0. |
| Element | This field is not supported, and must be set to 0. |
| Reservation ID | This field is not supported, and must be set to 0. |
