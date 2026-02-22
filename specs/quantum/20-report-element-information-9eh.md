# Report Element Information - 9Eh

- **LongID**: This field is not supported, and must be set to 0 as IDs greater than 255 are not supported.
- **Third Party Device ID**: This field is required and used only when the 3rdPrty bit is set. Since the 3rdPrty bit is not supported, this field must be set to 0.
- **Parameter List Length**: This field is not supported, and must be set to 0.


## What the Library Does With This Command

The REPORT ELEMENT INFORMATION command requests information pages that describe an
element or a set of elements. The library does not report element information for elements that are not
associated with an element address.


## Command Usage

This command should be used to determine element status and state information and/or physical location
coordinates association with an element's SCSI element address identification.


## Report Element Information CDB Format

The POSITION TO ELEMENT CDB format is shown in the following table.

**Table 1: REPORT ELEMENT INFORMATION CDB format**

```
              Bit        7              6           5         4              3            2            1       0
  Byte
          0                                                 Op Code (9Eh)

          1                      Reserved                                        Sevice Action (10h)
          2                                                   Page Code
          3                  Reserved            NEV       CDATA                       Element Type Code
          4
                                                                  Reserved
          5
```

```
              Bit       7            6             5             4             3         2            1           0
  Byte
          6           (MSB)
          7
                                                            Starting Element Address
          8
          9                                                                                                      (LSB)
          10          (MSB)
          11
                                                               Allocation Length
          12
          13                                                                                                     (LSB)
          14                                                  Number of Elements
          15                                                          Control
```

- **Page Code**: This field specifies the element information page requested by the application client. If this field set to an unsupported value, the library terminates the command with Check Condition status, with the sense key set to Illegal Request and the additional sense code set to Invalid Field in CBS.
  - 00h - Supported Element Information Pages
  - 02h - Element Static Information
  - 03h - Element State Information
  - 04h - Element Location Information
  - 7Fh - Return All Supported Pages
- **NEV**: A number of elements valid (NEV) bit set to one specifies that the NUMBER OF ELEMENTS field is valid and the specified number of elements are selected for reporting. An NEV bit set to zero specifies that the NUMBER OF ELEMENTS field is not valid and all elements may be selected for reporting. If the PAGE CODE field is set to 00h (i.e., Supported Element Information Pages information page), then the NEV bit is set to zero.
- **CDATA**: A cached data (CDATA) bit set to one specifies that the library shall immediately return the requested element information page using cached discovery and inventory information. A CDATA bit set to zero specifies that the library may update discovery or inventory information (e.g., perform a discovery or inventory scan).
- **Element Type Code**: This field indicates the element type code for the element type that supports the list of pages that follows.
  - 0000b (0) - All element types reported
  - 0001b (1) - Medium transport element (accessor)
  - 0010b (2) - Storage element
  - 0011b (3) - Import/Export element
  - 0100b (4) - Data transfer element (drives)
- **Starting Element Address**: This field specifies the lowest element address to report. Only elements with an element type code selected by the Element Type Code field and an element address greater than or equal to the value specified in the Starting Element Address field shall be selected for reporting. If the Page Code field is set to 00h, then the Starting Element Address field is ignored.
- **Number of Elements**: This field specifies the maximum number of elements to be selected for reporting. Elements not associated with an element address will not be counted in the number of elements reported. Elements selected for reporting are reported once in ascending order by element address.

The OPERATION CODE byte and SERVICE ACTION field shall be set to the values shown above. The
PAGE CODE field specifies the element information page that is requested by the application. The
following page codes apply.

**Table 2: Element Information Page Codes**

| Page Code | Definition |
|-----------|------------|
| 00h | Supported Element Information Pages |
| 02h | Element Static Information |
| 03h | Element State Information |
| 04h | Element Location Information |
| 7Fh | Return All Supported Pages |

A number of elements valid (NEV) bit set to one specifies that the NUMBER OF ELEMENTS field is valid
and the specified number of elements are selected for reporting. An NEV bit set to zero specifies that the
NUMBER OF ELEMENTS field is not valid and all elements may be selected for reporting. If the PAGE
CODE field is set to 00h (i.e., Supported Element Information Pages information page), then the NEV bit
is set to zero.

A cached data (CDATA) bit set to one specifies that the library immediately return the requested element
information page using cached discovery and inventory information. A CDATA bit set to zero specifies
that the library may update discovery or inventory information (e.g., perform a discovery or inventory
scan).

The STARTING ELEMENT ADDRESS field specifies the lowest element address to report. Only
elements with an element type code selected by the ELEMENT TYPE CODE field, see Element Type
Codes, and an element address greater than or equal to the value specified in the STARTING ELEMENT
ADDRESS field is selected for reporting. If the PAGE CODE field is set to 00h (i.e., Supported Element
Information Pages), then the STARTING ELEMENT ADDRESS field will be ignored.

