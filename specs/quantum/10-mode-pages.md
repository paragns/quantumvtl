# Mode Pages

The following table lists the mode pages supported by the library.

**Table 1: Supported Mode Pages**

| Page Code | SubPage Code | Page Name | Page Description | Device |
|-----------|-------------|-----------|-----------------|--------|
| 02h | 00h | Disconnect Reconnect | Provides information regarding the physical bus performance characteristics. | DA blade controller; media changer (only if hosted by a tape drive on LUN 0) |
| 18h | 00h | Fibre Channel Logical Unit Control | Provides Fibre Channel control information that is associated with the logical unit. | DA blade controller; media changer |
| 19h | 00h | Fibre Channel Port Control | Provides Fibre Channel control information that is associated with the port. | DA blade controller; media changer |
| 1Ch | 00h | Informational Exceptions Control | Provides information regarding SCSI tape alert processing within the library. | media changer |
| 1Dh | 00h | Element Address Assignment | Provides information regarding SCSI element address assignments and respective element ranges. | media changer |
| 1Eh | 00h | Transport Geometry Parameters | Provides information regarding the media changer device's capabilities. | media changer |
| 1Fh | 00h | Device Capabilities | Provides information regarding cartridge movement possibilities within the library. | media changer |
| 1Fh | 41h | Extended Device Capabilities | Provides information regarding media changer access capabilities for data transfer, IE, and storage elements. | media changer |
| Any supported page code above | FFh | Report all subpages for the requested specified page code. | Reports subpages for subcode 00h in page_0 format, and sub_page format for any other subpage (only 1Fh/41h reports sub_page format). | media changer |
| 3Fh | 00h | Report all Mode Pages | Returns all mode pages with no or subpage 00h in page_0 format. | DA blade controller; media changer |
| 3Fh | FFh | Report all sub pages for all Mode Pages | Reports subpages for subcode 00h in page_0 format, and sub_page format for any other subpage.. | media changer |

1. Supported by Scalar i7 RAPTOR only.
2. See Mode Page Header Format Types below for a definition of the page_0 and sub_page format types.
3. Supported by Sclar i500 and Scalar i6000 only.


## Mode Page Header Format Types

Each mode page contains a PS bit, an SPF bit, a PAGE CODE field, a PAGE LENGTH field, and a set of
mode parameters.

When using the MODE SENSE command, a parameters saveable (PS) bit set to one indicates that the
mode page may be saved by the logical unit in a nonvolatile, vendor specific location. A PS bit set to zero
indicates that the device is not able to save the supported mode parameters. When using the MODE
SELECT command, the PS bit is reserved.

A SubPage Format (SPF) bit set to zero indicates that the page_0 mode page format is being used to
report mode parameter information. A SPF bit set to one indicates that the sub_page mode page format
is being used. A sub_page mode page format applies to reporting non-zero subpage mode parameters
only.

**Table 2: Page_0 MODE Page format**

```
                 Bit      7            6            5           4          3           2    1     0
  Byte

          0              PS        SPF (0b)                                 Page Code
          1                                                   Page Length (n-1)
          2
         ....                                                 Mode Parameters
          N
```

**Table 3: Sub_page Mode Page format**

```
                Bit        7               6            5        4          3          2    1     0
  Byte

         0                PS         SPF (1b)                                  Page Code
         1                                                     SubPage Code
         2              (MSB)
                                                              Page Length (n-3)
         3                                                                                      (LSB)
         4
         ....                                                Mode Parameters
         N
```


## Disconnect-Reconnect Page (02h)

The Disconnect-Reconnect mode page is only supported by the controller device logical unit, and
describes the interconnect tenancy characteristics of the Fibre Channel interface. An interconnect
tenancy is a period of time during which a SCSI device owns or may access the interface. This page is
only available on the Fibre Channel interface.

**Table 4: Disconnect-Reconnect Page (02h)**

