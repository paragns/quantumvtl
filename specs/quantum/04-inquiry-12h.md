# Inquiry - 12h

## What the Library Does With This Command

In response to this command the library returns static data that describes various subsystem parameters.
Each controller device and media changer device logical unit will return its own Inquiry data. If an
INQUIRY command is received from an initiator with a pending unit attention condition, the library will
perform the INQUIRY command and will not clear the unit attention condition. An INQUIRY command will
respond with a Check Condition status only when it cannot return the requested Inquiry data.


## Command Usage

This command would normally only be issued once for each logical unit as desired by the initiator to
facilitate the initialization process.


## Inquiry CDB Format

The INQUIRY CDB format is shown in the following table.

**Table 1: INQUIRY CDB format**

```
               Bit       7           6              5         4           3         2             1             0
 Byte
          0                                                       Op Code (12h)
          1                                         Reserved                                  Obsolete        EVPD
          2                                                        Page Code
          3                                                         Reserved
          4                                                    Allocation Length
          5                                                          Control
```

| Field | Description |
|---|---|
| Enable Vital Product Data (EVPD) | An EVPD value of 1 indicates that the vital product data specified by the Page Code should be returned. A value of 0 indicates that standard inquiry data should be returned. |
| Page Code | This field specifies which vital product data page to return if the EVPD bit is set to 1. If the EVPD bit is set to 0, the Page Code must be 00h. The library supports the following page codes: 00h - Supported Vital Product Data pages (this list), 80h - Unit Serial Number page, 83h - Device Identification page, 85h - Management Network Addresses page, C8h - Vendor Specific Device Capabilities page |
| Allocation Length | The Allocation Length field specifies the maximum number of bytes that the initiator allocated for returned inquiry data. An Allocation Length of 0 indicates that no inquiry data is to be transferred (this condition is not considered an error). The library terminates the data transfer when it has transferred the lesser of either the number of bytes specified by the Allocation Length field or all of the available inquiry data. |


## Standard Inquiry Response

**Table 2: Standard Inquiry Response**

```
  Bit         7             6             5               4         3            2              1          0
 Byte
   0              Peripheral Qualifier                                  Peripheral Device Type

   1        RMB         LU_                                              Reserved
                        CONG
   2                                                          Version
           Reserve      Reserve      NormAC
   3                                                    HiSup               Response Data Format
              d            d            A
   4                                                Additional Length n-4
                                                                                                      PROTEC
   5       SCCS           ACC                 TPGS                 3PC               Reserved
                                                                                                        T
           Obsolet      EncSer                          Multi     Obsolet    Reserve       Reserve
   6                                    BarC                                                            Addr16
             e            v                              P          e           d             d
           Obsolet      Reserve                                   Obsolet    Reserve       CmdQu
   7                                  Wbus16            Sync                                             SftRe
             e             d                                        e           d            e

  Bit         7             6              5            4          3              2    1            0
 Byte
   8
    :                                               Vendor Identification
  15
  16
    :                                               Product Identification
  31
  32
    :                                            Firmware Revision Level
  35
  36
    :                                          Full Firmware Revision Level
  54
  55                                                Reserved                                      BarC
  56*                           Reserved                               Clocking       QAS          IUS
  57*                                                       Reserved
  58*
                                                Optional Version Descriptor 1
  59*


  72*
  73*                                           Optional Version Descriptor 8
  75*
   ...                                                      Reserved
  95*
  96*

                                               Vendor Specific (not supported)
   ...
   n*
```

\* When requesting a standard inquiry response from a media changer device logical unit hosted by a DA
blade, only data for bytes 0 to 55 will be returned. Bytes 56 and all following bytes are not returned by
media changer device logical units hosted by a DA blade.

> **Note:** For Scalar i2000 and i6000 libraries - Once a library control firmware update gets scheduled,
> the library will not show the firmware revision of the installed library firmware bundle in the standard
> inquiry response if the library is operating in an emulation mode, but rather emulates a firmware
> version for the emulated device.