The NUMBER OF ELEMENTS field specifies the maximum number of elements to be selected for
reporting. Elements not associated with an element address are not counted in the number of elements
reported. Elements selected for reporting are reported once in ascending order by element address.


## Supported Element Information Page (00h)

The Supported Element Information Pages element information page returns the list of element
information pages supported by the device server for the element type specified in the REPORT
ELEMENT INFORMATION command. If all element types are specified, then the library shall return one
supported pages descriptor for each supported element type.

**Table 3: Supported Element Information Pages element information page**

```
             Bit         7            6             5         4           3    2           1      0
  Byte
         0                                                 Page Code (00h)
         1
                                                               Reserved
         3
         4            (MSB)
                                                           Page Length (n-7)
         7                                                                                      (LSB)

                                          Supported element information page descriptors
```

```
              Bit        7            6              5            4           3           2           1           0
  Byte
          8
                                          Supported element information page descriptors (first)

                                                                       ...


                                          Supported element information page descriptors (last)
          n
```

- **Page Length**: This field indicates the length in bytes of the supported element information pages descriptors that follow. If the descriptors are truncated because of the allocation length, then the PAGE LENGTH field will not be affected.
- **Supported element information page descriptors**: Refer to Support Element Information Descriptors below.


## Support Element Information Page Descriptors

One supported element information pages descriptor will be returned for each selected element type.
Supported element information pages descriptors shall be returned in ascending order by element type
code.

**Table 4: Supported element information pages descriptor**

```
              Bit        7            6              5            4           3           2           1           0
  Byte
          0                               Reserved                                      Element Type Code
          1                                                        Reserved
          2           (MSB)
                                                          Page Code List Length (n-3)
          3                                                                                                     (LSB)
          4
                                              Supported element information page code list
          n
```

- **Element Type Code**: This field indicates the element type code for the element type that supports the list of pages that follows.
  - 0000b (0) - All element types reported
  - 0001b (1) - Medium transport element (accessor)
  - 0010b (2) - Storage element
  - 0011b (3) - Import/Export element
  - 0100b (4) - Data transfer element (drives)
- **Page Code List Length**: This field is the length in bytes of the supported element information page code list. If the descriptor is truncated because of the allocation length, then the PAGE CODE LIST LENGTH field will not be affected.
- **Supported element information page code list**: This list contains a list of element information page codes implemented by the device server for the specified element type code in ascending order beginning with page code 00h.
  - 00h - Supported Element Information Pages
  - 02h - Element Static Information
  - 03h - Element State Information
  - 04h - Element Location Information
  - 7Fh - Return All Supported Pages


## Element Static Information Page (02h)

The Element Static Information element information page returns a set of element characteristics that are
defined as static. Any change to an element's static configuration setting will establish a Unit Attention
condition with additional sense code set to ELEMENT STATIC INFORMATION CHANGED.

**Table 5: Element Static Information element information page**

```
              Bit        7            6             5             4           3           2          1          0
  Byte
          0                                                    Page Code (02h)
          1                                                       Reserved
          2
                                                               Descriptor Length
          3
```

```
              Bit         7             6               5              4             3        2           1            0
  Byte
          4            (MSB)
                                                                  Page Length (n-7)
          7                                                                                                          (LSB)

                                                 Element static information descriptors

          8
                                                 Element static information descriptor (first)

                                                                           ...

                                                 Element static information descriptor (last)
          n
```

- **Descriptor Length**: This field indicates the length of each element static information descriptor. The descriptor length will be a multiple of 4 bytes long. The element static information descriptors will be zero padded.
- **Page Length**: This field is the length in bytes of the element static information descriptors that follow. If the descriptors are truncated because of the allocation length, then the PAGE LENGTH field will not be affected.
- **Element static information descriptors**: Refer to Element Static Information Descriptors below.


## Element Static Information Descriptors

Element static information descriptors will be returned in the format below.

**Table 6: Element static information descriptor**

```
              Bit        7            6             5              4             3           2               1             0
  Byte
          0           (MSB)
                                                            First Element Address Reported
          3                                                                                                             (LSB)
          4           (MSB)
                                                            Number of Elements Reported
          7                                                                                                             (LSB)
          8                             Reserved                                                  Reserved
```

```
              Bit       7              6            5           4            3          2                1            0
  Byte
          9                 Reserved            SO             DO           NXP       MDO            IESTOR       EDC
         10
                                                                    Reserved
          n
```

