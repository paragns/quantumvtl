# Read Element Status - B8h

## What the Library Does With This Command

The library returns current status and information regarding the requested elements. The data is primarily
derived from having done an Inventory operation (using INITIALIZE ELEMENT STATUS or INITIALIZE
ELEMENT STATUS WITH RANGE), but in the case of data transfer elements is also augmented by
communication with the drives. Element status remains valid as long as the subsystem integrity has not
been breached, such as by opening a door or through a power cycle.

Element status will be reported for all elements, including those represented by only a placeholder, as in
the case of uninstalled drives or magazines that physically have a place reserved in the configuration. As
such, it is important to process the fields governing accessibility and exception conditions.


## Command Usage

This command should be issued whenever new element status information is needed, or the library has
indicated that status may have changed. If the status information is suspect, an INITIALIZE ELEMENT
STATUS WITH RANGE command should be issued to refresh it.


## Read Element Status CDB Format

The READ ELEMENT STATUS CDB format is shown in the following table.

**Table 1: READ ELEMENT STATUS CDB format**

```
             Bit    7          6         5            4          3          2         1           0
  Byte
       0                                                  Op Code (B8h)
       1                    Reserved                VolTag                  Element Type Code
       2
                                                    Starting Element Address
       3
       4
                                                      Number of Elements
       5
       6                                     Reserved                             CurData       DVCID
       7
         :                                              Allocation Length
       9
      10                                                     Reserved
      11                                                      Control
```

| Field | Description |
|-------|-------------|
| Volume Tag (VolTag) | This field indicates whether the volume tag (bar code label) information should be returned. A value of one will return the labels, a value of zero will not. |
| Element Type Code | This field specifies the element types selected for the returned information, as shown in Table 2 on the next page. |
| Starting Element Address | This field specifies the minimum element address to report. Only elements with an element type code specified by the Element Type Code field, and with an address greater than or equal to the starting element address will be reported. The starting element address must be a valid element address, but not have to be within the range specified by the Element Type Code field. |
| Number of Elements | This field specifies the maximum number of element descriptors to return. Only those descriptors that can be completely transferred within the allotted allocation length will be returned. |
| Current Data (CurData) | This field specifies whether the library may cause device motion to confirm element status data. The library will not cause device motion if this field is set to either 0 or 1. |
| Device ID (DVCID) | This field indicates whether device identifiers (inquiry page information or serial numbers) are returned for the specified range. Identifiers are returned if this field is set to 1. They are not returned if this field is set to 0. Only data transfer elements can return device identifiers. |
| Allocation Length | This field specifies the byte length allowed for returned element descriptors. Only complete element descriptors are returned. The library returns element descriptors until one of the following conditions are met: |
| | - All available element descriptors have been returned |
| | - The number of element descriptors specified in the Number of Elements field have been returned |
| | - The number of bytes specified in the Allocation Length field have been returned |
| | - There is less allocation length space available than is required by the next complete element descriptor |

**Table 2: Element Type Code**

| Code | Selected Element Type |
|------|----------------------|
| 0000b (0) | All element types reported |
| 0001b (1) | Medium transport element (accessor) |
| 0010b (2) | Storage element |
| 0011b (3) | Import/Export element |
| 0100b (4) | Data transfer element (drives) |


## Read Element Status Response

Element status data consists of an eight-byte header, followed by one or more element status pages (per
element type). Each element status page consists of a header, followed by one or more element
descriptor blocks. A complete response then looks like:

```
                  Element Status Header
                             Element Status Page Header (first element type)
                                    Element Descriptor
                                    ...(more descriptors)...
                                    Element Descriptor
                             ...(more status pages)...
                             Element Status Page Header (next element type)
                                    Element Descriptor
                                    ...
                                    Element Descriptor
```

There are only up to four Element Status Pages, one for each element type.

### Element Status Header

One header is returned for each READ ELEMENT STATUS command. The format is shown in the
following table.

**Table 3: Element Status Header format**

```
                Bit      7           6              5        4         3        2    1    0
  Byte
         0
                                                    First Element Address Reported
         1
         2
                                                      Number of Elements Available
         3
         4                                                      Reserved
         5
         :                                      Byte Count of Report Available
         7
```