| Field | Description |
|---|---|
| Peripheral Qualifier | A return value of 000b indicates that the library supports the peripheral device type at the specified LUN. A return value of 001b indicates that the library is capable of supporting the peripheral device type at the specified LUN, however the device is not currently connected to it. A return value of 011b indicates no peripheral device types are supported at that LUN. |
| Peripheral Device Type | For media changer device logical units, this field returns 01000b (08h) to indicate it is a media changer device. For the DA blade controller device logical unit, this field returns 01100b (0Ch) to indicate it is a controller device. If an unsupported LUN was specified, this field returns 11111b (1Fh), which indicates that the device type is unknown. |
| Removable Medium Bit (RMB) | For media changer device logical units, this field returns 1, indicating media is removable. For the DA blade controller device logical unit it returns 0. |
| LU_CONG | This field returns a 0 to indicate that the Logical Unit is not part of a logical unit conglomerate |
| Version | This field returns 03h, indicating compliance with the SCSI-3 standard. Note that this field will report SCSI-3 standard compliance despite support of various SMC-x defined command and response information for certain SCSI commands, or this field returns the value 06h to indicate compliance with the SPC-4 standard (Scalar i7 RAPTOR only). |
| Normal ACA Supported (NormACA) | If the media changer device is configured through a control path drive, the NACA bit is not supported and this field returns 0. |
| Hierarchical Support (HiSup) | This field returns a 1, indicating that the hierarchical addressing model is used to assign LUNs, and that the REPORT LUNs command is supported. |
| Response Data Format | Returned as 0010b, indicating response data is in standard SCSI format. |
| Additional Length | When requesting a standard inquiry response from the DA blade controller device logical unit, this field is set to 1Fh, indicating 31 additional bytes of data following this field. When requesting a standard inquiry response from a media changer device logical unit hosted by a DA blade, this field returns 33h, indicating 51 additional bytes of data following this field. When requesting a standard inquiry response from a media changer device logical unit hosted by a tape drive, this field may report 33h (51) or more bytes of additional data, indicating that 51 or more additional bytes of data will be following this field. If more than 51 bytes of data follow, the media changer may indicate various protocol support in the optional version descriptors, or the tape drive hosting the media changer device control path may modify data within the response and update the length value, depending on the tape drive interface type and supported version descriptors. |
| SCC Supported (SCCS) | For the media changer device logical units, this field returns a 0. For the DA blade controller device logical unit, this field returns a 1. |
| Target Port Groups Supported (TPGS) | If the media changer device control path is hosted by a DA blade, this field is set to 0. If the media changer device control path is hosted by a tape drive, this field is filled in by the tape drive hosting the interface and may be set to 1 if the REPORT TARGET PORT GROUPS command is supported. |
| Third Party Copy (3PC) | This field is set to 0 to indicate that third party copy is not supported. |
| PROTECT | This field is set to 0 to indicate that protection information is not supported. |
| Enclosure Services (EncServ) | Returned as 0, indicating an enclosure services component is not included. |
| Bar Code (BarC) | For media changer device logical units, this field returns a 1, indicating a bar code scanner or imaging device is installed (also returned in byte 55 below). For the controller device logical unit, this field returns a 0. |
| Multi Port (MultiP) | Returned as 0 if the device providing the medium changer interface supports a single port; returned as 1 if the device providing the medium changer interface supports 2 or more ports. |
| Wide SCSI Address 16 (Addr16) | Returned as 0, indicating 16-bit wide SCSI addresses are supported (applies to parallel SCSI only). |
| Wide Bus 16 (Wbus16) | Returned as 0, indicating 16 bit transfers are supported (applies to parallel SCSI only). |
| Synchronous Transfer (Sync) | Returned as 0, indicating synchronous transfers are supported (applies to parallel SCSI only). |
| Linked Commands (Linked) | Returned as 0, indicating linked commands are not supported. |
| Command Queuing (CmdQue) | For Fibre Channel, this is returned as 1, indicating command queuing is supported. For SCSI and SAS, this is returned as 0, indicating no command queuing is supported. |
| Soft Reset (SftRe) | Returned as 0, indicating a soft reset is not supported. |
| Vendor Identification | Returned as one of the following (space filled to 8 bytes): "QUANTUM " or "ADIC " |
| Product Identification | Depending upon the library type, returned as one of the following (space filled to 16 bytes): "Scalar 24 ", "Scalar 100 ", "Scalar 1000 ", "Scalar 10K ", "Pathlight VX ", "Scalar i500 ", "Scalar i40-i80 ", "Scalar i6000 ", "Scalar i3-i6 ", "Scalar i7 " |
| Firmware Revision Level | Returned as the ASCII representation of the revision level, such as "100A" or "203G". The remaining fields are only returned for media changer device logical units. |
| Full Firmware Revision Level | Same as the firmware revision level, but extended to include the build number (if available). |
| Bar Code (BarC) | Returned as 1, indicating a bar code scanner or imaging device is installed. Also returned in byte 6 above. |
| Clocking | This field is only returned for media changer device logical units controlled via parallel SCSI; otherwise, this field is reserved. This field does not apply to asynchronous transfers and is defined as: 00b - Indicates the target port supports only single transition (one transfer per clock cycle). 01b - Indicates the target port supports only double transition (two transfers per clock cycle; 16-bit only). 10b - Reserved. 11b - Indicates the target port supports single transition and double transition. |
| QAS | This field is only returned for media changer device logical units controlled via parallel SCSI; otherwise, this field is reserved. A quick arbitration and selection supported (QAS) bit of one indicates that the target port supports quick arbitration and selection. A value of zero indicates that the target port does not support quick arbitration and selection. |
| IUS | This field is only returned for media changer device logical units controlled via parallel SCSI; otherwise, this field is reserved. An information units supported (IUS) bit of one indicates that the SCSI target device supports information unit transfers. A value of zero indicates that the SCSI target device does not support information unit transfers. |
| Version Descriptors | Optional Version Descriptors indicate SCSI command compliance with certain standards. The data transfer element hosting the interface may add/insert version descriptors for the tape drive transport protocol reporting which physical layer and transport layer revisions are claimed for any FC or SAS transport protocols. The Scalar i7 RAPTOR tape library reports the following SCSI Medium Changer version descriptors in the optional version descriptors 1, 2 and 3: 00A0h - SAM-5, no version claimed. 0460h - SPC-4, no version claimed. 0480h - SMC-3, no version claimed. |


