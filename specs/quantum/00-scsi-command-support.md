# SCSI Command Support

## Device Model

Quantum intelligent libraries support both a media changer device (device type 08h) and, depending on
the library model and configuration, a controller device (device type 0Ch). The controller device is used
primarily to aid initialization and discovery for the servers in conjunction with the library's Logical Library
model. Only the Scalar i2000/i6000 and Scalar i500 library models support controller device (DA blade)
configurations. Library models without DA blades configure tape drives to host the library control path to
the logical library partition and hereby provide access to the media changer devices.

> **Note:** "DA blade" is a generic term used in this manual to describe a Fibre Channel (FC)-to-FC
> I/O blade. The DA blade controller device is not a pure controller device, in that it does not support
> all the mandatory commands defined by SCC. This is an accepted industry practice established by
> vendors of bridges and routers.

This device model approach works by having the controller device or tape drive (typically at LUN 0)
respond to commands directed at LUN 0, including the REPORT LUNS command, and the media
changer device(s) (one media changer device per logical library partition) typically respond to commands
on LUN1. This approach allows an initiator (host) to issue a REPORT LUNS command to the controller
device to retrieve a listing of all available logical units, to determine the presence of connected Data
Transfer Device(s) and media changer device(s).


## DA Blade Controller Device Commands and Parameters

The following table lists the commands supported by the DA blade controller device.

**Table 1: DA Blade Controller Device Supported Commands**

| Command | Code |
|---|---|
| INQUIRY | 12h |
| MODE SELECT (6) | 15h |
| MODE SELECT (10) | 55h |
| MODE SENSE (6) | 1Ah |
| MODE SENSE (10) | 5Ah |
| READ BUFFER | 3Ch |
| REPORT LUNS | A0h |
| REQUEST SENSE | 03h |
| TEST UNIT READY | 00h |
| WRITE BUFFER | 3Bh |

The following table lists the parameters supported by the DA blade controller device.

**Table 2: DA Blade Controller Device Supported Parameters**

| Command | Page | Code |
|---|---|---|
| Inquiry | Supported VPD Pages | 00h |
| Inquiry | Unit Serial Number Page | 80h |
| Inquiry | Device Identification Page | 83h |
| Mode Select/Sense | Disconnect Reconnect Page | 02h |
| Mode Select/Sense | FC LU Control Page | 18h |
| Mode Select/Sense | FC Port Control Page | 19h |
| Mode Sense | Return all pages | 3Fh |


## Media Changer Commands and Parameters

The following table lists the commands supported by the media changer device.

**Table 3: Media Changer Supported Commands**

| Command | Code |
|---|---|
| Exchange Medium - A6h on page 15^4 | A6h |
| Initialize Element Status - 07h on page 18 | 07h |
| Initialize Element Status With Range - E7h/37h on page 20 | E7h |
| Inquiry - 12h on page 22^1 | 12h |
| Log Sense - 4Dh on page 40 | 4Dh |
| Mode Select (6) - 15h on page 51 | 15h |
| Mode Select (10) - 55h on page 53 | 55h |
| Mode Sense (6) - 1Ah on page 55 | 1Ah |
| Mode Sense (10) - 5Ah on page 58 | 5Ah |
| Move Medium - A5h on page 82 | A5h |
| Persistent Reserve In - 5Eh on page 84^1 | 5Eh |
| Persistent Reserve Out - 5Fh on page 93^1 | 5Fh |
| Position to Element - 2Bh on page 101 | 2Bh |
| Prevent Allow Medium Removal - 1Eh on page 103 | 1Eh |
| Read Buffer - 3Ch on page 105^1 | 3Ch |
| Read Element Status - B8h on page 110 | B8h |
| Release Element (6) - 17h on page 126^1 | 17h |
| Release Element (10) - 57h on page 127^1 | 57h |
| Report Element Information - 9Eh on page 128^5 | 9Eh |
| Report LUNS - A0h on page 143^2 | A0h |
| Request Sense - 03h on page 145^1 | 03h |
| Request Volume Element Address - B5h on page 154^3 | B5h |
| Reserve Element (6) - 16h on page 158^1 | 16h |
| Reserve Element (10) - 56h on page 159^1 | 56h |
| Send Diagnostic - 1Dh on page 161 | 1Dh |
| Send Volume Tag - B6h on page 162^3 | B6h |
| Test Unit Ready - 00h on page 165^1 | 00h |
| Write Buffer - 3Bh on page 166^1 | 3Bh |

1. If the library control path is configured via a tape drive, this command is processed by the
   tape drive on behalf of the media changer device.
2. This command is supported by a controller device or tape drive hosting a media changer
   device.
