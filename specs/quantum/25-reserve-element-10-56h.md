# Reserve Element (10) - 56h

## What the Library Does With This Command

The library reserves the entire library for the initiator making the request. Only whole logical unit
reservations are allowed; individual element reservations are not supported. The reservation remains in
effect until either the initiator that made the reservation sends a RELEASE command, or a reset or power-
cycle of the library occurs.

This command is only supported if the library control path is provided by a tape drive.

> **Note:** DA blades connected to drives that also configure a partition control path, may report certain
> library ready conditions differently than drives that are configured to host a partition control path.


## Command Usage

This command should be used to reserve the library for extended operations, such as issuing a SEND
VOLUME TAG followed by a REQUEST VOLUME ELEMENT ADDRESS sequence. Initiators issuing a
RESERVE should follow it with a RELEASE when the extended operation sequence is complete.


## Reserve Element CDB Format

The RESERVE ELEMENT CDB format is shown in the following table.

**Table 1: RESERVE ELEMENT CDB format**

```
             Bit    7           6         5           4             3         2     1         0
  Byte
         0                                                Op Code (56h)
         1                   Reserved               3rdPrty         Reserved      LongID    Rsvd
         2                                                 Reserved
         3                                            Third Party Device ID
         4
         :                                                    Reserved
         6
         7         MSB
         :                                           Parameter List Length
         8                                                                                     LSB
         9                                                    Control
```

- **3rdPrty**: This field is not supported, and must be set to 0.
- **LongID**: This field is not supported, and must be set to 0 as IDs greater than 255 are not supported.
- **Third Party Device ID**: This field is required and used only when the 3rdPrty bit is set. Since the 3rdPrty bit is not supported, this field must be set to 0.
- **Parameter List Length**: This field is not supported, and must be set to 0.