```
                Bit        7               6        5           4           3           2   1     0
  Byte
         0                PS         SPF (0b)                            Page Code (02h)
         1                                              Parameter List Length = 0Eh
         2                                                   Buffer Full Ratio
         3                                                   Buffer Empty Ratio
         4
                                                              Bus Inactivity Limit
         5
         6
                                                           Disconnect Time Limit
         7
         8
                                                             Connect Time Limit
         9
         10
                                                            Maximum Burst Size
         11
         12             EMDP                  Fair Arbitration                 DIMM                      DTDC
         13                                                         Reserved
         14
                                                                 First Burst Size
         15
```

| Field | Description |
|-------|-------------|
| Parameters Savable (PS) | This page is not savable, and this field is set to zero. |
| Subpage Format (SPF) | If the subpage format (SPF) bit is set to zero, the PAGE_0 mode page format is reported. If the SPF bit is set to one, the SUBPAGE mode page format is reported. |
| Page Code | This field identifies the Disconnect-Reconnect mode page and returns 02h. |
| Parameter List Length | This field is set to 0Eh (14). |
| Buffer Full Ratio | This field indicates how full the buffer will be (during read operations) prior to requesting an interconnect tenancy. A value of 0 is returned to indicate that requests for an interconnect tenancy are consistent with the Disconnect Time Limit field. |
| Buffer Empty Ratio | This field indicates how empty the buffer will be (during write operations) prior to requesting an interconnect tenancy (request for the initiator to send data). A value of 0 is returned to indicate that requests for an interconnect tenancy are consistent with the Disconnect Time Limit field. |
| Bus Inactivity Limit | This field indicates the maximum time limit allowed for maintaining an interconnect tenancy without any data or information transfer. A value of 0 is returned to indicate that there is no bus inactivity limit. |
| Disconnect Time Limit | This field indicates the minimum wait time between interconnect tenancies. A value of 0 is returned to indicate that there is no disconnect time limit. |
| Connect Time Limit | This field indicates the maximum duration of an interconnect tenancy. A value of 0 is returned to indicate that there is no connect time limit. |
| Maximum Burst Size | This field indicates the maximum amount of data that will be transferred during a single data transfer operation. The value is expressed in increments of 512 bytes. A value of 0 is returned if a maximum burst size is not supported. |
| Enable Modify Data Pointers (EMDP) | This field indicates whether data transfers are allowed to be re-ordered. A value of 0 is returned to indicate that data transfers will always have continually increasing and contiguous data relative offset values. A value of 1 is returned to indicate that data transfers can be re-ordered. |
| Fair Arbitration | This field indicates whether fair or unfair arbitration is used when requesting an interconnect tenancy. A value of 000b is returned to indicate that the various fairness algorithms may not be used. |
| Disconnect Immediate (DIMM) | A value of 0 is returned to indicate that data may be transferred for a command during the same interconnect tenancy in which the command was received. |
| Data Transfer Disconnect Control (DTDC) | A value of 000b is returned to indicate that data transfer disconnect control is not used. |
| First Burst Size | This field indicates the maximum amount of data that may be transferred along with a command. A value of 0 is returned to indicate that there is no first burst size limit. |


## Fibre Channel Logical Unit Control Page (18h)

The Fibre Channel Logical Unit Control mode page reports logical unit behavior for the Fibre Channel
Protocol. This page is only available on the Fibre Channel interface.

**Table 5: Fibre Channel Logical Unit Control Page (18h)**

```
          Bit       7             6             5           4           3           2             1              0
  Byte
      0            PS         SPF (0b)                                  Page Code (18h)
      1                                              Parameter List Length = 06h
      2                               Reserved                                     Protocol Identifier (0h)
      3                                                 Reserved                                                EPDC
      4                                                          Reserved
      5                                                          Reserved
      6                                                          Reserved
      7                                                          Reserved
```

| Field | Description |
|-------|-------------|
| Parameters Savable (PS) | This page is not savable, and this field is set to zero. |
| Subpage Format (SPF) | The subpage format (SPF) bit is set to zero to indicate the PAGE_0 mode page format is reported. |
| Page Code | This field identifies the Fibre Channel Logical Unit Control mode page and returns 18h. |
| Parameter List Length | This field is set to 06h. |
| Protocol Identifier | This field returns 0 to indicate the Fibre Channel protocol. |
| Enable Precise Delivery Checking (EPDC) | If this field returns 0, it indicates that the Fibre Channel Command Reference Number is not checked to verify that command packets are received in order. If this field returns 1, it indicates that the precise delivery checking is enabled and that the Fibre Channel Command Reference Number is checked to verify that command packets are received in order. |