## Vital Product Data Pages

The collection of Vital Product Data pages is as follows:

### Supported Vital Product Data Page (00h)

Contains a list of all supported vital product data page codes.

**Table 3: Supported Vital Product Data Page (00h)**

```
               Bit       7           6              5          4          3          2           1           0
 Byte
          0               Peripheral Qualifier                           Peripheral Device Type
          1                                                 Page Code (00h)
          2                                                        Reserved
          3                                                   Page Length
          4                                             First Page Code Supported
          5                                         Second Page Code Supported
          6                                             Third Page Code Supported
          7                                         Fourth Page Code Supported
          8                                             Fifth Page Code Supported
```

| Field | Description |
|---|---|
| Peripheral Qualifier | The return value 000b indicates that the library supports the peripheral device type at the specified LUN. This field returns 011b if no peripheral device types are supported at that LUN. |
| Peripheral Device Type | For media changer device logical units, this field returns 01000b (08h) to indicate it is a media changer device. For the controller device logical unit, this field returns 01100b (0Ch) to indicate it is a controller device. If an unsupported LUN was specified, this field returns 11111b (1Fh), which indicates that the device type is unknown. |
| Page Code | Returned as 00h, indicating this page. |
| Page Length | Returns the number of bytes following this field. |
| First Page Code Supported | Returned as 00h, indicating support for the Supported Vital Product Data Page. |
| Second Page Code Supported | Returned as 80h, indicating support for the Unit Serial Number Page. |
| Third Page Code Supported | Returned as 83h, indicating support for the Device Identification Page. |
| Fourth Page Code Supported | Returned as 85h, indicating support for the Management Network Addresses Page. |
| Fifth Page Code Supported | Returned as C8h, indicating support for Vendor Specific Device Capabilities Page. |

### Unit Serial Number Page (80h)

Second page code supported. Returns system serial number.

**Table 4: Unit Serial Number Page (80h)**

```
               Bit       7            6             5        4          3           2           1            0
 Byte
          0               Peripheral Qualifier                         Peripheral Device Type
          1                                             Page Code (80h)
          2           (MSB)
                                                         Page Length (n-3)
          3                                                                                              (LSB)
          4
          :                                               Serial Number
          27
```

