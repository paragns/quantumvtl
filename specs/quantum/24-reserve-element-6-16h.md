# Reserve Element (6) - 16h

## What the Library Does With This Command

The library reserves the entire library for the initiator making the request. Only whole logical unit
reservations are allowed; individual element reservations are not supported. The reservation remains in
effect until either the initiator that made the reservation sends a RELEASE command, or a reset or power-
cycle of the library occurs.


## Command Usage

This command should be used to reserve the library for extended operations, such as issuing a SEND
VOLUME TAG followed by a REQUEST VOLUME ELEMENT ADDRESS sequence. Initiators issuing a
RESERVE should follow it with a RELEASE when the extended operation sequence is complete.


## Reserve Element CDB Format

The RESERVE ELEMENT CDB format is shown in the following table.

**Table 1: RESERVE ELEMENT CDB format**

```
              Bit      7          6          5          4           3          2           1      0
  Byte
          0                                              Op Code (16h)
                                                      3rd
          1                  Reserved                              Third Party Device ID       Element
                                                      Party
          2                                             Reservation ID
          3
                                                            Reserved
          4
          5                                                   Control
```

- **3rdPrty**: This field is not supported, and must be set to 0.
- **Third Party Device ID**: This field is not supported, and must be set to 0.
- **Element**: This field is not supported, and must be set to 0.
- **Reservation ID**: This field is not supported, and must be set to 0.
