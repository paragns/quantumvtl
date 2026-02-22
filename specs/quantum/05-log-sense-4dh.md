# Log Sense - 4Dh

## What the Library Does with This Command

The library returns information for the requested log page. The only supported log page is the Tape Alert
page, with a limited set of flags. The library will return the current values of the flags on request, and then
clear them.


## Command Usage

This command can be used to monitor conditions of the library.


## Log Sense CDB Format

The LOG SENSE CDB format is shown in the following table.

**Table 1: LOG SENSE CDB format**

```
             Bit      7           6          5       4          3          2         1           0
  Byte
         0                                               Op Code (4Dh)

         1                                    Reserved                            Obsolete       SP
         2                 PC                                         Page Code
         3                                               SubPage Code
         4                                                 Reserved
         5
                                                     Parameter Pointer
         6
         7
                                                     Allocation Length
         8
         9                                                  Control
```

| Field | Description |
|---|---|
| Parameter Pointer Control (PPC) | Must be set to 0. The library will return log parameters starting with the parameter code specified in the Parameter Pointer field, and return up to the number of bytes specified in the Allocation Length field. Log parameters are returned in ascending order according to their parameter code. A PPC bit of 0 and a Parameter Pointer field of 0 will cause all available log parameters for the requested page code to be returned, subject to the Allocation Length. |
| Save Parameters (SP) | Must be set to 0. The library does not support the saving of log parameters. |
| Page Control (PC) | Must be set to 01b. The library only returns cumulative values for any log parameter rather than threshold or default values. |
| SubPage Code | The subpage code identifies any subpage for the page code. No subpage is defined for the Scalar i7 RAPTOR. This value must be set to 0. |
| Page Code | The Page Code field identifies which log page is being requested by the initiator. See Table 2 below. |
| Parameter Pointer | This field specifies which log parameter to begin with for the requested log page. A PPC bit of 0 and a Parameter Pointer field of 0 will cause all available log parameters for the requested page code to be returned, subject to the Allocation Length. More detailed definition of this field is contained within the specific log page descriptions. |
| Allocation Length | The Allocation Length field is used to determine the maximum amount of data to return. The transfer completes after either all the data has been transferred or an amount equal to the Allocation Length has been sent. Specify FFFFh to include all available data. |

**Table 2: Page Code field**

| Page Code | Page Name | Page Description |
|---|---|---|
| 00h | Supported Log Pages | Returns list of supported log pages |
| 0Dh | Temperature | Returns Library Tape Storage Area temperature |
| 12h | Tape Alert Response | Returns combined library tape alert flags |
| 2Eh | Tape Alert Log | Returns the 64 library-defined tape alert flags |
| 30h | Humidity | Returns Library Tape Storage Area humidity |


## Log Sense Response

The response to a LOG SENSE command returns the log page specified in the Page Code field of the
CDB. The log page format is described in "Log Page Format." The valid Page Code fields are listed in
Table 2 on the previous page. The various log parameters are described within their respective pages,
along with their Parameter Codes. The Log Parameter format is described in "Log Parameter Format."

### Log Page Format

The following table shows the Log Page format. The first four bytes are the Parameter List Header,
followed by the list of log parameters.

**Table 3: Log Page format**

```
           Bit     7            6            5          4              3         2           1            0
  Byte

       0          DS          SPF                                        Page Code
       1                                                    Reserved
       2
                                                     Page Length (n-3)
       3
       4
                                                    Log Parameter (First)
       :
                                                            (Length x)
    x+3
                                                        :
   n-y+1
                                                    Log Parameter (Last)
       :
                                                            (Length y)
       n
```

| Field | Description |
|---|---|
| DS | The DS bit indicates whether log parameters in this log page are saved if the SP bit is set to one in the CDB. If the DS bit is set to zero, the log parameters are saved if the SP bit is set to one. If the DS bit is set to one, the log parameters are not saved. The value for this bit is 0, as the SP bit in the CDB must be set to 0, indicating no saving of log parameters is supported. |
| SPF | If the subpage format (SPF) bit is set to zero, the SUBPAGE CODE field shall contain 00h. If the SPF bit is set to one, the SUBPAGE CODE field shall contain a value between 01h and FFh. The value is 0 as subpage codes are not supported. |
| Page Code | This field identifies which log page is being transferred. |
| SubPage Code | This field identifies which log sub page is being transferred. Subpages are not supported and must be set to 0. |
| Page Length | This field indicates the total number of bytes available to return for this page, beginning with the first log parameter. The value set for this field depends on the value specified for the Page Code. |
| Log Parameters | These are dependent upon the log page. The various parameters as well as their format for the supported pages are listed below. |