- **First Element Address Reported**: This field indicates the lowest element address being reported.
- **Number of Elements Reported**: This field indicates the number of elements with contiguous element addresses starting from the element address indicated by the FIRST ELEMENT ADDRESS REPORTED field with the same:
  - ELEMENT TYPE CODE field value
  - SO bit value
  - DO bit value
  - NXP bit value
  - MDO bit value
  - IESTOR bit value
  - EDC bit value
- **Element Type Code**: This field indicates the element type code for the reported element.
  - 0000b (0) - All element types reported
  - 0001b (1) - Medium transport element (accessor)
  - 0010b (2) - Storage element
  - 0011b (3) - Import/Export element
  - 0100b (4) - Data transfer element (drives)
- **SO**: A source only (SO) bit set to one indicates that the element is not capable of being used as a destination for a command that moves volumes (e.g., for an import/export element, this element is not capable of exporting volumes). A SO bit set to zero indicates that the element is capable of being used as a destination for a command that moves volumes.
- **DO**: A destination only (DO) bit set to one indicates that the element is not capable of being used as a source for a command that moves volumes (e.g., for an import/export element, this element is not capable of importing volumes). A DO bit set to zero indicates that the element is capable of being used as a source for a command that moves volumes.
- **NXP**: A no export (NXP) bit set to one indicates that a volume in the element is not able to be moved to an import/export element. A NXP bit set to zero indicates that a volume in the element is able to be moved to an import/export element.
- **MDO**: A moves during operation (MDO) bit set to one indicates that the physical position of the specified element is not fixed and the element moves during operation. A MDO bit set to zero indicates that the physical position of the specified element is fixed and the element does not move during operation (e.g., the media changer moves a magazine as part of the process of opening an import/export element and all elements in that magazine have the MDO bit set to one).
- **IESTOR**: An import/export or storage (IESTOR) bit set to one indicates that the specified element is configurable as either an import/export element or as a storage element. An IESTOR bit set to zero indicates that the specified element is not configurable as an import/export element or as a storage element (e.g., the element is always a storage element).
- **EDC**: An element disabled capable (EDC) bit set to one indicates that the specified element is capable of being disabled. An EDC bit set to zero indicates that the specified element is not capable of being disabled.


## Element State Information Page (03h)

The Element State Information element information page returns the current state of a set of element
characteristics that are not defined as static and may change.

**Table 7: Element State Information element information page**

```
              Bit        7            6             5            4           3           2            1           0
  Byte
          0                                                   Page Code (03h)
          1                                                       Reserved
```

```
               Bit          7          6                5              4          3          2            1           0
  Byte
          2
                                                                  Descriptor Length
          3
          4            (MSB)
                                                                  Page Length (n-7)
          7                                                                                                         (LSB)

                                                Element state information descriptors

          8
                                                Element state information descriptor (first)

                                                                           ...

                                                Element state information descriptor (last)
          n
```

- **Descriptor Length**: This field indicates the length of each element stae information descriptor. The descriptor length will be a multiple of 4 bytes long. The element state information descriptors will be zero padded.
- **Page Length**: This field is the length in bytes of the element state information descriptors that follow. If the descriptors are truncated because of the allocation length, then the PAGE LENGTH field will not be affected.
- **Element state information descriptors**: Refer to Element State Information Descriptors below.


## Element State Information Descriptors

Element state information descriptors will be returned in the format below.

**Table 8: Element state information descriptor**

```
              Bit       7             6             5              4             3           2            1            0
  Byte
         0           (MSB)
                                                                  Element Address
         3                                                                                                           (LSB)
         4                                Reserved                                         Element Type Code
```

```
             Bit       7             6              5            4              3         2            1            0
  Byte
         5           Volume Present                     Import              OIR          ED          MTAP          SDV
         6                                          Element State Additional Sense Code
         7                                  Element State Additional Sense Code Qualifier
         8
                                                                     Reserved
         n
```

- **Element Address**: This field indicates the element address being reported.
- **Element Type Code**: This field indicates the element type code for the element being reported.
  - 0000b (0) - All element types reported
  - 0001b (1) - Medium transport element (accessor)
  - 0010b (2) - Storage element
  - 0011b (3) - Import/Export element
  - 0100b (4) - Data transfer element (drives)
- **Volume Present**: This field values are defined below.
  - 00b - The device server is not able to determine if there is a volume in this element (for example, the CDATA bit in the CDB is set to one and the inventory information of this element is not known).
  - 01b - This element contains a volume.
  - 10b - This element does not contain a volume.
  - 11b - Reserved.
- **Import**: This field values are defined below. If the VOLUME PRESENT field is not set to 01b, then the IMPORT field is not valid.
  - 00b - The device server is not able to determine if the volume of this element was placed there by a medium transport element (for example, the media changer has processed a hard reset).
  - 01b - This volume in this element was placed there by a medium transport element.
  - 10b - This volume in this element was not placed there by a medium transport element.
  - 11b - Reserved.
