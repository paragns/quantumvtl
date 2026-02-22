# Mode Sense (10) - 5Ah

## What the Library Does With This Command

The library will return the current settings for the supported mode pages. This command can only be
issued to the controller device logical unit.


## Command Usage

This command can be used to determine certain operational settings governing the behavior of the
library. Use of MODE SENSE to obtain these parameters during initialization is highly recommended to
facilitate the most flexibility in supporting the library.


## Mode Sense (10) CDB Format

The ten-byte MODE SENSE CDB format is shown in the following table.

**Table 1: MODE SENSE CDB format**

```
              Bit       7          6           5          4              3          2        1          0
  Byte
          0                                              Op Code (5Ah)
          1                    Reserved                LLBAA            DBD             Reserved
          2                 PC                                          Page Code
          3                                              SubPage Code
          4
          5                                                   Reserved
          6
          7
                                                       Allocation Length
          8
          9                                                   Control
```

| Field | Description |
|---|---|
| Disable Block Descriptors (DBD) | A value of 0 or 1 is supported, although block descriptors are not returned. |
| Long LBA Accepted (LLBAA) | If the Long LBA Accepted (LLBAA) bit is set to one, the device is allowed to return mode parameter header data with the LONGLBA bit equal to one. If LLBAA bit is set to zero, the LONGLBA bit shall be zero in the mode parameter header data returned by the device. |
| Page Control (PC) | This field indicates the type of mode page parameter values to return as shown in the following table. See Table 2 below. |
| Page Code | This field determines which pages should be reported. A list of the supported pages is shown in the previous table. |
| Subpage Code | This field determines which subpage codes should be reported. |
| Allocation Length | This field specifies the number of bytes that the initiator allocated for returned MODE SENSE data. A length of 0 means that the library will return no MODE SENSE data. This is not considered to be an error. |

**Table 2: Page Control (PC) field**

| Page Control | Description |
|---|---|
| 0 0 | Report current values defined by: The values set by the last successful MODE SELECT command. The default values if no saved values exist. |
| 0 1 | Report changeable values. |
| 1 0 | Report default values. |
| 1 1 | Report saved values (report default values if no pages are previously saved). |

> **Note:** Although the library may support changes for various mode parameters via the library user
> interface configuration settings, a mode parameter will be reported as non-changeable if not
> supported to be changeable via SCSI control.


## Mode Sense (10) Response

The ten-byte MODE SENSE response consists of a single eight-byte Mode Parameter Header, followed
by zero or more mode pages. Each page is individually described in "Mode Pages."

### Mode Parameter Header

The following table shows the format of the Mode Parameter Header for the ten-byte MODE SENSE
command.

**Table 3: Mode Parameter Header format for Mode Sense (10)**

```
          Bit      7            6            5            4              3       2            1           0
  Byte
     0
                                                       Mode Data Length
     1
     2                                                    Medium Type
     3                                              Device-Specific Parameter
     4
                                                              Reserved
     5
     6
                                                     Block Descriptor Length
     7
```

| Field | Description |
|---|---|
| Mode Data Length | This specifies the length in bytes that is available to be transferred as part of the response. The Mode Data Length does not include itself but does include the remaining six bytes of the parameter header, as well as the overall total number of bytes being |
| Medium Type | A Medium Type is not supported. This value is set to 0. |
| Device-Specific Parameter | A Device-Specific Parameter is not supported. This value is set to 0. |
| Block Descriptor Length | A Block Descriptor Length is not supported. This value is set to 0. |