**Table 4: Log Parameter Format**

```
          Bit     7              6              5            4           3          2           1           0
  Byte
     0
                                                       Parameter Code
     1
                                                                                               FORMAT and
     2            DU         Obsolete         TSD          ETC               TMC
                                                                                                 LINKING
     3                                              Parameter Length (n-3)
     4
      :                                                Parameter Value
     n
```

| Field | Description |
|---|---|
| Parameter Code | This field identifies which log parameter was transferred. The valid values for this field depend on the log page. |
| Disable Update (DU) | Will be set to 0. The library will always update values reflected by the log parameters. |
| Target Save Disable (TSD) | Will be set to 0. The library provides a self-defined method for saving log parameters. |
| Enable Threshold Comparison (ETC) | Will be set to 0. No comparison to threshold values is made. |
| Threshold Met Criteria (TMC) | Will be set to 0. Comparison to threshold values is not supported. |
| FORMAT and LINKING | Will be reported as 01b when reporting ASCII format data, and 11b when reporting binary data. |
| Parameter Length | This field indicates the number of bytes that follow this field, which is the size of the parameter value. |
| Parameter Value | This field contains the actual parameter data, which can be either a data counter or a list parameter (ASCII string or binary value). |


### Supported Log Page (00h)

This page returns a list of all log pages supported by the library.

**Table 5: Supported Log Page (00h)**

```
           Bit     7              6         5            4           3           2            1            0
  Byte

       0         DS(0b)     SPF(0b)                                Page Code (00h)

       1                                            SubPage Code (00h)
       2
                                                      Page Length (5h)
       3
       4               Reserved                               Supported Log Page (00h)
       5               Reserved                              Temperature Log Page (0Dh)
       6               Reserved                          Tape Alert Response Log Page (12h)
       7               Reserved                               Tape Alert Log Page (2Eh)
       8               Reserved                                Humidity Log Page (30h)
```

| Field | Description |
|---|---|
| DS | Set to 0. See Log Parameter Format on the previous page Description. |
| SPF | Set to 0. See Log Parameter Format on the previous page Description. |
| Page Code | The returned value is 00h, indicating this page. |
| SubPage Code | Set to 00h to request all log page codes with a subpage code of 00h. |
| Page Length | The returned value is 0005h. |

> **Note:** Requests for subcodes other than subpage 00h will be rejected as an Illegal
> request as non-zero subpages are not supported.

The page codes for all the supported pages (including this one) follow the page length field.


### Temperature Log Page (0Dh)

Using the format shown below, the Temperature log page provides information about the current
operating temperature of the tape library enclosure temperature, as reported by robotics.

**Table 6: Temperature Log Page (0Dh)**

```
             Bit     7              6          5            4           3          2           1     0
  Byte

       0           DS(0b)       SPF (0b)                               Page Code (0Dh)

       1                                               SubPage Code (00h)
       2           (MSB)
                                                           Page Length (n-3)
       3                                                                                           (LSB)
                                                    Temperature log parameters
       4
       ...                                          Temperature log parameter
       n
```

| Field | Description |
|---|---|
| DS | Set to 0. See Log Parameter Format on page 43 Description. |
| SPF | Set to 0. See Log Parameter Format on page 43 Description. |
| Page Code | The returned value is 0Dh, indicating the Temperature Log Page. |
| SubPage Code | The returned value is 0006h. |
| Log Parameter Identification | See definition of Temperature Log Parameter below below. |

#### Temperature Log Parameter

The contents of the temperature log parameter depends on the value in its PARAMETER CODE field.
Only a single log parameter is supported for parameter code 0000h to report the current library
temperature in degrees Celsius. A TEMPERATURE value of FFh, indicates the current temperature is
unknown.

**Table 7: Temperature Log Parameter Page (0Dh)**

```
          Bit       7                 6               5             4           3          2         1           0
  Byte
      0           (MSB)
                                                      Parameter Code (0000h)
      1                                                                                                        (LSB)

      2            DU            Obsolete            TSD          ETC               TMC                  Format and
                                                                                                           Linking
      3                                               Parameter Length (02h)
      4                                                          Reserved
      5                                                      Temperature
```

