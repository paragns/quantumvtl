# Exchange Medium - A6h

## What the Library Does With This Command

The EXCHANGE MEDIUM command is only supported by the Scalar i7 RAPTOR tape library.


## Command Usage

This command provides a means to replace the volume at an element address, typically a data transfer
element, with another volume. Support of this command requires that the library has 2 operational pickers
capable of handling two volumes at the same time.


## Exchange Medium CDB Format

The EXCHANGE MEDIUM CDB format is shown in the following table.

**Table 1: EXCHANGE MEDIUM CDB format**

```
             Bit        7            6          5         4         3           2     1           0
 Byte
         0                                                Op Code (A6h)
         1                                                    Reserved
         2           (MSB)
                                                    Medium Transport Access
         3                                                                                     (LSB)
         4           (MSB)
                                                         Source Address
         5                                                                                     (LSB)
         6           (MSB)
                                                     First Destination Access
         7                                                                                     (LSB)
         8           (MSB)
                                                    Second Destination Access
         9                                                                                     (LSB)

        10                                     Reserved                             INV1        INV2
        11                                                    Control
```

| Field | Description |
|---|---|
| Operation Code | Set to A6h to request an Exchange Medium command operation. |
| Medium Transport Address | Set to 0 or address of the SCSI Medium Changer element. |
| Source Address | The volume in the SOURCE ADDRESS element is moved to the FIRST DESTINATION ADDRESS element and the volume that previously occupied the FIRST DESTINATION ADDRESS element is moved to the SECOND DESTINATION ADDRESS element. The SECOND DESTINATION ADDRESS element may or may not be the same as the SOURCE ADDRESS element. In the case of a simple exchange, SOURCE ADDRESS and SECOND DESTINATION ADDRESS are the same. The Device Capabilities mode page provides a matrix that defines the supported source element type and first destination element type combinations for EXCHANGE MEDIUM commands when the source element type is the same as the second destination element type. |
| First Destination Address | See definition above. |
| Second Destination Address | See definition above. |
| INV1 | An INV1 bit set to one specifies that the volume shall be inverted prior to depositing the volume into the FIRST DESTINATION ADDRESS element. This bit must be set to 0. |
| INV2 | An INV2 bit set to one specifies that the volume shall be inverted prior to depositing the volume into the SECOND DESTINATION ADDRESS element. This bit must be set to 0 |

The volume in the SOURCE ADDRESS element is moved to the FIRST DESTINATION ADDRESS
element and the volume that previously occupied the FIRST DESTINATION ADDRESS element is
moved to the SECOND DESTINATION ADDRESS element. The SECOND DESTINATION ADDRESS
element may or may not be the same as the SOURCE ADDRESS element. In the case of a simple
exchange, SOURCE ADDRESS and SECOND DESTINATION ADDRESS are the same. The Device
Capabilities mode page provides a matrix that defines the supported source element type and first
destination element type combinations for EXCHANGE MEDIUM commands when the source element
type is the same as the second destination element type.
The SOURCE ADDRESS field and the SECOND DESTINATION ADDRESS field may represent a
storage element, an import/export element, a data transfer element. The FIRST DESTINATION
ADDRESS field should be a data transfer element and shall not be the same element type as the
SOURCE ADDRESS field.
When processing an EXCHANGE MEDIUM command, the library terminates the command with CHECK
CONDITION status, with the sense key set to ILLEGAL REQUEST and the additional sense code set to
MEDIUM REMOVAL PREVENTED BY DATA TRANSFER ELEMENT if:

a. the element address specified in the SOURCE ADDRESS field, or the FIRST DESTINATION
   ADDRESS field represents a data transfer element;
b. the library detects a prevention of medium removal condition exists within the data transfer device
   (see applicable command standard); and
c. the library does not allow moves from an element associated with a data transfer device that has a
   prevent medium removal condition.

When processing an EXCHANGE MEDIUM command, the library terminates the command with CHECK
CONDITION status, with the sense key set to ILLEGAL REQUEST and the additional sense code set to
MEDIUM REMOVAL PREVENTED if:

a. the element address specified in the FIRST DESTINATION ADDRESS field or the SECOND
   DESTINATION ADDRESS field represents an import/export element;
b. a prevention of medium removal condition exists within the medium changer; and
c. the MVPRV bit in the Extended Device Capabilities mode page set to one.

The Device Capabilities mode page provides a matrix with the supported source element or destination
element combinations for the EXCHANGE MEDIUM command. If the source element, first destination
element, and second destination element combination is not supported, then the command is terminated
with CHECK CONDITION status, with the sense key set to ILLEGAL REQUEST and the additional sense
code set to INVALID FIELD IN CDB.