| Field | Description |
|-------|-------------|
| First Element Address Reported | This field indicates the lowest element address found that meets the CDB request. |
| Number of Elements Available | This field indicates the number of elements found that meet the CDB request. |
| Byte Count of Report Available | This field indicates the number of available element status bytes that meet the CDB requirements. The value does not include the eight-byte element status header, and is not adjusted to match the value specified in the Allocation Length field of the CDB. This facilitates first issuing a READ ELEMENT STATUS command with an allocation length of eight bytes in order to determine the allocation length required to transfer all the element status data specified by the command. |

### Element Status Page

Each element status page consists of an eight-byte header, followed by one or more element descriptor
blocks. One Element Status Page header is returned for each grouping of element descriptor blocks, by
element type. The format of the Element Status Page header is shown in the following table.

**Table 4: Element Status Page**

```
                 Bit            7           6              5          4         3          2           1       0
  Byte

         0                                 Reserved                                 Element Type Code
         1                   PVolTag   AVolTag                                  Reserved
         2
                                                      Element Descriptor Length
         3
         4                                                Reserved
         5
          :                             Byte Count of Descriptor Data Available
         7
```

| Field | Description |
|-------|-------------|
| Element Type Code | This field indicates the specific element type being returned by the element descriptors for this page. |
| PVolTag | A value of one indicates that the primary volume tag field (barcode label) is present in each of the element descriptor blocks that follow. A value of zero indicates that they are not present. |
| AVolTag | Alternate Volume Tags are not supported. The returned value for this field is 0, and the alternate volume tag fields are omitted from the element descriptors. |
| Element Descriptor Length | This field indicates the number of bytes contained in a single element descriptor. Refer to the individual element descriptor descriptions for each element type for their respective possible lengths. |
| Byte Count of Descriptor Data Available | This field indicates the number of element descriptor data bytes available for the elements of this element type that meet the CDB requirements. This value represents the Element Descriptor Length field multiplied by the number of element descriptors for this element type. This value does not include the 8-byte Element Status Page header, nor is it adjusted to match the allocation length. |


### Element Descriptors

The following sections contain the definitions for the following element descriptors:
- Medium transport element
- Storage elements
- Import/Export elements
- Data transfer elements

Each element descriptor includes the element address, status flags, source storage element address,
and barcode label. Some descriptors also contain extended status information. Additional sense code
and qualifier information depends on the element type.

### Primary Volume Tag Field

Volume tags (returned in the Primary Volume Tag field) are basically barcode labels on the media. The
library supports labels from 5 to 16 characters in length. The Primary Volume Tag field contains 32 bytes
of label data (space filled to 32 bytes), followed by two reserved bytes, then two bytes of volume
sequence number. The library returns zeros for the last four bytes of Primary Volume Tag data.

If the user has configured the library to support media identification, media identifiers will be reported as
configured per partition barcode reporting setting.

### Medium Transport Element Descriptor

**Table 5: Primary Volume Tag Field**

```
           Bit       7              6           5           4         3           2            1          0
  Byte
      0
                                                        Element Address
      1
      2                      Element Descriptor Length                         Except        Rsvd       Full
      3                                                        Reserved
      4                                              Additional Sense Code
      5                                       Additional Sense Code Qualifier
      6
       :                                                       Reserved
      8
      9           Svalid         Invert             Reserved         ED                 Medium Type
      10
                                             Source Storage Element Address
      11
      12
                                              Primary Volume Tag Information
       :
                                (Field omitted if PVolTag = 0; remaining fields move up)
      47
      48
       :                                                       Reserved
      51
```

