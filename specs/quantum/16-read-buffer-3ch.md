# Read Buffer - 3Ch

## What the Library Does With This Command

The library will return requested buffer descriptor information or buffer data.


## Command Usage

This command can be used primarily for enhanced domain validation. The initiator can use Descriptor
mode first to determine the size of the data available to read, followed by Data mode to then read it.
Depending on the size of the requested buffer, it can also be retrieved in blocks, utilizing offsets into the
buffer.


## Read Buffer CDB Format

The READ BUFFER CDB format is shown in the following table.

**Table 1: READ BUFFER CDB format**

```
              Bit        7           6              5      4              3    2        1          0
  Byte
          0                                              Op Code (3Ch)
          1                   Mode Specific                                   Mode
          2                                                    Buffer ID
          3
          :                                               Buffer Offset
          5
          6
          :                                             Allocation Length
          8
          9                                                     Control
```

| Field | Description |
|-------|-------------|
| Mode Specific | The usage of the MODE SPECIFIC field depends on the value in the MODE field. This field is reserved for all tape library supported modes per Mode definition below. |
| Mode | The supported modes are: |
| | - 2h -- Data, (the Mode Specific field is reserved) |
| | - 3h -- Descriptor, (the Mode Specific field is reserved) |
| | - Ah -- Echo Buffer, (the Mode Specific field is reserved) |
| | - Bh -- Echo Buffer Descriptor, (the Mode Specific field is reserved) |
| | In Data Mode, data is transferred from the buffer specified by the Buffer ID field. Buffer IDs are assigned beginning with zero, and are assigned contiguously. Buffer ID code assignments for the READ BUFFER command are the same as for the WRITE BUFFER command. |
| | In Descriptor Mode, a maximum of four bytes of READ BUFFER descriptor information is returned. The library returns the descriptor information for the buffer specified by the buffer ID. If there is no buffer associated with the specified Buffer ID, all zeros are returned in the READ BUFFER descriptor. The Buffer Offset field is reserved in this mode. The allocation length should be set to at least four for this mode. See Table 2 on the next page for a definition of the READ BUFFER descriptor. |
| | In Echo Buffer Mode, data is transferred to the initiator from the echo buffer. The echo buffer will transfer the same data that was received from the last WRITE BUFFER command sent with Echo Buffer Mode. If the allocation length is insufficient to accommodate the number of bytes of data as received in the prior echo buffer mode WRITE BUFFER command, the returned data will be truncated. This is not considered an error. If a prior echo buffer mode WRITE BUFFER command was not successfully completed the echo buffer mode READ BUFFER will return a Check Condition, with a Sense Key of Illegal Request and additional sense code of Command Sequence Error. The data may be read from the echo buffer multiple times. |
| | In Echo Buffer Descriptor Mode, a maximum of four bytes of READ BUFFER descriptor information is returned for the echo buffer. The Buffer Offset field is reserved in this mode. The allocation length should be set to at least four for this mode. See Table 3 on the next page for a definition of the READ BUFFER descriptor. See Table 5 on page 108 for a definition of the Echo Buffer descriptor. |
| | > **Note:** The Echo Buffer is supported only if the library control path is provided by a tape drive. |
| Buffer ID | This field specifies which buffer the request is for. A Buffer ID is currently not supported. The library currently only supports Echo Buffer communication. The IDs are the same for both the READ BUFFER and WRITE BUFFER commands. The IDs supported by the library, along with their primary use (data or download modes), are listed in Table 2 on the next page. |
| Buffer Offset | This field contains the byte offset within the specified buffer from which data shall be transferred. The initiator should conform to the offset boundary requirements returned in the READ BUFFER descriptor described in "Read Buffer Response." |
| Allocation Length | In Data Mode, this field should be set to accommodate the amount of data being requested for return. In Descriptor Mode, this field should be set to at least four. |

**Table 2: Supported Buffer ID**

| Buffer ID | Description | Read/Write |
|-----------|-------------|------------|
| N/A | A buffer ID is not currently supported. | N/A |

Additional Buffer IDs beyond those listed are reserved. Descriptor Mode can be used to determine the
size or capacity of a given buffer.


## Read Buffer Response

In Data Mode, the requested buffer of data is returned per the buffer offset and allocation length.

In Descriptor Mode, a buffer descriptor is returned as shown in the following table.

**Table 3: Read Buffer Descriptor**

```
              Bit        7             6            5           4        3          2          1          0
  Byte
          0                                                   Offset Boundary
          1
          :                                                   Buffer Capacity
          3
```

| Field | Description |
|-------|-------------|
| Offset Boundary | This field returns the boundary alignment (byte boundary) within the selected buffer for subsequent READ BUFFER commands. The value contained in this field is interpreted as a power of two. Therefore the value contained in the Buffer Offset field of subsequent READ BUFFER commands should be a multiple of 2^(offset boundary) as shown in Table 4 on the next page.. |
| Buffer Capacity | This field returns the size of the requested buffer in bytes. The Return Buffer stops being filled when the number of allocation length bytes has been transferred or when all the available data from the buffer has been transferred, whichever amount is less. This holds true for either mode. In Echo Buffer Descriptor Mode, an echo buffer descriptor is returned as shown in Table 5 below. |

**Table 4: Offset Boundary**

| Offset Boundary | 2^(offset boundary) | Buffer Offsets |
|-----------------|---------------------|----------------|
| 0h | 2^0 = 1 | Byte boundaries |
| 1h | 2^1 = 2 | Even-byte boundaries |
| 2h | 2^2 = 4 | Four-byte boundaries |
| 3h | 2^3 = 8 | Eight-byte boundaries |
| 4h | 2^4 = 16 | 16-byte boundaries |
| : | : | Etc. |

**Table 5: Echo Buffer Descriptor**

```
                  Bit      7           6          5           4           3          2            1      0
  Byte
          0                                              Reserved                                      EBOS
          1                                                        Reserved

          2                       Reserved                                          MSB
                                                                               Buffer Capacity
          3                                                                                                  LSB
```

| Field | Description |
|-------|-------------|
| Echo Buffer Overwritten Supported (EBOS) | This field returns a 0 to indicate that other initiators or intervening commands may overwrite the echo buffer. An EBOS bit set to 1 indicates either: |
| | - the target returns the ECHO BUFFER OVERWRITTEN extended sense code if the data being read from the echo buffer is not the data previously written by the same initiator, or |
| | - the target ensures echo buffer data from each initiator is the same as that previously written by the same initiator. |
| Buffer Capacity | This field returns 252, which is the size of the echo buffer (in bytes). |