3. Supported only by Scalar i2000 and Scalar i6000.
4. Supported only by Scalar i7 RAPTOR
5. Supported only by Scalar i3/i6/i6H and Scalar i7 RAPTOR.

The following table lists the parameters supported by the media changer device. The media changer
device does not support any diagnostic parameters.

**Table 4: Media Changer Supported Parameters**

| Command | Page | Code | SubPage Code |
|---|---|---|---|
| Inquiry | Supported Vital Product Data Page (00h) on page 29 | 00h | 00h |
| Inquiry | Unit Serial Number Page (80h) on page 30 | 80h | 00h |
| Inquiry | Device Identification Page (83h) on page 31 | 83h | 00h |
| Inquiry | Management Network Addresses Page (85h) on page 36 | 85h | 00h |
| Inquiry | Vendor Specific Device Capabilities Page (C8h) on page 38 | C8h | 00h |
| Log Sense | Supported Log Page (00h) on page 44 | 00h | 00h |
| Log Sense | Temperature Log Page (0Dh) on page 45^2 | 0Dh | 00h |
| Log Sense | Temperature Log Parameter Field Description (see Table 5 on the next page). Tape Alert Response Log Page (12h) on page 46^2 | 12h | 00h |
| Log Sense | Tape Alert Response Log Parameter Field Description (see Table 6 on the next page). Tape Alert Log Page (2Eh) on page 47 | 2Eh | 00h |
| Log Sense | Humidity Log Page (30h) on page 49^2 | 30h | 00h |
| Mode Sense | Disconnect-Reconnect Page (02h) on page 63^1 | 02h | 00h |
| Mode Sense | Fibre Channel Logical Unit Control Page (18h) on page 65^1 | 18h | 00h |
| Mode Sense | Fibre Channel Port Control Page (19h) on page 66^1 | 19h | 00h |
| Mode Sense | Informational Exceptions Control Page (1Ch) on page 69 | 1Ch | 00h |
| Mode Sense | Element Address Assignment Page (1Dh) on page 71 | 1Dh | 00h |
| Mode Sense | Transport Geometry Parameters Page (1Eh) on page 72 | 1Eh | 00h |
| Mode Sense | Device Capabilities Page (1Fh) on page 73^2 | 1Fh | 41h |
| Mode Sense | Extended Device Capabilities Mode Page (1Fh/41h) on page 75 | 1Fh | 00h |
| Mode Sense | All Mode Pages (3Fh/00h) on page 81 | 3Fh | 00h |
| Mode Sense | Specific SubPage Mode Pages (xxh/FFh) on page 80 (reporting subpages in the page_0 format for subpage 00h and in the sub_page format for subpages 01h - FEh)^2 | 00h - 3fh | FFh |
| Mode Sense | All SubPage Mode Pages (3Fh/FFh) on page 81 (reporting subpages in the page_0 format for subpage 00h and in the sub_page format for subpages 01h - FEh)^2 | 3fH | FFh |

1. If the library control path is provided by a tape drive, this page is provided by the tape drive.
2. Supported by the Scalar i7 RAPTOR only

**Table 5: Temperature Log Parameter Field Description**

| Parameter Code | DU | TSD | ETC | TMC | Format and Linking | Parameter Length |
|---|---|---|---|---|---|---|
| 0000h | 0 | 0 | 0 | 00b | 00b | 2 |

**Table 6: Tape Alert Response Log Parameter Field Description**

| Parameter Code | DU | TSD | ETC | TMC | Format and Linking | Parameter Length |
|---|---|---|---|---|---|---|
| 0000h | 0 | 1 | 0 | 00b | 00b | 08h |


## General Command Support Behavior

### Multiple Initiator Support

Multiple initiators are not supported on the parallel SCSI or SAS interfaces. Information such as Unit
Attentions and SCSI sense data will be held for only a single initiator. Multiple initiators are supported on
the Fibre Channel interface. Unit attentions and SCSI sense data will be held for each initiator.

### Element Type Codes

| Code | Element Type Definition |
|---|---|
| 0000b (0) | All element types |
| 0001b (1) | Medium transport element (accessor) |
| 0010b (2) | Storage element |
| 0011b (3) | Import/Export element |
| 0100b (4) | Data transfer element (drives) |

### Element Addressing

The element-addressing model follows that of previous Quantum libraries. The starting addresses of the
four element types are:

- **0001h**: Medium Transport.

  > **Note:** Depending on product model, one or two physical media changers (robots) may be
  > installed, but only a single media changer device will be virtualized and reported.

- **0010h**: Import/Export
- **0100h**: Data Transfer
- **1000h**: Storage