| Field | Description |
|-------|-------------|
| Element Address | This field contains the element address of the medium transport. |
| Except | This field is set to 1 if the element is in an abnormal state. Additional information will be available in the Additional Sense Code and Additional Sense Code Qualifier fields. This field is set to 0 if the element is in a normal state. |
| Full | This field is set to 0 if the element does not contain media. It is set to 1 if it does. Since the medium transport element cannot be a destination element, this field should normally return 0. There may be error situations where media is left in the picker, which would be indicated by this field. In the case of dual pickers, differentiation of state will be provided through the Additional Sense Code and Additional Sense Code Qualifier fields when exception conditions are present (such as stranded media). |
| | > **Note:** Since the medium transport element is virtualized, any media being transported will not be reported doing a move operation and will be reported in either its source or destination element while being moved. |
| Additional Sense Code | If the element is in an abnormal state (an exception associated with it), this field will be set to a value as described in Table 4 on page 149. |
| Additional Sense Code Qualifier | If the element is in an abnormal state, this field will be set to a value as described in Table 4 on page 149. |
| Source Valid (Svalid) | This field is set to 1 if the Source Storage Element Address field is valid, otherwise it is set to 0. |
| | > **Note:** Since the medium transport element is virtualized, any media being transported will not be reported doing a move operation and will be reported in either its source or destination element while being moved. |
| Invert | This field is set to 0. The library does not support inverting media. |
| Element Disabled (ED) | An element disabled (ED) bit set to zero indicates that the element is not disabled. An ED bit set to one indicates that the element is disabled (e.g., medium changer / robot is varied off or not installed). |
| Medium Type | This field is set to 000b to indicate that the medium changer is not identifying any medium type |
| Source Storage Element Address | If the Source Valid field is set to 1, this field will contain the element address of the last storage element the media was in. Since the medium transport element cannot be a destination element, this would be an abnormal condition. |
| | > **Note:** Since the medium transport element is virtualized, any media being transported will not be reported doing a move operation and will be reported in either its source or destination element while being moved. |
| Primary Volume Tag | This field will normally return spaces if the primary volume tag is requested, since the medium transport element is virtualized. In certain error situations, a volume tag will be returned to indicate which cartridge may be stranded within the picker. |
| | > **Note:** Since the medium transport element is virtualized, any media being transported will not be reported doing a move operation and will be reported in either its source or destination element while being moved. |

### Storage Element Descriptor

**Table 6: Storage Element Descriptor**

```
          Bit      7             6            5          4              3            2            1          0
  Byte
     0
                                                      Element Address
     1
     2                          Reserved                          Access         Except        Rsvd        Full
     3                                                       Reserved
     4                                              Additional Sense Code
     5                                       Additional Sense Code Qualifier
     6
      :                                                      Reserved
     8

     9          Svalid         Invert          Reserved              ED                  Medium Type
     10
                                            Source Storage Element Address
     11
     12
                                            Primary Volume Tag Information
        :
                               (Field omitted if PVolTag = 0; remaining fields move up)
     47
     48
        :                                                      Reserved
     51
```

| Field | Description |
|-------|-------------|
| Element Address | This field contains the element address of the storage element. |
| Access | This field is set to 1 if access by a medium transport element is allowed. It is set to 0 if access is denied. |
| Except | This field is set to 1 if the element is in an abnormal state. Additional information may be available in the Additional Sense Code and Additional Sense Code Qualifier fields. If this field is 1, the primary volume tag information could be invalid. This field is set to 0 if the element is in a normal state. |
| Full | This field is set to 0 if the element does not contain media. It is set to 1 if it does. |
| Additional Sense Code | If the element is in an abnormal state, this field will be set to a value as described in Table 4 on page 149. |
| Additional Sense Code Qualifier | If the element is in an abnormal state, this field will be set to a value as described in Table 4 on page 149. |
| Source Valid (Svalid) | This field is set to 1 if the Source Storage Element Address (media home slot) field is valid, otherwise it is set to 0. |
| Invert | This field is set to 0. The library does not support inverting media. |
| Element Disabled (ED) | An element disabled (ED) bit set to zero indicates that the element is not disabled. An ED bit set to one indicates that the element is disabled (e.g., magazine not installed). |
| Medium Type | This field is set to one of the following to indicate the type of medium residing at the element location: |
| | 000b -- Unspecified medium type |
| | 001b -- Cleaning tape |
| | 011b -- Diagnostic Tape |
| | 100b -- WORM Medium |
| | 101b -- Microcode image Medium |
| Source Storage Element Address | If the Source Valid field is set to 1, this field will contain the element address of the last storage element the media was moved from. It may be the same as this element. |
| Primary Volume Tag | If requested, this field contains the volume tag (bar code label) information for media residing in this element address. |

### Import/Export Element Descriptor

**Table 7: Import/Export Element Descriptor**

```
     Bit         7            6             5              4              3           2              1           0
  Byte
     0
                                                       Element Address
     1
     2          OIR         CMC        InEnab         ExEnab        Access        Except        Imp/Exp         Full
     3                                                         Reserved
     4                                              Additional Sense Code
     5                                          Additional Sense Code Qualifier
     6
     :                                                         Reserved
     8
     9        Svalid        Invert              Reserved               ED                   Medium Type
     9        Svalid        Invert                                        Reserved
    10
                                            Source Storage Element Address
    11
    12
                                              Primary Volume Tag Information
     :
                              (Field omitted if PVolTag = 0; remaining fields move up)
    47
    48
     :                                                     Reserved
    51
```