## Fibre Channel Port Control Page (19h)

The Fibre Channel Port Control mode page reports port behavior for the Fibre Channel Protocol. This
mode page is only available on the Fibre Channel interface. This mode page is only supported by devices
at LUN 0.

**Table 6: Fibre Channel Port Control Page (19h)**

```
          Bit       7             6              5           4             3           2             1              0
  Byte
      0            PS         SPF (0b)                                    Page Code (19h)
      1                                              Parameter List Length = 06h
      2                               Reserved                                     Protocol Identifier (0h)
      3          DTFD           PLPB          DDIS             DLM          RHA         ALWI         DTIPE          DTOLI
      4                                                              Reserved
      5                                                              Reserved
      6                                    Reserved                                              RR_TOV Units
      7                                   Resource Recovery Time-out Value (RR_TOV)
```

| Field | Description |
|-------|-------------|
| Parameters Savable (PS) | This page is not savable, and this field is set to zero. |
| Subpage Format (SPF) | The subpage format (SPF) bit is set to zero to indicate the PAGE_0 mode page format is reported. |
| Page Code | This field identifies the Fibre Channel Port Control mode page and returns 19h. |
| Parameter List Length | This field is set to 06h. |
| Protocol Identifier | This field returns 0 to indicate the Fibre Channel protocol. |
| Disable Target Fabric Discovery (DTFD) | A DTFD bit of one indicates that if the target is attached by an arbitrated loop, it will not recognize the presence of a fabric loop port on the loop. The target will perform only the private loop functions defined for targets defined by FC-PLDA and FC-TAPE. When this bit is set to zero, and the target is attached by an arbitrated loop, it will discover a fabric loop port if present on the loop and perform the public loop functions defined for targets by FC-FLA. This field is ignored if the target is not attached to an arbitrated loop. |
| Prevent Loop Port Bypass (PLPB) | This field is set to zero to indicate that the target allows the Loop Port Bypass (LPB) and Loop Port Enable (PBE) primitive sequences to control the port bypass circuit and participation on the loop as specified by FC-AL-2. When not attached to an arbitrated loop, this field is ignored. |
| Disable Discovery (DDIS) | This field returns zero to indicate that the target will wait to complete target discovery as defined by FC-PLDA, FC-FLA, and FC-TAPE before allowing processing of tasks to resume. When not attached to an arbitrated loop, this field is ignored. |
| Disable Loop Master (DLM) | If this field returns zero, it indicates the target may participate in loop master arbitration in the normal manner and, if successful, may become loop master during the loop initialization process. If this field returns 1, it indicates the target does not become loop master, and that the target repeats LISM frames it receives. This allows the Initiator to be loop master during loop initialization. This field is ignored when not attached to an arbitrated loop. |
| Require Hard Address (RHA) | A RHA bit of one indicates that if the target is attached to an arbitrated loop, it will only attempt to obtain its hard address available in the SCA-2 SFF- 8067 connector or device address jumpers during loop initialization. The target will not attempt to obtain an address during the LISA phase of initialization. If there is a conflict for the hard address selection during loop initialization or the target does not have a valid hard address available, the target shall enter the nonparticipating state. If the target detects loop initialization while in the nonparticipating state, the target will again attempt to get its hard address. If the hard address has not changed from the address obtained in a previous successful loop initialization, the target will attempt to obtain the address in the LIFA phase if a valid Fabric Login exists or LIPA phase of loop initialization. If the hard address has changed, the target will attempt to obtain the new address in the LIHA phase. When the RHA bit is set to zero, the target follows the normal initialization procedure, including the possibility of obtaining a soft address during the loop initialization process. When not attached to an arbitrated loop, this field is ignored the RHA bit. |
| Allow Login without Loop Initialization (ALWI) | This field returns zero to indicate the target will perform the normal loop initialization procedure before entering the monitoring mode and accepting a login ELS. This field is ignored when not attached to an arbitrated loop. |
| Disable Target Initiated Port Enable (DTIPE) | This field returns zero to indicate the target will enable itself onto the loop according to the rules specified in FC-AL-2. This field is ignored when not attached to an arbitrated loop. |
| Disable Target Originated Loop (DTOLI) | This field returns zero to indicate the target attached by an arbitrated loop will generate LIP(F7,xx) after it enables a port into a loop. If the target is attached to an arbitrated loop and detects loop failure at its input, it shall follow the error initialization process defined by FC-AL-2 regardless of the state of this bit. This field is ignored when not attached to an arbitrated loop. |
| RR_TOV Units | This field indicates the units for the Resource Recovery Time-out Value field. A value of 3 is returned to indicate the units are in tenths of seconds. |
| Resource Recovery Time-out Value (RR_TOV) | This field returns the resource recovery time-out value specified in RR_TOV units. For example, an RR_TOV of 14h (20 decimal) with an RR_TOV units value of 3 indicates a resource recovery time-out value of 2 seconds; and an RR_TOV of F0h (240 decimal) with an RR_TOV units value of 3 indicates a resource recovery time-out value of 24 seconds. |


