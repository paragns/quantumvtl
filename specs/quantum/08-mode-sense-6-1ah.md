# Mode Sense (6) - 1Ah

## What the Library Does With This Command

The library will return the current settings for the supported mode pages. This command can be issued to
the DA blade controller device logical unit as well as the media changer device logical units. The mode
pages supported by each device vary.


## Command Usage

This command can be used to determine certain operational settings governing the behavior of the
library. For example, the number of elements and their assigned addresses can be obtained through use
of MODE SENSE, which allows an application to adapt to a library configuration instead of using fixed
values. Use of MODE SENSE to obtain these parameters during initialization is highly recommended to
facilitate the most flexibility in supporting the library.


## Mode Sense (6) CDB Format

The six-byte MODE SENSE CDB format is shown in the following table.

**Table 1: MODE SENSE CDB format**

```
              Bit        7           6              5           4             3        2          1           0
  Byte
          0                                                    Op Code (1Ah)
          1                           Reserved                            DBD                Reserved
          2                   PC                                              Page Code
          3                                                   SubPage Code
          4                                                  Allocation Length
          5                                                         Control
```

| Field | Description |
|---|---|
| Disable Block Descriptors (DBD) | A value of 0 or 1 is supported, although block descriptors are not returned. |
| Page Control (PC) | This field indicates the type of mode page parameter values to return. See Table 2 on the next page. |
| Page Code and Subpage Code | This field determines which pages should be reported. For a list of available mode pages, refer to Mode Pages on page 61. |
| Allocation Length | This field specifies the number of bytes that the initiator allocated for returned MODE SENSE data. A length of 0 means that the library will return no MODE SENSE data. This is not considered to be an error. |

**Table 2: Page Control (PC) field**

| Page Control | Description |
|---|---|
| 0 0 | Report current values defined by: The values set by the last successful MODE SELECT command. The default values if no saved values exist. |
| 0 1 | Report changeable. |
| 1 0 | Report default values. |
| 1 1 | Report saved values (report default values if no pages are previously saved). |

> **Note:** Although the library may support changes for various mode parameters via the library user
> interface configuration settings, a mode parameter will be reported as non-changeable if not
> supported to be changeable via SCSI control.


## Mode Sense (6) Response

The six-byte MODE SENSE response consists of a single four-byte Mode Parameter Header, followed by
zero or more mode pages. Each page is individually described in "Mode Pages."

### Mode Parameter Header

The following table lists the format of the Mode Parameter Header for the six-byte MODE SENSE
command.

**Table 3: Mode Parameter Header format for Mode Sense (6)**

```
          Bit      7               6         5            4            3           2            1            0
  Byte
      0                                                Mode Data Length
      1                                                   Medium Type
      2                                             Device-Specific Parameter
      3                                              Block Descriptor Length
```

| Field | Description |
|---|---|
| Mode Data Length | This specifies the length in bytes that is available to be transferred as part of the response. The Mode Data Length does not include itself but does include the remaining 3 bytes of the parameter header, as well as the overall total number of bytes being sent for all requested pages. |
| Medium Type | A Medium Type is not supported. This value is set to 0. |
| Device-Specific Parameter | A Device-Specific Parameter is not supported. This value is set to 0. |
| Block Descriptor Length | A Block Descriptor Length is not supported. This value is set to 0. |