| Field | Description |
|---|---|
| Peripheral Qualifier | The return value 000b indicates that the library supports the peripheral device type at the specified LUN. This field returns 011b if no peripheral device types are supported at that LUN. |
| Peripheral Device Type | For media changer logical units, this field returns 01000b (08h) to indicate it is a media changer device. For the controller device logical unit, this field returns 01100b (0Ch) to indicate it is a controller device. If an unsupported LUN was specified, this field returns 11111b (1Fh), which indicates that the device type is unknown. |
| Page Code | Returned as 80h, indicating this page. |
| Page Length | Returned as 18h, indicating the remaining number of bytes following this field. |
| Serial Number | The value returned for this field is the serial number for the system, prefixed with the vendor identification. The serial number is padded with trailing spaces as needed to complete the 24 bytes. |

### Device Identification Page (83h)

Third page code supported. Returns device identification descriptors.

**Table 5: Device Identification Page (83h)**

```
              Bit         7            6            5         4         3           2          1           0
 Byte
          0                Peripheral Qualifier                        Peripheral Device Type
          1                                               Page Code (83h)
          2            (MSB)
                                                          Page Length (n-3)
          3                                                                                             (LSB)
                         Identification Descriptors (see Table 6 on the next page)
          4
                                                    First Identification Descriptor
          :
                                                          :
          :
                                                    Last Identification Descriptor
          n
```

| Field | Description |
|---|---|
| Peripheral Qualifier | The return value 000b indicates that the library supports the peripheral device type at the specified LUN. This field returns 011b if no peripheral device types are supported at that LUN. |
| Peripheral Device Type | For media changer device logical units, this field returns 01000b (08h) to indicate it is a media changer device. For the controller device logical unit, this field returns 01100b (0Ch) to indicate it is a controller device. If an unsupported LUN was specified, this field returns 11111b (1Fh), which indicates that the device type is unknown. |
| Page Code | Returned as 83h, indicating this page. |
| Page Length | Returns the remaining number of bytes following this field. |

### Identification Descriptors

The general format of identification descriptors are in the table below.

**Table 6: Identification Descriptors**

```
               Bit        7                6          5           4             3         2          1           0
 Byte
         0                            Protocol Identifier                                  Code Set
         1               PIV              Rsvd        Association                       Identifier Type
         2
                                                           Identifier Length (n-3)
         3
         4
          :                                                        Identifier
         n
```

| Field | Description |
|---|---|
| Protocol Identifier | The PROTOCOL IDENTIFIER field may indicate the SCSI transport protocol to which the identifier type applies. If the ASSOCIATION field contains a value other than 01b (i.e., target port) or 10b (i.e., SCSI target device) or the PIV bit is set to zero, then the PROTOCOL IDENTIFIER field contents are reserved. If the ASSOCIATION field contains a value of 01b or 10b and the PIV bit is set to one, then the PROTOCOL IDENTIFIER field shall contain one of the values shown below to indicate the SCSI transport protocol to which the identifier type applies. 0h - Fibre Channel, 1h - Parallel SCSI, 2h - SSA, 3h - IEEE 1394, 4h - SCSI Remote Direct Memory Access Protocol, 5h - Internet SCSI (iSCSI), 6h - SAS Protocol Layer, 7h - Automation/Drive Interface Transport Protocol, 8h - AT Attachment Interface (ATA/ATAPI), 9h to Eh - Reserved, Fh - No specific protocol |
| Code Set | This field returns the following values: 1h - the Identifier field contains binary values, 2h - the Identifier field contains ASCII characters |
| PIV | A protocol identifier valid (PIV) bit set to zero indicates the PROTOCOL IDENTIFIER field contents are reserved. If the ASSOCIATION field contains a value of 01b or 10b then a PIV bit set to one indicates the PROTOCOL IDENTIFIER field contains a valid protocol identifier. If the ASSOCIATION field contains a value other than 01b or 10b, then the PIV bit contents are reserved. |
| Association | This field returns the following values: 0h - the Identifier field is associated with the address physical or logical device, 1h - the Identifier field is associated with the port that received the request |
| Identifier Type | This field returns the following values: 1h - The identifier is a concatenation of the Vendor Identification field from the Standard Inquiry response and the Serial Number field (without the Vendor Identification prefix) from the Unit Serial Number page. 3h - The identifier is an IEEE Registered format Name_Identifier (Worldwide Name). 4h - The identifier is a port number. In this case, the Code Set and Association fields will both be set to 1. |
| Identifier Length | This is the length of the Identifier field, and will vary by identifier type. |
| Identifier | This is the identifier as described by the Code Set, Association, and Identifier Type fields. |

