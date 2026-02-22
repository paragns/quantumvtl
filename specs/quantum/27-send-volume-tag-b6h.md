# Send Volume Tag - B6h

## What the Library Does With This Command

> **Note:** Send Volume Tag B6h is only supported by Scalar i2000 and i6000 libraries.

The library searches its existing inventory for barcode labels that match the volume tag template passed
in with this command. The results of this search can then be retrieved through sending a subsequent
REQUEST VOLUME ELEMENT ADDRESS command.


## Command Usage

This command can be used to search for specific cartridges or ranges of cartridges within the library. A
REQUEST VOLUME ELEMENT ADDRESS command must be sent to retrieve the results of the search.
The results only reflect those of the most recent SEND VOLUME TAG command.


## Send Volume Tag CDB Format

The SEND VOLUME TAG CDB format is shown in the following table.

**Table 1: SEND VOLUME TAG CDB format**

```
              Bit        7           6              5         4             3          2           1    0
  Byte
         0                                                   Op Code (B6h)

         1                            Reserved                                      Element Type Code
         2
                                                        Starting Element Address
         3
         4                                                        Reserved
         5                      Reserved                                        Send Action Code
         6
                                                                  Reserved
         7
         8
                                                         Parameter List Length
         9
         10                                                       Reserved
         11                                                       Control
```

- **Element Type Code**: This field specifies the element types selected for the search, as shown in the following table.

**Table 2: Element Type Code**

| Code | Selected Element Type |
|------|----------------------|
| 0000b (0) | All element types reported |
| 0001b (1) | Medium transport element (accessor) |
| 0010b (2) | Storage element |
| 0011b (3) | Import/Export element |
| 0100b (4) | Data transfer element (drives) |

- **Starting Element Address**: This field specifies the minimum element address to begin the search. Only elements with an element type code specified by the Element Type Code field, and with an address greater than or equal to the starting element address will be searched.
- **Send Action Code**: This field must be set to 5h to indicate translate and search primary volume tags and ignore sequence numbers. No other action codes are supported.
- **Parameter List Length**: This field is either set to 0 to indicate that no parameter data is sent, or 28h (40) to indicate a Volume Identification Template is sent. A value of 0 is not considered an error.


## Volume Identification Template Parameter

The Volume Identification Template parameter is shown in the following table.

**Table 3: Volume Identification Template Parameter**

```
               Bit        7          6              5        4          3           2           1           0
  Byte
          0
          :                                         Volume Identification Template
          31
```

```
               Bit       7           6              5            4          3           2          1           0
  Byte
          32
          :                                                          Reserved
          39
```

- **Volume Identification Template**: This field specifies the template to apply for the search. Two wildcard characters are supported as follows:
  - '?' -- Will match any single character.
  - '*' -- Will match any string of characters. When it appears in the template the remainder of the template at higher offsets in the field is not used.