- **OIR**: An operator intervention required (OIR) bit set to one indicates that operator intervention is required to make the element accessible (e.g., an I/E magazine needs to be closed or a drive needs to be installed or varied on).
- **ED**: An element disabled (ED) bit set to one indicates that the specified element is disabled. An ED bit set to zero indicates that that the specified element is not disabled.
- **MTAP**: A medium transport element access prohibited (MTAP) bit set to one indicates that access to the specified element by a medium transport element is prohibited. An MTAP bit set to zero indicates that access to the specified element by a medium transport element is not prohibited.
- **SDV**: A sense data valid (SDV) bit set to one indicates that the ELEMENT STATE ADDITIONAL SENSE CODE field and ELEMENT STATE ADDITIONAL SENSE CODE QUALIFIER field is valid and contain an additional sense code associated with the specified element. A SDV bit set to zero indicates that the content of the ELEMENT STATE ADDITIONAL SENSE CODE field and the ELEMENT STATE ADDITIONAL SENSE CODE QUALIFIER field is not valid.


## Element Location Information Page (04h)

The Element Location Information element information page returns descriptors containing vendor
specific descriptions for the element location. The set of element location information descriptors
reported by the library consist of the library's coordinate system information for the respective element
address, such that the reported byte string represents the section, column, and row coordinate as a
positive or negative byte number An element may be reported in multiple descriptors (e.g., an element
may be reported in a descriptor for frame 1 and a second descriptor for magazine 3).

**Table 9: Element Location Information element information page**

```
              Bit        7             6            5            4              3        2           1           0
  Byte
          0                                                   Page Code (04h)
          1
                                                                     Reserved
          3
          4           (MSB)
                                                              Page Length (n-7)
          7                                                                                                    (LSB)

                                            Element location information page descriptors
          8
                                           Element location information page descriptors (first)

                                                                       ...


                                           Element location information page descriptors (last)
          n
```

- **Page Length**: The value in the PAGE LENGTH field is the length in bytes of the element location information descriptors that follow. If the descriptors are truncated because of the allocation length, the PAGE LENGTH field will not be affected.
- **Element location information descriptors**: Refer to Element Location Information Descriptors below.


## Element Location Information Descriptors

Element location information descriptors will be returned in the format below.

**Table 10: Element location information descriptor**

```
              Bit        7             6            5            4              3        2           1           0
  Byte
          0           (MSB)
                                                        First Element Address Reported
          3                                                                                                    (LSB)
          4           (MSB)
                                                        Number of Elements Reported
          7                                                                                                    (LSB)
```

```
               Bit       7            6             5              4              3        2             1        0
  Byte
          8                            Reserved                                        Element Type Code
          9                                                            Reserved
          10          (MSB)
                                                            Parameter List Length (n-11)
          11                                                                                                     (LSB)
                                           Element Location Information Parameters List
          12
                                          Element Location Information Parameters List (First)

                                                                          ...

                                          Element Location Information Parameters List (Last)
          n
```

- **First Element Address Reported**: This field indicates the lowest element address being reported.
- **Number of Elements Reported**: This field indicates the number of contiguous elements with element addresses greater than or equal to the value specified in the FIRST ELEMENT ADDRESS REPORTED field and with the same set of element location information parameters.
- **Element Type Code**: This field indicates the element type reported by this element location information descriptor.
  - 0000b (0) - All element types reported
  - 0001b (1) - Medium transport element (accessor)
  - 0010b (2) - Storage element
  - 0011b (3) - Import/Export element
  - 0100b (4) - Data transfer element (drives)
- **Element Location Information Parameters List**: Refer to Element Location Information Parameters List below.


## Element Location Information Parameters List

The element location information parameters list contains a list of element location information
parameters for the specified element.

**Table 11: Element location information parameter**

```
               Bit       7            6              5             4              3        2            1            0
  Byte
          0           (MSB)
                                               Element Location Information Length (w-5)
          3                                                                                                        (LSB)
          4                               Reserved                                             Reserved
          5                                                   Location Type Code
          16          (MSB)
                                                                       Location
          n                                                                                                        (LSB)
```

- **Element Location Information Length**: This field indicates the length in bytes of the element location information to follow.
- **Code Set**: This field contains a code set enumeration (see SPC-4) that indicates the format of the LOCATION field.
- **Location Type Code**: This field is set to 0Eh to identify the reporting of an element coordinate string in byte string notation.


## Return All Supported Pages (7fH)

If the Return All Supported Pages element information page is requested, then the library will return all of
the pages supported by the elements selected by the STARTING ELEMENT ADDRESS field in the CDB
and the ELEMENT TYPE CODE field in the CDB in ascending order by page code.
