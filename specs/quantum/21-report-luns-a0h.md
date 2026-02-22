# Report LUNS - A0h

## What the Library Does With This Command

The library will return a list of the logical units that it supports. When this command is sent to a controller
device logical unit or tape drive hosting a media changer device logical unit, it will return a list of all
additional logical units that are available. This list will primarily be media changer devices representing
the configured Logical Libraries. When this command is sent to any of the media changer device logical
units, they will only report themselves.


## Command Usage

This command can be used to retrieve what Logical Unit Numbers are supported to avoid scanning for all
possible numbers. It is useful for identifying the various Logical Libraries that may be configured.


## Report LUNS CDB Format

The REPORT LUNS CDB format is shown in the following table.

**Table 1: REPORT LUNS CDB format**

```
              Bit        7           6              5      4             3   2          1         0
  Byte
         0                                               Op Code (A0h)
         1                                                     Reserved
         2                                                Select Report
         3                                                     Reserved
         4                                                     Reserved
         5                                                     Reserved
         6
         :                                              Allocation Length
         9
         10                                                    Reserved
         11                                                    Control
```

- **Select Report**: The Select Report specifies the addressing method used to report logical units to the I_T nexus. This field must be set to 00h to identify the simple logical unit addressing method.
- **Allocation Length**: This field must be set to a minimum of 10h (16 bytes) to retrieve information for at least a single LUN.


## Report LUNS Response

**Table 2: Report LUNS Response**

```
                Bit      7           6              5       4           3           2          1           0
  Byte
          0
           :                                            LUN List Length (n-7)
          3
          4
           :                                                    Reserved
          7
          8
           :                                                 First LUN
          15
          n-7
           :                                                 Last LUN
          n
```

- **LUN List Length**: This field returns the length in bytes of the list of LUNs being returned.
- **LUN**: These fields return each available assigned LUN. The information conforms to the Logical Unit Address Method defined in SCC-2, and supports only First Level addressing (for each LUN, only the second byte is used and contains the assigned LUN).