## Informational Exceptions Control Page (1Ch)

The Informational Exceptions Control mode page describes the capabilities of the library for reporting
exception conditions. It was previously known as the Tape Alert mode page when exception conditions
were limited to only Tape Alert flags.

The main purpose of this page is to indicate that the library can report exception conditions by being
polled. The exception conditions primarily involve the Tape Alert flags, but may include additional
conditions as well, as defined by the Sense Data.

**Table 7: Informational Exceptions Control Page (1Ch)**

```
          Bit      7            6            5                4               3             2            1             0
  Byte
      0           PS        SFP (0b)                                      Page Code (1Ch)
      1                                               Parameter List Length = 0Ah
      2          Perf         Rsvd         EBF            EWasc           Dexcpt          Test          Rsvd       LogErr
      3                             Reserved                                                     MRIE
      4
      :                                                        Interval Timer
      7
      8
      :                                                        Report Count
     11
```

| Field | Description |
|-------|-------------|
| Parameters Savable (PS) | This page is not savable, and this field is set to zero. |
| Subpage Format (SPF) | The subpage format (SPF) bit is set to zero to indicate the PAGE_0 mode page format is reported. |
| Page Code | This field identifies the Informational Exceptions Control mode page and returns 1Ch. |
| Parameter List Length | This field is set to 0Ah (10). |
| Log Errors (LogErr) | This field is set to 0 to indicate that logging of informational exception conditions is vendor specific (unique to the library in this case). |
| Test | This field is set to 0 to indicate that test failure indications will not be generated. |
| Disable Exception Control (Dexcpt) | This field is set to 1, indicating that the initiator must poll the LOG SENSE Tape Alert page. |
| Enable Warning (EWasc) | This field is set to 0, indicating that reporting of warnings is disabled. |
| Enable Background Function (EBF) | This field is set to 0 indicating that background functions are not enabled. |
| Performance (Perf) | This field is set to 0 to indicate that informational exception operations that are the cause of delays are acceptable. |
| Method of Reporting Informational Exceptions (MRIE) | This field is set to 0h to indicate that exception conditions or warnings will not be reported, and that the initiator must poll. |
| Interval Timer | This field is set to 0000 0000h to indicate that the interval is vendor specific. The library does not support a timer interval. |
| Report Count | This field is set to 0000 0000h to indicate that there is no limit on the number of exception conditions reported. |


## Element Address Assignment Page (1Dh)

The Element Address Assignment mode page returns the first element address and the element quantity
for each element type. The quantity is based on the number of elements configured in the library, some of
which may be temporarily removed (like a storage magazine or drive). Elements that are temporarily
removed will not change the overall number of elements for that element type. Table 8 on the next page
shows the format of the page. Initiators should always retrieve this page and use these values when
communicating element-based commands with the library. The addresses and quantities of elements
should never be assumed or hard-coded by the initiator, as they are subject to change.

