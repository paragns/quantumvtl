# Write Buffer - 3Bh

## What the Library Does With This Command

The library will receive a requested buffer of data and write it to the appropriate internal storage.


## Command Usage

This command can be used primarily for enhanced domain validation (using the echo buffer mode). The
initiator can either transfer the data with a single WRITE BUFFER command, or it can also transfer it in
blocks utilizing offsets into the buffer.


## Write Buffer CDB Format

The WRITE BUFFER CDB format is shown in the following table.

**Table 1: WRITE BUFFER CDB format**

```
              Bit        7           6              5        4             3     2          1           0
  Byte
          0                                                 Op Code (3Bh)
          1                   Mode Specific                Rsvd                      Mode
          2                                                   Buffer ID
          3
          :                                                 Buffer Offset
          5
          6
          :                                             Parameter List Length
          8
          9                                                      Control
```

- **Mode Specific**: This field is reserved and must be set to 0 as none of the supported Buffer IDs require mode specific details.
- **Mode**: The supported modes are:
  - 2h -- Data
  - Ah -- Echo Buffer

  In Data Mode, an amount of data specified by the Parameter List Length is targeted for the buffer defined by the Buffer ID field, starting at the specified Buffer Offset. Buffer IDs are assigned beginning with zero, and are assigned contiguously. Buffer ID code assignments for the WRITE BUFFER command are the same as for the READ BUFFER command.

  In Echo Buffer Mode, the amount of data specified by the Parameter List Length is transferred from the initiator to the echo buffer. The Buffer ID and Buffer Offset fields are ignored in this mode.

  > **Note:** The Echo Buffer is supported only if the library control path is provided by a tape drive.

- **Buffer ID**: This field specifies which buffer the request is for. The IDs are the same for both the READ BUFFER and WRITE BUFFER commands. The IDs supported by the library, along with their primary use, are listed in Table 2 on page 107.
- **Buffer Offset**: This field indicates the starting location (byte offset) within the specified buffer to write data. The initiator should conform to the offset boundary requirements returned in the READ BUFFER descriptor described in "Read Buffer Response."
- **Parameter List Length**: If applicable, this field should be set to indicate the amount of data being written.