| Parameter Code | DU | TSD | ETC | TMC | Format and Linking | Parameter Length |
|---|---|---|---|---|---|---|
| 0000h | 0 | 0 | 0 | 00b | 00b | 2 |


### Tape Alert Response Log Page (12h)

Using the format show in table below, the TapeAlert Response log page (page code 12h) is an alternative
log page to log page 2Eh, and provides error, warning and informational flags used for detailed device
diagnostics and management. The parameter fields represent the state flags for each corresponding
TapeAlert flag.

**Table 8: Tape Alert Response Log Page (00h)**

```
          Bit       7             6              5           4              3         2              1           0
  Byte

      0           DS(0b)      SPF(0)                                    Page Code (12h)

      1                                                   SubPage Code (00h)
      2           (MSB)
                                                          Page Length (000Ch)
      3                                                                                                        (LSB)
                                                 TapeAlert response log parameters
     4
                                                 TapeAlert response log parameter
     15
```

#### Tape Alert Response Log Parameter

The Tape Alert Response Log reports the tape library tape alerts in a single log parameter. A FLAGXXh
(where xxh denotes the corresponding TapeAlert flag) bit set to one indicates the state flag for the
corresponding TapeAlert flag is activated. A FLAGXXh bit set to zero indicates the state flag for the
corresponding TapeAlert flag is deactivated.

**Table 9: Tape Alert Response Log Parameter**

```
   Bit        7             6            5              4           3             2            1         0
  Byte
    0       (MSB)
                                                    Parameter Code (0000h)
    1                                                                                                 (LSB)

    2        DU         Obsolet        TSD            ETC                 TMC              Format and Linking
                          e
    3                                               Parameter Length (08h)

           FLAG0        FLAG02        FLAG0          FLAG04       FLAG05      FLAG06       FLAG0      FLAG0
    4
             1h            h            3h              h            h           h           7h         8h

           FLAG0        FLAG0         FLAG0          FLAG0        FLAG0         FLAG0      FLAG0      FLAG1
    5
             9h           Ah            Bh             Ch           Dh            Eh         Fh         0h

           FLAG1        FLAG12        FLAG1          FLAG14       FLAG15      FLAG16       FLAG1      FLAG1
    6
             1h            h            3h              h            h           h           7h         8h

           FLAG1        FLAG1         FLAG1          FLAG1        FLAG1         FLAG1      FLAG1      FLAG2
    7
             9h           Ah            Bh             Ch           Dh            Eh         Fh         0h

           FLAG2        FLAG22        FLAG2          FLAG24       FLAG25      FLAG26       FLAG2      FLAG2
    8
             1h            h            3h              h            h           h           7h         8h

           FLAG2        FLAG2         FLAG2          FLAG2        FLAG2         FLAG2      FLAG2      FLAG3
    9
             9h           Ah            Bh             Ch           Dh            Eh         Fh         0h

           FLAG3        FLAG32        FLAG3          FLAG34       FLAG35      FLAG36       FLAG3      FLAG3
   10
             1h            h            3h              h            h           h           7h         8h

           FLAG3        FLAG3         FLAG3          FLAG3        FLAG3         FLAG3      FLAG3      FLAG4
   11
             9h           Ah            Bh             Ch           Dh            Eh         Fh         0h
```

| Parameter Code | DU | TSD | ETC | TMC | Format and Linking | Parameter Length |
|---|---|---|---|---|---|---|
| 0000h | 0 | 1 | 0 | 00b | 00b | 08h |


### Tape Alert Log Page (2Eh)

The Tape Alert log page follows the standard log page format. Each Tape Alert is returned as an
individual log parameter, with its state reflected in bit zero of the one-byte Parameter Value field of the log
parameter. When this bit is set to one, the alert is active.
When requesting the Tape Alert log page, the Parameter Pointer determines from what point in the Tape
Alert table the alerts are returned. The value zero specifies that all tape alerts should be returned. If the
Parameter Pointer is set from 1 to 64, all tape alerts from that point to the end of the list are returned. The
various log parameters are listed in the following table.

**Table 10: Tape Alert Log Page Parameters**