**Table 8: Element Address Assignment Page (1Dh)**

```
          Bit       7              6                5          4              3           2             1                0
  Byte
      0            PS         SPF (0b)                                    Page Code (1Dh)
      1                                                 Parameter List Length = 12h
      2
                                        First Medium Transport Element Address (0001h)
      3
      4
                                              Number of Medium Transport Elements
      5
      6
                                              First Storage Element Address (1000h)
      7
      8
                                                        Number of Storage Elements
      9
     10
                                          First Import/Export Element Address (0010h)
     11
     12
                                                    Number of Import/Export Elements
     13
     14
                                          First Data Transfer Element Address (0100h)
     15
     16
                                                Number of Data Transfer Elements
     17
     18
                                                                   Reserved
     19
```

The following information lists the default SCSI addressing scheme. Different settings are supproted
depending on product model, such that the Scalar i6000 may configure the drive starting element
address as 0x0100 or 0x0800, and the Scalar i7 RAPTOR may report DT elements starting at 0x0010,
and IE elements at 0x0100 instead.

| Field | Description |
|-------|-------------|
| Parameters Savable (PS) | This page is not savable, and this field is set to zero. |
| Subpage Format (SPF) | The subpage format (SPF) bit is set to zero to indicate the PAGE_0 mode page format is reported. |
| Page Code | This field identifies the Element Address Assignment mode page and returns 1Dh. |
| Parameter List Length | This field is set to 12h (18). |
| First Medium Transport Element Address | This returns 0001h, which is the address of the first medium transport element (accessor). |
| Number of Medium Transport Elements | This field returns 0001h. |
| First Storage Element Address | This field returns 1000h, which is the address of the first storage element. |
| Number of Storage Elements | This field varies, depending on the configuration of the subsystem. |
| First Import/Export Element Address | This field returns 0010h, which is the address of the first Import/Export element. |
| Number of Import/Export Elements | This field varies, depending on the configuration of the subsystem. If no Import/Export elements are installed, this field returns zero. |
| First Data Transfer Element Address | This field returns 0100h, which is the address of the first data transfer element (drive). |
| Number of Data Transfer Elements | This field varies, depending on the configuration of the subsystem. |


## Transport Geometry Parameters Page (1Eh)

The Transport Geometry Parameters page describes whether a medium transport element is a member
of a set of elements that share a common robotics subsystem, and whether it is capable of handling
double-sided media. Libraries currently contain a single medium transport element, so all are the first
element in a set of one.

**Table 9: Transport Geometry Parameters Page (1Eh)**

```
          Bit       7             6             5         4             3             2           1               0
  Byte
      0            PS         SPF (0b)                                Page Code (1Eh)
      1                                             Parameter List Length = 02h
      2                                                Reserved                                              Rotate
      3                                     Member Number In Transport Element Set
```

| Field | Description |
|-------|-------------|
| Parameters Savable (PS) | This page is not savable, and this field is set to zero. |
| Subpage Format (SPF) | The subpage format (SPF) bit is set to zero to indicate the PAGE_0 mode page format is reported. |
| Page Code | This field identifies the Transport Geometry Parameters mode page and returns 1Eh. |
| Parameter List Length | This field is set to 02h, since only a single medium transport is reported. |
| Rotate | This field returns 0, since double-sided media is not supported. |
| Member Number In Medium Transport Element Set | This field returns 0, since the library has a single medium transport. |


## Device Capabilities Page (1Fh)

The Device Capabilities page defines the rules governing cartridge movement within the library. It
describes from which element type to the next a cartridge can be moved, directly defining which element
types can be used as either source or target elements. The library does not allow the medium transport
element (accessor) to be a target, and only as a source on a limited basis.

Abbreviations are as follows:
- MT - Medium Transport device
- DT - Data Transfer device
- ST - Storage element
- IE - Import/Export element

**Table 10: Device Capabilities Page (1Fh)**

