# Initialize Element Status With Range - E7h/37h

## What the Library Does With This Command

The library will examine the range of elements requested and determine their status relative to media
presence (full or empty). Barcode labels will be scanned unless otherwise directed (and the library
supports a non-barcode option). The library may not fully execute this command if the Automatic
Inventory option is enabled, and element status is already known.
Results of the status initialization will be buffered by the library for retrieval via the READ ELEMENT
STATUS command. Element status and barcode label information is retained by the library across power
cycles.


## Command Usage

This command can be issued to gather status for some or all of the elements, and can be used in
conjunction with host application error handling if the normal element status maintained by the library
returns an unexpected result. It should then be followed by a READ ELEMENT STATUS command to
retrieve the status.


## Initialize Element Status With Range CDB Format

The INITIALIZE ELEMENT STATUS WITH RANGE CDB format is shown in the following table.

**Table 1: INITIALIZE ELEMENT STATUS WITH RANGE CDB format**

```
               Bit       7           6              5           4         3            2      1      0
 Byte
         0                                                   Op Code (E7h or 37h)

         1                                              Reserved                           Fast   Range
         2
                                                            Starting Element Address
         3
         4                                                          Reserved
         5                                                          Reserved
         6
                                                              Number of Elements
         7
         8                                                          Reserved

         9              NBL                                              Control
```

| Field | Description |
|---|---|
| Range | A value of 0 indicates that all element addresses will be checked and that the Starting Element Address and Number of Elements fields will be ignored. A value of 1 indicates that the series of elements beginning at the specified Starting Element Address for the specified Number of Elements will be checked. |
| Fast | A FAST bit set to one specifies that the specified elements shall be scanned for media presence only. A FAST bit set to zero specifies that the specified elements shall be scanned for all relevant status. |
| Starting Element Address | The Starting Element Address specifies the beginning address of the range to check. It must be a valid address for an element that exists within the library; no adjustment will be made to convert to a next higher valid address. This field is ignored if the Range field is 0. |
| Number of Elements | This field specifies the number of elements to check. Gaps in element types and addresses are automatically handled until a quantity of physical elements equal to this number has been checked. If this field is 0, the range checked will start with the Starting Element Address and continue through all remaining elements. This field is ignored if the Range field is 0. |
| No Barcode Labels (NBL) | A value of 0 indicates that the specified elements will be checked for all relevant status, including bar code labels. A value of 1 indicates that elements will be checked for media presence only (no bar code labels). |

> **Note:** This bit is requesting the same operation as the vendor specific
> NBL bit in the control byte. Tape libraries that have barcode label
> scanners/readers installed will always establish barcode label
> element status, regardless of the FAST bit setting.

> **Note:** Tape libraries that have barcode label scanners/readers
> installed will always establish barcode label element status,
> regardless of the NBL bit setting.