### Command Status

Individual command status responses are not documented, as they all follow the same general format as
described here. After processing any command, the library returns status from among the following:

| Status | Description |
|---|---|
| Good | The library returns a Good status (00h) when it is able to process the command without errors. |
| Busy | The library returns Busy status (08h) when a motion command is still being processed, or the library is generally not able to process additional commands at that time. |
| Reservation Conflict | The library returns a Reservation Conflict (18h) whenever an initiator attempts to access a logical unit that has been reserved by another initiator, except for the following commands: INQUIRY, LOG SENSE, MODE SENSE (only if the library control path is configured through a tape drive), PREVENT/ALLOW MEDIUM REMOVAL, READ ELEMENT STATUS (only when the Current Data [CurData] field is set to 1), REPORT ELEMENT INFORMATION, REPORT LUNS, REQUEST SENSE, TEST UNIT READY |
| Check Condition | The library returns the Check Condition status (02h) when the following general situations occur (all generate sense data): The library is Not Ready (sense key 02h), The library has encountered a Hardware Error (sense key 04h), A parameter in the CDB is invalid or there is an invalid field in a parameter list resulting in an Illegal Request (sense key 05h), A Unit Attention condition is pending (sense key 06h), A command has been aborted (sense key 0Bh) |

For a complete list of all possible sense data and their causes, refer to Request Sense - 03h on page 145.
This status information will not be separated by individual commands.
Status values of Condition Met, Intermediate Condition Met, and Queue Full are not currently used. The
Initiator should issue a Request Sense command to determine the precise cause of the Check Condition
status and clear it.
Response data, however, will be documented as applicable for each command, and included as part of
the command section.

### Unit Attentions

Unit Attentions will be queued by the library as necessary to report all events and conditions. They are
presented in the order of their occurrence (first in, first out). Unit attentions are generated for the following
conditions:

- A power on or a reset (external or internal) occurred
- A library door closed, or a transition from not ready to ready occurred
- An import/export station or magazine closed
- An element status change for storage, data transfer or I/E elements.
- A firmware update completed
- A persistent reservation has been preempted or released, or a registration has been preempted
- Mode parameters have changed

### Resets

Either a Power On Reset or a SCSI Reset resets the library. When reset, the library does the following:

- Clears all non-persistent reservations
- Clears Prevent/Allow Medium Removal settings

### Common CDB Fields

Each Command Descriptor Block contains a Logical Unit Number (LUN) field as well as a Control byte
field. The LUN field is bits 5-7 of byte 1 and is there only for legacy compatibility. Logical Unit selection
should be accomplished via the Identify message.
The Control byte is shown in the following table. It is always the last byte of a CDB, regardless of the size
of the CDB.

**Table 7: Control Byte**

```
              Bit    7          6            5          4           3           2           1          0
Byte
       last         Vendor Specific                 Reserved                 NACA         Flag        Link
```

| Field | Description |
|---|---|
| Vendor Specific | This field is used to provide additional data or control for a command. Specific uses (if any) are described within the applicable commands. |
| Normal Auto Contingent Allegiance (NACA) | If this field is set to 0, the initiator should issue a REQUEST SENSE command immediately following receipt of a Check Condition. If this field is set to 1, ACA support will be provided. |
| Flag | This field is not supported and must be set to 0. |
| Link | This field is not supported and must be set to 0. |

### Reserved Fields

Reserved fields are not checked, and no error will be sent if they contain non-zero values.

### Vendor Specific Fields

Vendor Specific fields are not checked and no error will be sent if they contain non-zero values, unless
specific uses are defined within a SCSI command and vendor specific settings are required.

### Online/Offline Operation

Each media changer device can be placed in either an online or offline mode. The purpose of these
modes is to configure whether the media changer device is being controlled by a SCSI initiator or by the
local or remote user interface. When the media changer device is online, a SCSI initiator is controlling it
and all SCSI commands are supported. When the media changer device is offline, the local or remote
user interface is controlling it, and the only allowed SCSI commands are

- INQUIRY
- REPORT LUNS
- REQUEST SENSE
- TEST UNIT READY
- MODE SENSE
- READ ELEMENT STATUS. The command is allowed with DVCID=1, VOLTAG=0 while the library is
  offline.

All other commands will respond with a check condition, along with an ASC/ASCQ of 04/8Dh (Logical unit
offline) or 04/12h (Logical unit not ready, offline).

> **Note:** If the SCSI media changer device is configured via a library control path-enabled tape drive,
> RESERVE and RELEASE commands are also accepted and handled when the library is reporting
> offline status.

The DA blade controller device SCSI command set is not affected by the online/offline mode of the media
changer device.