All fields are the same as for the Storage Element Descriptor except:

| Field | Description |
|-------|-------------|
| Element Address | This field contains the element address of the import/export element. |
| OIR | An operator intervention required (OIR) bit set to one indicates operator intervention is required to make the import/export element accessible. An OIR bit set to zero indicates that operator intervention is not required to make the import/export element accessible. |
| CMC | A connected media changer (CMC) bit is set to 0, indicating that exports are to the operator and imports are from the operator. |
| Import Enable (InEnab) | A value of one indicates that the element supports movement of media into the scope of the media changer device. A value of zero indicates that this element does not support import actions. The library returns a value of one for all import/export elements. |
| Access | This field is set to 1 if access by a medium transport element is allowed. It is set to 0 if access is denied. |
| Except | This field is set to 1 if the element is in an abnormal state. Additional information may be available in the Additional Sense Code and Additional Sense Code Qualifier fields. If this field is 1, the primary volume tag information could be invalid. This field is set to 0 if the element is in a normal state. |
| Export Enable (ExEnab) | A value of one indicates that the element supports movement of media out of the scope of the media changer device. A value of zero indicates that this element does not support export actions. The library returns a value of one for all import/export elements. |
| Import/Export (ImpExp) | A value of one indicates that media present in the element was placed there by an operator. A value of zero indicates that media present in the element was placed there by a medium transport element. |
| Full | This field is set to 0 if the element does not contain media. It is set to 1 if it does. |
| Additional Sense Code | If the element is in an abnormal state, this field will be set to a value as described in Table 4 on page 149. |
| Additional Sense Code Qualifier | If the element is in an abnormal state, this field will be set to a value as described in Table 4 on page 149. |
| Source Valid (Svalid) | This field is set to 1 if the Source Storage Element Address (media home slot) field is valid, otherwise it is set to 0. |
| Invert | This field is set to 0. The library does not support inverting media. |
| Element Disabled (ED) | An element disabled (ED) bit set to zero indicates that the element is not disabled. An ED bit set to one indicates that the element is disabled (e.g., magazine removed and not installed). |
| Medium Type | This field is set to one of the following to indicate the type of medium residing at the element location: |
| | 000b -- Unspecified medium type |
| | 001b -- Cleaning tape |
| | 011b -- Diagnostic Tape |
| | 100b -- WORM Medium |
| | 101b -- Microcode image Medium |
| Source Storage Element Address | If the Source Valid field is set to 1, this field will contain the element address of the last storage element the media was moved from. It may be the same as this element. |
| Primary Volume Tag | If requested, this field contains the volume tag (bar code label) information for media residing in this element address. |

### Data Transfer Element Descriptor

**Table 8: Data Transfer Element Descriptor**

```
    Bit          7             6               5              4               3             2           1         0
  Byte
    0
                                                     Element Address
    1
    2                              Reserved                               Access        Except        Rsvd      Full
    3                                                        Reserved
    4                                                Additional Sense Code
    5                                        Additional Sense Code Qualifier
    6       Obsolete        Reserved       Obsolete         Obsolete       Reserved                Obsolete
    7                                                         Obsolete
    8                                                        Reserved

    9        SValid          Invert                 Reserved                   ED                 Medium Type
    10
                                                    Source Element Address
    11
    12
                                             Primary Volume Tag Information
     :
                               (Field omitted if PVolTag = 0; remaining fields move up)
    47
    48                        Protocol Identifier                                          Code Set
    49         PIV          Reserved               Association                          Identifier Type
    50                                                       Reserved
    51                      Identifier Length = x where x is 0h to 40h (valid identifier data)
    52                                                   Device Identifier
     :                          (Field omitted if DVCID = 0, remaining fields move up)
   115                                (Always padded to 64 byte length if DVCID = 1)
```

