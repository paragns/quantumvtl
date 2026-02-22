# Request Volume Element Address - B5h

## What the Library Does With This Command

> **Note:** Request Volume Element Address B5h is only supported by Scalar i2000 and i6000 libraries.

The library returns element descriptors that match the request made through a SEND VOLUME TAG
command. Each element address will only be reported once in response to a SEND VOLUME TAG
request. Multiple REQUEST VOLUME ELEMENT ADDRESS commands may be used to retrieve all the
elements. If no elements match the SEND VOLUME TAG request, or all elements have already been
reported, the response will only contain the volume element address header.


## Command Usage

This command is used to receive the results of a SEND VOLUME TAG command.


## Request Volume Element Address CDB Format

The REQUEST VOLUME ELEMENT ADDRESS CDB format is shown in the following table.

**Table 1: REQUEST VOLUME ELEMENT ADDRESS CDB format**

```
              Bit       7           6          5          4             3      2        1       0
  Byte
         0                                               Op Code (B5h)
         1                     Reserved                 VolTag              Element Type Code
         2
                                                    Starting Element Address
         3
         4
                                                      Number of Elements
         5
         6                                                 Reserved
         7
         :                                             Allocation Length
         9
         10                                                Reserved
         11                                                   Control
```

- **Volume Tag (VolTag)**: This field indicates whether the volume tag (bar code label) information should be returned. A value of one will return the labels, a value of zero will not.
- **Element Type Code**: This field specifies the element types selected for the returned information, as shown in Table 2 below.
- **Starting Element Address**: This field specifies the minimum element address to report. Only elements with an element type code specified by the Element Type Code field, and with an address greater than or equal to the starting element address will be reported. The starting element address must be a valid element address, but not have to be within the range specified by the Element Type Code field.
- **Number of Elements**: This field specifies the maximum number of element descriptors to return. Only those descriptors that can be completely transferred within the allotted allocation length will be returned.
- **Allocation Length**: This field specifies the byte length allowed for returned element descriptors. Only complete element descriptors are returned. The library returns element descriptors until one of the following conditions are met:
  - All available element descriptors have been returned
  - The number of element descriptors specified in the Number of Elements field have been returned
  - The number of bytes specified in the Allocation Length field have been returned
  - There is less allocation length space available than is required by the next complete element descriptor

**Table 2: Element Type Code**

| Code | Selected Element Type |
|------|----------------------|
| 0000b (0) | All element types reported |
| 0001b (1) | Medium transport element (accessor) |
| 0010b (2) | Storage element |
| 0011b (3) | Import/Export element |
| 0100b (4) | Data transfer element (drives) |


## Request Volume Element Address Response

Element status data consists of an eight-byte header, followed by one or more element status pages (per
element type). Each element status page consists of a header, followed by one or more element
descriptor blocks. A complete response then looks like:

```
Element Status Header
          Element Status Page Header (first element type)
                    Element Descriptor
                    …(more descriptors)…
                    Element Descriptor
          …(more status pages)…
          Element Status Page Header (next element type)
                    Element Descriptor
                    …
                    Element Descriptor
```

There are only up to four Element Status Pages, one for each element type. The element status pages
are identical to those described for the READ ELEMENT STATUS command in Element Status Page
(see Element Status Page on page 113).

One header is returned for each REQUEST VOLUME ELEMENT ADDRESS command. The format is
shown in the following table.

**Table 3: Element Status Header**

```
              Bit        7           6              5           4          3          2          1          0
  Byte
          0
                                                First Element Address Reported
          1
          2
                                                    Number of Elements Available
          3
          4                              Reserved                                Send Action Code (5h)
          5
          :                                         Byte Count of Report Available
          7
```

- **First Element Address Reported**: This field indicates the lowest element address found that meets the CDB request.
- **Number of Elements Available**: This field indicates the number of elements found that meet the CDB request.
- **Send Action Code**: This field contains the action code that was sent in the SEND VOLUME TAG command. The value is 5h.
- **Byte Count of Report Available**: This field indicates the number of available element status bytes that meet the CDB requirements. The value does not include the eight-byte element status header, and is not adjusted to match the value specified in the Allocation Length field of the CDB. This facilitates first issuing a REQUEST VOLUME ELEMENT ADDRESS command with an allocation length of eight bytes in order to determine the allocation length required to transfer all the element status data specified by the command.

The element descriptors within each page are also the same as those described for READ ELEMENT
STATUS in Element Descriptors (see Element Descriptors on page 114).