### Identification Descriptors

The media changer device logical unit returns the T10 Device Identification Descriptor (type 1h) as well
as the NAA Device Identification Descriptor (type 3h). The data transfer element hosting the interface will
add a NAA Port Identification Descriptor (type 3h), and a Relative Target Port Identifier (type 4h) and may
modify which descriptor is reported depending on the medium changer reporting as its own device at LUN
0, or just a LUN behind the data transfer element device.

#### Media Changer Identification Descriptor

Media changer logical units report only a single identifier. They will report the same identifier on either
SCSI or Fibre Channel.

**Table 7: Media Changer Identification Descriptor**

```
                  Bit         7          6           5            4         3             2         1          0
 Byte
           0                          Protocol Identifier                                Code Set = 2h
                             PIV
           1                           Rsvd       Association = 0h                     Identifier Type= 1h

           2
                                                         Identifier Length = 20h (32)
           3
           4
                                                             Vendor Identification
                                             (as reported in the Standard Inquiry response)
           11
           12
                                                                Serial Number
                              (as reported in the Unit Serial Number page without Vendor Identification prefix)
           35
```

#### Controller Device Identification Descriptors

On the parallel SCSI and SAS interface, the controller device logical unit will return the same
identification descriptor as the media changer devices, as shown in Table 7 above.
On the Fibre Channel interface, the controller device logical unit will return three different identification
descriptors as shown Table 8 below, Table 9 on the next page, and Table 11 on page 36.
The first two descriptors describe the World Wide Node Name and World Wide Port Name.

**Table 8: Controller Device Node Identification Descriptor**

```
            Bit          7              6                5            4            3          2         1      0
 Byte

       0                              Protocol Identifier                                  Code Set = 1h

       1                PIV           Rsvd           Association = 0h                    Identifier Type= 3h

       2
                                                         Identifier Length = 08h
       3

       4
        :                                           World Wide Node Name (WWNN)
      11
```

**Table 9: Controller Device Port Identification Descriptor**

```
                     Bit        7           6           5             4          3           2          1         0
 Byte
            0                            Protocol Identifier                                Code Set = 1h
            1                  PIV        Rsvd        Association = 1h                    Identifier Type= 3h
            2
                                                                Identifier Length = 08h
            3
            4
            :                                          World Wide Port Name (WWPN)
            11
```

The eight-byte Node and Port Worldwide Names have the following format:

**Table 10: Node and Port Worldwide Names**

```
 MSB
                                                                                                            LSB
                                                                                     36-bit Vendor Specified
     4-bit NAA ID                               24-bit Company ID
                                                                                             Identifier
                                       00 30 8C - Quantum Corporation
                                      (Default Vendor ID, formerly ADIC)

                5h              00 50 84 - Quantum Corporation (reassigned               Assigned per library
                                         by IEEE as of 09/03/2019)
                                        00 0E A4 - Quantum Corporation
                                     (reassigned by IEEE as of 09/03/2019)
```

The third descriptor (see table below) describes the relative target port.

**Table 11: Relative Target Port Identification Descriptor**

```
               Bit        7            6            5            4           3        2         1          0
 Byte
         0                         Protocol Identifier                               Code Set = 1h
         1               PIV         Rsvd        Association = 1h                  Identifier Type= 4h
         2
                                                           Identifier Length = 04h
         3
         4
                                                               Port Number
          :
                                                               (Starting with 1)
         7
```


### Management Network Addresses Page (85h)

Fourth page code supported. Returns system network configurations.

**Table 12: Network Address Page**

```
               Bit         7            6           5            4          3        2         1          0
 Byte
         0                              Peripheral Qualifier                          Peripheral Device Type
         1                                                   Page Code (85h)
         2              (MSB)

                                                    Page Length (n-3)
         3                                                                                               (LSB)


         4              (MSB)                       Network services descriptor (first)
                                                                                                         (LSB)


                                                    Network services descriptor (last)
         n              (MSB)
                                                                                                         (LSB)
```

Each network service descriptor contains information about one management service.

