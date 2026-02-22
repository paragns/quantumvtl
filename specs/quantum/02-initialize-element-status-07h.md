# Initialize Element Status - 07h

## What the Library Does With This Command

The library will determine element status (full or empty) as well as barcode label information (volume
tags) for the media. The library will accept and respond with good SCSI status without performing the
operation. The library establishes inventory status per the Automatic Inventory setting (enabled or
disabled).
If the Automatic Inventory option is enabled, the element status will be established upon each library
initialization procedure (library power-up).
If Automatic Inventory option is disabled, the element status is assumed to be correct or correct via library
management inventory functionality.
Results of the status initialization will be buffered by the library for retrieval via the READ ELEMENT
STATUS command. Element status and barcode label information is retained by the library across power
cycles.


## Command Usage

This command can be used to gather status for all the elements, and should be issued whenever the
library indicates that element status may have changed, such as after a power cycle or door opening and
closing. It should then be followed by a READ ELEMENT STATUS command to retrieve the status.


## Initialize Element Status CDB Format

The INITIALIZE ELEMENT STATUS CDB format is shown in the following table.

**Table 1: INITIALIZE ELEMENT STATUS CDB format**

```
                Bit        7         6              5    4          3       2        1         0
 Byte
          0                                             Op Code (07h)
          1                                                  Reserved
          2                                                  Reserved
          3                                                  Reserved
          4                                                  Reserved
          5              NBL                                      Control
```

| Field | Description |
|---|---|
| No Barcode Labels (NBL) | A value of 0 indicates that the specified elements will be checked for all relevant status, including bar code labels. A value of 1 indicates that elements will be checked for media presence only (no bar code labels). |

> **Note:** Tape libraries that have barcode label scanners/readers installed
> will always establish barcode label element status, regardless of the
> NBL bit setting.