| Log Parameter | Parameter Code | DU | TSD | ETC | TMC | Format and Linking | Parameter Length |
|---|---|---|---|---|---|---|---|
| Tape Alert Flag 1 | 0001h | 0 | 1 | 0 | 0 | 00b | 1 |
| Tape Alert Flag 2 | 0002h | 0 | 1 | 0 | 0 | 00b | 1 |
| : | | 0 | 1 | 0 | 0 | 00b | 1 |
| Tape Alert Flag 63 | 003Fh | 0 | 1 | 0 | 0 | 00b | 1 |
| Tape Alert Flag 64 | 0040h | 0 | 1 | 0 | 0 | 00b | 1 |

The supported Tape Alert flag parameter values are described in the table below. A tape alert is active if
the parameter value byte reports a 1, inactive if the parameter reports a 0.
The severity of the flags has the following meaning:

- Critical (C)
- Warning (W)
- Informational (I)

**Table 11: Tape Alert Flag Descriptions**

| Tape Alert Flag | Description |
|---|---|
| Flag 1: Drive Communication Failure (C) | This flag is set to indicate a drive communication failure. |
| Flag 2: Library Hardware B (W) | This flag is set for any unrecoverable mechanical error. |
| Flag 4: Library Hardware D (C) | This flag is set when the internal Power-On-Self-Tests (POST) fail or when a mechanical error occurs that requires a power cycle to recover, and is not internally cleared until the device is powered off. |
| Flag 13: Library Pick Retry (W) | This flag is set when a high retry count threshold is passed when performing an operation to pick a cartridge from a slot before the operation succeeds. It is internally cleared when another pick operation is attempted. |
| Flag 14: Library Place Retry (W) | This flag is set when a high retry count threshold is passed when performing an operation to place a cartridge back into a slot before the operation succeeds. It is internally cleared when another place operation is attempted. |
| Flag 15: Library Load Retry (W) | This flag is set when a high retry count threshold is passed when performing an operation to load a cartridge into a drive before the operation succeeds. It is internally cleared when another load operation is attempted. Note that if the load actually fails due to a media or drive problem, the appropriate TapeAlert flags should be set by the drive. |
| Flag 16: Library Door (C) | This flag is set when media move operations cannot be performed because a door is open, and is internally cleared when the door is closed. This flag is not supported by the Scalar i40 and Scalar i80 libraries. |
| Flag 17: Mailbox Mechanical Problem (C) | This flag is set when a mailbox station mechanical problem is detected. |
| Flag 23: Library Scan Retry (W) | This flag is set when a high retry count threshold is passed when performing an operation to scan the barcode on a cartridge before the operation succeeds. It is internally cleared when another barcode scanning operation is attempted. |
| Flag 27: Cooling Fan Failure (W) | This flag is set when a cooling fan has failed within a library component. This flag is only supported by the Scalar i40 and Scalar i80 libraries. |
| Flag 28: Power Supply Failure (W) | This flag is set when a redundant power supply has failed within the library. This flag is only supported by the Scalar i40 and Scalar i80 libraries. |
| Flag 32: Barcode Label Unreadable (I) | This flag is set when a tape cartridge barcode label could not be read. (This flag is only supported by the Scalar i40, Scalar i80, and Scalar i500 libraries.) |


### Humidity Log Page (30h)

Using the format shown below, the Humidity log page provides information about the current relative
humidity of the tape library enclosure per robot sensed humidity

**Table 12: Humidity Log Page (30h)**

```
          Bit       7             6              5            4           3            2            1            0
  Byte

      0           DS(0b)      SPF(0)                                     Page Code (30h)

      1                                                    SubPage Code (00h)
      2           (MSB)
                                                             Page Length (n-3)
      3                                                                                                        (LSB)
                                                          Humidity log parameters
     4
                                                     Humidity log parameter [first]
      n
```

#### Humidity Log Parameter

The contents of the humidity log parameter depends on the value in its PARAMETER CODE field. Only a
single log parameter is supported for parameter code 0000h to report the tape library enclosure's current
relative humidity percentage. A value of FFh indicates the current humidity level is unknown.

**Table 13: Humidity Log Page Parameters**

```
          Bit       7                 6               5             4            3           2            1             0
  Byte
      0           (MSB)
                                                            Parameter Code (0000h)
      1                                                                                                               (LSB)

      2            DU            Obsolete            TSD           ETC                TMC               Format and Linking
      3                                                     Parameter Length (02h)
      4                                                            Reserved
      5                                                             Humidity
```

| Parameter Code | DU | TSD | ETC | TMC | Format and Linking | Parameter Length |
|---|---|---|---|---|---|---|
| 0000h | 0 | 0 | 0 | 00b | 00b | 2 |