## Supported Interfaces

The library supports SAS and Fibre Channel interface types.

### Fibre Channel Interface Support

Fibre Channel connections support configurations for Point-To-Point and Loop topologies. While tape
drives support Point-To-Point (N), Loop (L) as well as preferred connections for Point-To-Point Preferred
(NL) and Loop Preferred (LN), DA blade configurations support selections for Class 3 Loop-Preferred
connections only. Private arbitrated loops are supported by DA blades if the library is not attached to a
Fibre Channel fabric, and public arbitrated loops are supported if the library is attached to a Fibre
Channel fabric.

### SAS Interface Support

SAS interface connections support direct connections. Configuration options do not exist.


## Logical Libraries

The underlying physical library is not exposed externally to applications. Rather, logical representations
of media changer devices are created, and these are presented instead. Through this method the
physical library can be partitioned and concurrently shared in a heterogeneous environment. Storage and
Data Transfer elements cannot be shared across logical libraries; they can only be assigned to one
logical library at a time. The Medium Transport element (the robotic mechanism) is shared across all
logical libraries, and as a result there may be some delays encountered as each logical library waits its
turn for this shared resource.
Depending on library model, Import/Export elements can also be shared across logical libraries. This is
further discussed in Mailbox Behavior below. All other aspects of the logical media changer devices are
identical to an independent physical media changer device.


## Mailbox Behavior

The following characteristics affect Import/Export elements:

- The Import/Export elements are contained in removable magazines. When the magazines are
  removed, the elements are still counted in the number of Import/Export elements and will have
  element descriptors returned for them in response to a READ ELEMENT STATUS command. Their
  element status will indicate that they are not accessible, and will also report an exception with an
  ASC/ASCQ of 3B/12.
- Some libraries are configured with multiple physical mailboxes, each containing their own set of
  magazines. Whenever a mailbox is opened, the status for the elements it contains will indicate that
  they are not accessible until the mailbox is closed again.
- The Import/Export magazines can be assigned to and shared by different logical libraries. The
  Import/Export elements they contain are then used on a "first come, first served" basis. When shared
  Import/Export elements are in use by one logical library, element status for those elements will indicate
  that they are full, but not accessible for all other logical libraries that share them. The presence of
  media and associated volume tag information will only be available to the logical library using the
  elements at that time. When media is removed from the shared elements, they become available for
  use by any logical library requesting them, and their element status indicates that they are accessible.

These characteristics require applications to process the complete element status returned in the
element descriptors (including accessibility and exception conditions) to achieve optimum usage of the
Import/Export elements. Reliance on only full or empty element status may result in failed operations
(e.g., an Export). This might be due to not locating a usable Import/Export element when several may
actually be available, if the search had only taken into account full or empty status rather than
accessibility.


## Drive Cleaning

The library always evaluates drive cleaning needs after unloading data cartridges. If a drive cleaning
need is determined, the library will report, suppress, or perform cleaning options depending on configured
partition cleaning policy.
The library supports the following three partition cleaning policies:

| Policy | Description |
|---|---|
| Manual Cleaning (Default) | This option indicates that the library or application will not automatically initiate cleaning. You will receive drive status and/or RAS ticket notification that cleaning is needed. |
| Application-Managed Cleaning | This option allows you to let a third party application determine when drives need to be cleaned. RAS tickets to indicate drive cleaning needs will not be generated by the library. |
| Library-Initiated Automatic Cleaning | This option allows you to have the library determine and perform drive cleaning when drives need to be cleaned. You must have cleaning slots defined and cleaning media imported to use this feature. While a cleaning operation is in progress, the Move Medium command will not occur until the cleaning operation completes. Cleaning operations vary by drive and conditions, but can take up to a few minutes to complete. Element status for the Data Transfer element being cleaned will not reflect the presence of the cleaning cartridge. It will continue to report that it is empty and accessible. |


## Removed Drives

Depending on how the library is configured, occasionally Data Transfer elements will be reported where
no drive is physically present at the time. This could be due to a drive that has been removed for service,
or simply a placeholder for the addition of a future drive. These empty "drive bays" will be counted and
reported via Mode Sense and Read Element Status commands. These elements could appear in
between Data Transfer elements that are present, creating "gaps" among the physical drives. This should
not be considered an error. Thus, removed drives will be reported in READ ELEMENT STATUS data with
an ACCESS bit set to zero and an EXCEPT bit set to one with a vendor specific ASC/ASCQ of 83/04h,
DATA TRANSFER ELEMENT NOT INSTALLED, or ASC/ASCQ 3B/1Ah, DATA TRANSFER DEVICE
REMOVED.