```
          Bit     7            6            5            4             3                2              1             0
  Byte
      0          PS         Rsvd                                         Page Code (1Fh)
      1                                               Parameter List Length = 12h
      2                         Reserved                           StorDT 1         StorIE 1       StorST 1     StorMT 0
      3                                    Reserved                                ACE 0 or 1      VTRP 1           S2C 1
                                                                                   MT > I/E        MT > ST      MT > MT
      4           MT -> RA 00b               Reserved             MT > DT 0
                                                                                     x              x              0
                                                                                                                ST > MT
      5           ST -> RA 00b               Reserved             ST > DT 1        ST > I/E 1     ST > ST 1
                                                                                                                   0
                                                                                                 I/E > ST       I/E > MT
      6              IE -> RA 00b            Reserved             I/E > DT 1      I/E > I/E 1
                                                                                                     1              0
                                                                                                 DT > ST       DT > MT
      7             DT -> RA 00b             Reserved             DT > DT 1       DT > I/E 1
                                                                                                    1             0
      8
      :                                                         Reserved
     11

     12            MT -> WA 00b                                                  MT <> IE            MT <>       MT <>
                                            Reserved            MT<>DT 0
                                                                                    0                ST 0        MT 0

     13            ST -> WA 00b                                                   ST <> IE      ST <> ST         ST <>
                                            Reserved             ST<>DT 1
                                                                                     1              0            MT 0

     14            IE -> WA 00b                                                   I/E <> IE     I/E <> ST        I/E <>
                                            Reserved             I/E<>DT 1
                                                                                      1              1            MT 0

     15            DT -> WA 00b                                                   DT <> IE           DT <>       DT <>
                                            Reserved             DT<>DT 1
                                                                                     1               ST 1        MT 0

     16
     ...                                                        Reserved

     19

                 Depending on the tape library model , x may be set to 1 or 0 to indicate support accordingly.
```

| Field | Description |
|-------|-------------|
| Parameters Savable (PS) | This page is not savable, and this field is set to zero. |
| Subpage Format (SPF) | The subpage format (SPF) bit is set to zero to indicate the PAGE_0 mode page format is reported. |
| Page Code | This field identifies the Device Capabilities mode page and returns 1Fh. |
| Parameter List Length | This field is set to 0Eh (14). |
| Store in Data Transfer (StorDT) | This field is set to 1 to indicate that the data transfer elements (drives) can store cartridges. |
| Store in Import/Export (StorI/E) | This field is set to 1 to indicate that the Import/Export elements can store cartridges. |
| Store in Storage (StorST) | This field is set to 1 to indicate that the storage elements can store cartridges. |
| Store in Medium Transport (StorMT) | This field is set to 0 to indicate that the accessor cannot store cartridges. |
| SMC-2 Capabilities (S2C) | This bit is set to 1 to indicate that VTRP, ACE, XX-RA and XX-WA fields are supported. |
| Volume Tag Reader Present (VTRP) | The volume tag reader present (VTRP) bit is set to 1, indicating that a volume tag reader (barcode scanner or camera) is installed in the media changer. |
| Auto-Clean Enabled (ACE) | An auto-clean enabled (ACE) bit set to 1 indicates that the library partition is managing the data transfer element cleaning process. An ACE bit set to 0 indicates that the logical library partition is not managing the cleaning process. |
| XX-RA and XX-WA | The XX-RA and XX-WA fields indicate the resources required to support a READ ATTRIBUTE command (RA) and WRITE ATTRIBUTE command (WA) for each element type. Since the medium changer does not participate or support an Read Attribute or Write Attribute commands, these fields report 00b to indicate that MAM access is not available. |
| X > Y, vs. X <> Y | X > Y Denotes a move capability from x to Y, X <> Y denotes an Exchange capability between X and Y |

The remaining element type to element type fields describe the allowable source to target transitions. A
zero is returned for any transition involving the Medium Transport (MT) except for when the MT is a
source and the destination is either I/E or Storage. A one is returned for all other transitions.


## Extended Device Capabilities Mode Page (1Fh/41h)

The Extended Device Capabilities mode page defines characteristics of the Media Changer with respect
to media movement for IE, storage and data transfer elements.

**Table 11: Extended Device Capabilities Mode Page (1Fh/41h)**