| Field | Description |
|-------|-------------|
| Element Address | This field contains the element address of the data transfer element. |
| Access | This field is set to 1 if access by a medium transport element is allowed. It is set to 0 if access is denied. When set to 1, it implies that cartridges are unloaded and accessible if present. When set to 0, it implies that cartridges are not unloaded if present. |
| Except | This field is set to 1 if the element is in an abnormal state. Additional information may be available in the Additional Sense Code and Additional Sense Code Qualifier fields. If this field is 1, the primary volume tag information could be invalid. This field is set to 0 if the element is in a normal state. |
| Full | This field is set to 0 if the element does not contain media. It is set to 1 if it does. |
| Additional Sense Code | If the element is in an abnormal state, this field will be set to a value as described in "Additional Sense Codes and Qualifiers." |
| Additional Sense Code Qualifier | If the element is in an abnormal state, this field will be set to a value as described in "Additional Sense Codes and Qualifiers." |
| Not This Bus (NotBus) | This field is not supported and is set to 0. |
| IDValid | A value of one indicates that the SCSI Bus Address field is valid. A value of zero indicates that it is not. |
| LUValid | This field is not supported and is set to 0. |
| Logical Unit Number | This field is not supported and is set to 0. |
| SCSI Bus Address | When the IDValid field is set to one, this field contains the tape drive SCSI address. This is only applicable to SCSI tape drives, and does not apply to Fibre Channel tape drives. |
| Source Valid (Svalid) | This field is set to 1 if the Source Storage Element Address (media home slot) field is valid, otherwise it is set to 0. |
| Invert | This field is set to 0. The library does not support inverting media. |
| Element Disabled (ED) | An element disabled (ED) bit set to zero indicates that the element is not disabled. An ED bit set to one indicates that the element is disabled (e.g., data transfer device is not installed or varied off). |
| Medium Type | This field is set to one of the following to indicate the type of medium residing at the element location: |
| | - 000b -- Unspecified medium type |
| | - 001b -- Cleaning tape |
| | - 011b -- Diagnostic Tape |
| | - 100b -- WORM Medium |
| | - 101b -- Microcode image Medium |
| Source Storage Element Address | If the Source Valid field is set to 1, this field will contain the element address of the last storage element the media was moved from. |
| Primary Volume Tag | If requested, this field contains the volume tag (bar code label) information for media residing in this element address. |
| Protocol Identifier | This field identifies the Data Transfer SCSI transfer protocol associated with the Data Transfer element location. Set to one of the following: |
| | - 0h - Fibre Channel Protocol for SCSI (FCP-4). |
| | - 6h - SAS Serial Protocol (SPL-4). |
| | - Fh - No specific protocol identified. |
| Code Set | This field is set to: |
| | - 0h -- RESERVED. |
| | - 1h -- The device identifier field contains binary values. |
| | - 2h -- The device identifier field contains ASCII values. |
| PIV | The protocol identifier valid (PIV) bit set to 0 indicates the Protocol Identifier field contents is reserved. If set to 1, then the Association field is set to either a value of 01b or 10b, indicating the protocol identifier field contains a valid protocol identifier. |
| Association | The Association field indicates the entity with which the Identifier field is associated. This field is set to 10b to indicate the identifier field is associated with the SCSI target device that contains the addressed logical unit. |
| Identifier Type | This field is set to: |
| | - 0h -- The Device Identifier, if the Identifier Length is set, lists the vendor specific device serial number only. |
| | - 1h -- The Device Identifier lists the eight-byte Vendor Identification, followed by vendor specific unique identifier information. |
| | - 2h -- The Device Identifier contains a Canonical form of IEEE Extended Unique Identifier, 64-bit (EUI-64). In this case, the Identifier Length field is set to 8. |
| | - 3h -- The Device Identifier contains an FC-PH Name_identifier. |
| Identifier Length | This field contains the length in bytes of valid Device Identifier information. If no device identifier is available, or the DVCID bit in the CDB is zero, the Identifier Length field is 0h and the Code Set and Identifier Type fields are also 0h. If the DCVID bit is set, the Identifier Length may be set between 0 and 64 (40h) bytes, depending on the associated identifier type. |
| Device Identifier | This field provides up to 64 bytes of device identifier information for the device associated with the data transfer element. |
| | - If the Identifier Type 1 is specified, drive's Inquiry page 83h is reported. |
| | - The Identifier Length specifies the length of valid device identifier information. |
| | - If the DVCID bit in the CDB is zero, this field is omitted and padded with with ASCII character 20h (space) to fill the complete 64 bytes. |
| | - If the DVCID bit is set and the Identifier Length is 0, this field will still be 64 bytes long. |