**Table 13: Network Services Descriptor**

```
              Bit         7             6           5          4         3         2          1           0
 Byte
          0           Rsvd              Association                           Service Type
          1                                                   Reserved
          2           (MSB)

                                            Network Address Length (n-3)
          3                                                                                            (LSB)
          4
          .
          .                                                 Network Address
          N
```

| Field | Description |
|---|---|
| Association | The association shall always be set to 00h "The Identifier field is associated with the addressed logical unit." |
| Service Type | The service type defines the library access and protocol methodologies. Scalar libraries only support service type 03h to indicate remote UI, CLI, or Web Services support. |
| Network Address Length | The network address length field contains the length in bytes of the network address field. This length reported in this field is a multiple of 4 bytes. |
| Network Address | The network address is a null-terminated, null-padded URL. The table above lists the defined network addresses which may be returned. Other vendor unique network addresses may also be returned, although only service type 03h is supported at this time. The network address conforms to RFC 2396 and is of the form "scheme://host:port/path". The host field contains the numeric IP address of the referenced host. The service type and scheme identifies the unique service that the library may report. For HTTP communication this path may be a redirect but for all others it is a direct path. The port may be vendor specific unless otherwise designated in the specification for that service. |

**Table 14: Service Types**

| Service Type | Service Type Name | Description | Protocol | Scheme |
|---|---|---|---|---|
| 00h | Reserved | | | |
| 01h | Reserved | | | |
| 02h | Diagnostics | Service Web Interface | HTTP | service |
| 02h | Diagnostics | Secure Service Web Interface | HTTPS | service |
| 03h | Management/Status^1 | Library Web Interface | HTTP | http |
| 03h | Management/Status | Secure Library Web Interface | HTTPS | https |
| 03h | Status | Vendor Command Line Interface | Telnet | telnet |
| 03h | Status | Secure Vendor Command Line Interface | SSH | ssh |
| 04h | Reserved | | | |
| 05h | Code Download^2 | Library Firmware Download | FTP | ftp |
| 05h | Code Download | Library Firmware Download | TFTP | tftp |
| 05h | Code Download | Library Firmware Download | SFTP | sftp |
| 05h | Code Download | Drive Firmware Download | FTP | dftp |
| 05h | Code Download | Drive Firmware Download | TFTP | dtftp |
| 05h | Code Download | Drive Firmware Download | FTP | dsftp |

1. The library web interface service is required if this page is supported.
2. The default behavior for "Code Download" network services shall be to apply the new firmware and perform any
   necessary reconfiguration to apply that firmware. The device may support applying the firmware at the next
   power cycle and may designate such "postponed" download by reporting another network service and pre-
   pending a "p" to the scheme (for example: pftp).

### Vendor Specific Device Capabilities Page (C8h)

Fifth page code supported. Indicates if device server supports advanced failover or basic failover.

> **Note:** This page may not be supported by all library models.

**Table 15: Vendor Specific Device Capabilities Page**

```
            Bit      7          6          5         4        3            2             1               0
 Byte
        0           Peripheral Identifier                          Peripheral Device Type
        1                                                Page Code (C8h)
        2
                   MSB
        3                                                Page Length (4)
                                                                                                        LSB
        4                           Reserved                           ADVFO         BASICFO            RSVD
        5                                   Reserved                                         Reserved
        6                      Reserved                                        Reserved
        7                                                   Reserved
```

| Field | Description |
|---|---|
| Peripheral Qualifier | The return value of 000b indicates that the library is a single LUN device. If a LUN other than 00000b was specified, this field returns 011b which indicates that only LUN 0 is supported. |
| Peripheral Device Type | For media changer device logical units, this field returns 01000b (08h) to indicate it is a media changer device. Any LUN other than 0 returns 11111b which indicates that the device is unknown. |
| Page Code | Returned as C8h, indicating this page. |
| Page Length | Returns 4, indicating the remaining number of bytes following this field. |
| ADVFO | An advanced failover (ADVFO) bit set to one indicates the device server supports advanced path failover. An advanced failover (ADVFO) bit set to zero indicates the device server does not support advanced path failover. |
| BASICFO | A basic failover (BASICFO) bit set to one indicates the device server supports basic path failover. A BASICFO bit set to zero indicates support for basic path failover is not supported. |