```
      Bit       7          6           5               4              3              2               1              0
  Byte
               PS        SPF
     0                                                                Page Code (1Fh)
               (0b)      (1b)
     1                                                       Subpage (41h)
     2
                                                          Page Length = 10h
     3

     4           Reserved           MVPRV           MVCL         MVOP           USRCL           USROP            IEST

     5                  Reserved                    DTEDA        RSSEA          MVTRY           IEMGZ           SMGZ

     6                                  Reserved                                TREXC            LCKIE           LCKD

     7                                  Reserved                                SPMER           DPMER           PEPOS

     8                                                Reserved                                                   UCST
    9
    ...                                                       Reserved
    19
```

| Field | Description |
|-------|-------------|
| Parameters Savable (PS) | This page is not savable, and this field is set to zero. |
| Subpage Format (SPF) | The subpage format (SPF) bit is set to one to indicate the SUBPAGE mode page format is reported. |
| Page Code | This field identifies the Device Capabilities mode page and returns 1Fh. |
| Subpage | This field reports value 41h, and identifies that the Extended Device Capabilities mode page is being and returned. |
| Page Length | This field is set to 10h to indicate 16 bytes of data follow. |
| Import/Export Element State (IEST) | An import/export element state (IEST) bit set to one indicates that the media changer is able to detect medium presence in all import/export elements. An IEST bit set to zero indicates that the media changer is not able to detect medium presence in all import/export elements. The value returned is 1. |
| User Control Import/Export Element Open (USROP) | A user control import/export element open (USROP) bit set to one indicates that the media changer requires the operator to manually open a closed import/export element. An USROP bit set to zero indicates that the media changer does not require the operator to manually open a closed import/export element. The value returned is 1. |
| User Control Import/Export Element Close (USRCL) | A user control import/export element close (USRCL) bit set to one indicates that the media changer requires the operator to manually close an open import/export element. An USRCL bit set to zero indicates that the media changer does not require the operator to manually close an open import/export element. The value returned is 1. |
| Move Opens Import/Export element (MVOP) | A move opens import/export element (MVOP) bit set to one indicates that the media changer opens the import/export element for operator access whenever a command is issued to move media with an import/export element as a destination element address. An MVOP bit set to zero indicates that the media changer does not open the import export element for operator access whenever a command is issued to move media with an import/export element as a destination element address. The value returned is 0. |
| Move Prevented to Import/Export Element (MVPRV) | A move prevented to import/export element (MVPRV) bit set to one indicates that the media changer prevents moves with the import/export element as a destination element address when medium removal is prevented with the PREVENT ALLOW MEDIUM REMOVAL command. An MVPRV bit set to zero indicates that the media changer does not prevent moves with the import/export element as a destination element address when medium removal is prevented with the PREVENT ALLOW MEDIUM REMOVAL command. The value returned is 1. If media removal is prevented the media changer does not lock the IE station(s) and allows IE access for library managed operations such as export operations of cleaning tapes or operator directed import operations. |
| Storage Magazine (SMGZ) | A storage magazine (SMGZ) bit set to one indicates that the media changer uses media magazines for some storage elements. A SMGZ bit set to zero indicates that the media changer does not use media magazines for any storage element. The value returned is 1. |
| Import/Export Magazine (IEMGZ) | An import/export magazine (IEMGZ) bit set to one indicates that the media changer uses media magazines for some import/export elements. An IEMGZ bit set to zero indicates that the media changer does not use media magazines for any import/export element. The value returned is 1. |
| Move Tray (MVTRY) | A move tray (MVTRY) bit set to one indicates that the media changer requires the medium to be placed in a tray and the tray moved to the desired position as part of a move operation (e.g., a CD-ROM must be placed in a tray, then loaded into a data transfer device). A MVTRY bit set to zero indicates that the media changer does not use trays. The value returned is 0. |
| Return to Source Storage Element Address (RSSEA) | A return to source storage element address (RSSEA) bit set to one indicates that the media changer is only able to move a volume from a data transfer device to the element address from which it A RSSEA bit set to zero indicates that the media changer is able to move a volume from a data transfer device to an element address other than the element address from which it came. The value returned is 0. |
| Data Transfer Element Empty on Door Access (DTEDA) | A data transfer element empty on door access (DTEDA) bit set to one indicates that the media changer does not allow the door to be opened if any data transfer element contains a volume. A DTEDA bit set to zero indicates that the media changer does not prohibit opening the door if a data transfer element contains a volume. The value returned is 0. |
| True Exchange Capable (TREXC) | A true exchange capable (TREXC) bit set to one indicates that the media changer allows an EXCHANGE MEDIUM command that has the second destination element address equal to the source element address. A TREXC bit set to zero indicates that the media changer does not allow an EXCHANGE MEDIUM command that has the second destination element address equal to the source element address. The value returned is 1. |
| Lock Door (LCKD) | A lock door (LCKD) bit set to one indicates that the PREVENT ALLOW MEDIUM REMOVAL command with the PREVENT field set to 01b secures the media changer door(s). An LCKD bit set to zero indicates that the PREVENT ALLOW MEDIUM REMOVAL command with the PREVENT field set to 01b does not secure the media changer door(s). The value returned is 0. |
| Lock Import/Export Element (LCKIE) | A lock import/export element (LCKIE) bit set to one indicates that the PREVENT ALLOW MEDIUM REMOVAL command with the PREVENT field set to 01b secures the media changer import/export element(s). An LCKIE bit set to zero indicates that the PREVENT ALLOW MEDIUM REMOVAL command with the PREVENT field set to 01b does not secure the media changer import/export element(s). The value returned is 1. |
| Pre-Eject Position (PEPOS) | A pre-eject position (PEPOS) bit set to one indicates that the media changer requires a POSITION TO ELEMENT command to position the medium transport element to a data transfer element before an eject. A PEPOS bit set to zero indicates that the media changer does not require a POSITION TO ELEMENT command to position the medium transport element to a data transfer element before an eject. The value returned is 0. |
| Destination Pre-Move Eject Required (DPMER) | A destination pre-move eject required (DPMER) bit set to one indicates that the media changer requires an application client to send a command to the data transfer device to prepare the data transfer device to accept a move to this data transfer element (e.g., command a CD-ROM drive to extend the tray). A DPMER bit set to zero indicates that an application client does not need to send a command to the data transfer device to accept a move to this data transfer element. The value returned is 0. |
| Source Pre-Move Eject Required (SPMER) | A source pre-move eject required (SPMER) bit set to one indicates that the media changer requires an application client to send a command to the data transfer device to prepare the data transfer device to accept a move from this data transfer element. A SPMER bit set to zero indicates that an application client does not need to send a command to the data transfer device to prepare the data transfer device to accept a move from this data transfer element. This value may be returned as 0 or 1 and depends on the library partition setting to have the library assist with unload operations or not, respectively. |
| Unassigned Cleaning Storage (UCST) | An unassigned cleaning storage (UCST) bit set to one indicates that the device supports physical entities that contain cleaning volumes and that do not have assigned element addresses. These unassigned physical entities are not reported in the READ ELEMENT STATUS data. A UCST bit set to zero indicates that the device does not support physical entities that contain cleaning volumes and that do not have assigned element addresses. This value may be returned as 1 or 0 and depends on the library partition setting to have the library perform library-assisted, automatic cleaning or not, respectively. |

> **Note:** It is good practice to always issue an unload/eject request to the drive prior to requesting media to be moved from a drive, regardless of the value reported in the SPMER bit.


## Specific SubPage Mode Pages (xxh/FFh)

When this page is requested, all supported subpages for the specified mode page (xxh) are reported in
ascending order, reporting subpage 00h in page_0 mode page format and any defined additional
subpages in sub_page mode page format.


## All Mode Pages (3Fh/00h)

When this page is requested, all supported mode pages are returned in ascending order.


## All SubPage Mode Pages (3Fh/FFh)

When this page is requested, all supported subpages for all supported mode pages are reported in
ascending order, reporting any subpage 00h in page_0 mode page format, and any defined additional
subpages in sub_page mode page format.
