# 3. Introduction

## 3.1 Drive Overview

The products that are discussed in this book are high-performance, high-capacity data-storage devices that
connect to and provide additional storage for supported servers. They include the LTO5 through LTO9 models of
the IBM LTO Ultrium Tape Drive.
All products use the Small Computer Systems Interface (SCSI) Architecture Model. The transports used are
shown in table 3.
Figure 1 shows the IBM 3580 Ultrium Tape Drive and the IBM System Storage LTO Ultrium Tape Drive Model
T200. The IBM System Storage TS2350 Tape Drive Express is similar to the IBM System Storage LTO Ultrium
Tape Drive Model T200 and the IBM System Storage TS2250 Tape Drive Express is half the height.

<!-- Figure 1: IBM System Storage Ultrium Tape Drive Models. -->

Designed to perform unattended backups as well as to retrieve and archive files, the Ultrium Tape Drives include
the features that are described in table 3.

**Table 3 -- Features of the IBM Ultrium Tape Drives and the IBM 3580 Ultrium Tape Drive**

| Feature | Ultrium 5 | Ultrium 6 | Ultrium 7 | Ultrium 8 ^h | Ultrium 9 ^h |
|---|---|---|---|---|---|
| Native storage capacity | 1500 GB | 2500 GB | 6000 GB | 12000 GB | 18 000 GB |
| Storage capacity when compression is enabled ^a | 3.0 TB | 6250 GB | 15000 GB | 30000 GB | 45 000 GB |
| Native sustained data transfer rate | 140 MB/s | 160 MB/s | 300 MB/s | 360 MB/s ^d | 400 MB/s ^d |
| Data transfer rate when compression is enabled ^a | 280 MB/s | 400 MB/s | 700 MB/s ^e / 500 MB/s ^f | 700 MB/s ^e / 500 MB/s ^f | 700 MB/s ^e / 900 MB/s ^g |
| Burst data transfer rate (1GFC) | 100 MB/s | 100 MB/s | 100 MB/s | 100 MB/s | 100 MB/s |
| Burst data transfer rate (2GFC) | 200 MB/S | 200 MB/S | 200 MB/S | 200 MB/S | 200 MB/S |
| Burst data transfer rate (4GFC) | 400 MB/s | 400 MB/s | 400 MB/s | 400 MB/s | 400 MB/s |
| Burst data transfer rate (8GFC) | 800 MB/s | 800 MB/s | 800 MB/s | 800 MB/s | 800 MB/s |
| Burst data transfer rate (3G SAS) | 300 MB/s | 300 MB/s | 300 MB/s | 300 MB/s | 300 MB/s |
| Burst data transfer rate (6G SAS) | 600 MB/s | 600 MB/s | 600 MB/s | 600 MB/s | 600 MB/s |
| Burst data transfer rate (12G SAS) | | | Not Supported | | 1 200 MB/s |
| Type of interface | LC-D ^b / SAS ^c | LC-D ^b / SAS ^c | LC-D ^b / SAS ^c | LC-D ^b / SAS ^c | LC-D ^b / SAS ^c |

> **Note** — All sustained data rates are dependent on the capabilities of the interconnect (for example, a 8GFC link is limited to less than 800MB/sec). All information assumes same generation media and drive.

- ^a Generation 5 nominal compression ratio is 2:1. Subsequent generations nominal compression ratio is 2.5:1. Depending on the data, the compression ratio may be higher or lower.
- ^b LC-D: LC-Duplex Fibre Channel, with the use of SCSI protocol
- ^c SAS: Serial-Attached SCSI
- ^d LTO8 and LTO9 Half-High drives have a native sustained data transfer rate of 300 MB/s
- ^e When using an 8GFC interface
- ^f When using a 6 Gbps SAS interface
- ^g When using a 12Gbps SAS interface
- ^h See 3.4--Supported Tape Cartridges

## 3.2 Supported Servers and Operating Systems

The Ultrium Tape Drives are supported by a wide variety of servers and operating systems, as well as adapters.
These attachments can change throughout the products' life cycles. To determine the latest supported
attachments, visit the web at https://www-03.ibm.com/systems/support/storage/ssic/interoperability.wss.

### 3.2.1 Primary Interface Attachment

The Ultrium Tape Drives attach to servers and operating systems shown in table 4. An attachment includes (but
is not limited to) the servers and operating systems in the table.
For specific instructions about attachment, see one or more of the following:
- a) IBM System Storage TS2350 Tape Drive Setup, Operator, and Service Guide, GC27-2277-00.

**Table 4 -- Supported Servers and Operating Systems for Primary Interface Attachment**

| Supported Servers | Supported Operating Systems |
|---|---|
| zSeries s390x platform | zLinux (RHEL and SLES) |
| IBM Power Systems | IBM i |
| IBM Power Systems | AIX |
| IBM Power Systems | Linux (RHEL and SLES) |
| Sun Microsystems | Solaris |
| 32-bit, Intel-compatible servers | Windows Server / Linux (RHEL and SLES) |
| 64-bit, Intel-compatible servers | Windows Server / Linux (RHEL and SLES) |
| Supported SAN Components for Fibre Channel Attachment | Visit the web at: http://www-03.ibm.com/systems/support/storage/ssic/interoperability.wss |

## 3.3 Supported Device Drivers

IBM maintains the latest levels of device drivers and driver documentation for the IBM Ultrium Tape Drives on the
Internet. You can access this material from your browser or through the IBM FTP site by performing one of the
following procedures. (Note: If you do not have Internet access and you need information about device
drivers, contact your Marketing Representative.)
Using a browser, go to one of the following websites:
- a) The IBM storage website at http://www.ibm.com/storage; or
- b) The IBM Fix Central website at http://www.ibm.com/support/fixcentral. This is a portal to enter the download area. There are a few pull down menus to get you to the correct download as follows:
  1. in menu labeled "Product Group" select "Storage Systems";
  2. in menu labeled "Product Family" select "Tape Systems";
  3. in menu labeled "Product Type" select "Tape Device Drivers and Software";
  4. in menu labeled "Product" select "Tape Device Drivers";
  5. in menu labeled "Platform" select the correct operating system. You can select the generic form of the platform (e.g., Linux) and all device drivers for that platform will come up;
  6. click continue; and
  7. select the checkbox(es) of the fix pack(s) needed and click continue.

## 3.4 Supported Tape Cartridges

The IBM LTO Ultrium Tape Drives support LTO Cartridges as described in table 5.

**Table 5 -- LTO capacities by density, cartridges, and products**

| DENSITY NAME / Colloquial Term | Cart ^a | LTO5 | LTO6 | LTO7 | LTO8 | LTO9 |
|---|---|---|---|---|---|---|
| U-316 / LTO-3 Format | L3/LT | 400 GB RO | - | - | - | - |
| U-416 / LTO-4 Format | L4/LU | 800 GB | 800 GB RO | - | - | - |
| U-516 / LTO-5 Format | L5/LV | 1 500 GB | 1 500 GB | 1 500 GB RO | - | - |
| U-616 / LTO-6 Format | L6/LW | - | 2 500 GB | 2 500 GB | - | - |
| U-732 / LTO-7 Format | L7/LX | - | - | 6 000 GB | 6 000 GB | - |
| U-832M / LTO-8 Type M Format | M8 ^b | - | - | - | 9 000 GB / 8 400 GB ^b | - |
| U-832 / LTO-8 Format | L8/LY ^b | - | - | - | 12 000 GB / 11 600 GB ^b | 12 000 GB / 11 600 GB ^b |
| U-932 / LTO-9 Format | L9/LZ ^b | - | - | - | - | 18 000 GB / 17 400 GB ^b |

Key:
- `-` Not Supported. Values in GB (10^9) native capacity (no compression)
- RO -- Read-Only
- L3 Generation 3 DATA, LT Generation 3 WORM
- L4 Generation 4 DATA, LU Generation 4 WORM
- L5 Generation 5 DATA, LV Generation 5 WORM
- L6 Generation 6 DATA, LW Generation 6 WORM
- L7 Generation 7 DATA, LX Generation 7 WORM
- M8 Generation 8 Type M DATA
- L8 Generation 8 DATA, LY Generation 8 WORM
- L9 Generation 9 DATA, LZ Generation 9 WORM
- ^a Cart -- Cartridge
- ^b System designs should allow for capacity variations using the Application design capacity {LP17h:0018h}, see 6.4.14--LP 17h: Volume Statistics.

The Ultrium 9 Tape Drive (Generation 9) uses the IBM TotalStorage 18 000 GB Data Cartridge, and is compatible
with the cartridges of its predecessor (called Generation 8). The Ultrium 9 Tape Drive performs the following
functions:
- a) Reads and writes Generation 9 cartridges to Generation 9 format
- b) Reads and writes Generation 8 cartridges to Generation 8 format
- c) Does not write Generation 9 cartridges to other generations' format
- d) Does not write Generation 8 cartridges to other generations' format
- e) Does not write or read Generation 1 through Generation 7 cartridges in any format. This includes the M8 format.

The Ultrium 8 Tape Drive (Generation 8) uses the IBM TotalStorage 12000 GB Data Cartridge, and is compatible
with the cartridges of its predecessor (called Generation 7). The Ultrium 8 Tape Drive performs the following
functions:
- a) Reads and writes Generation 8 cartridges to Generation 8 format
- b) Changes eligible Generation 7 Type A (i.e., L7) cartridges to Generation 8 Type M (i.e., M8) cartridges
- c) Reads and writes M8 cartridges to M8 format
- d) Reads and writes Generation 7 Type A cartridges to Generation 7 format
- e) Does not write Generation 8 Type A cartridges to other generations' format
- f) Does not write Generation 8 Type M cartridges to other generations' format
- g) Does not write Generation 7 Type A cartridges to other generations' format
- h) Does not write or read Generation 1 through Generation 6 cartridges in any format

The Ultrium 7 Tape Drive (Generation 7) uses the IBM TotalStorage 6000 GB Data Cartridge, and is compatible
with the cartridges of its predecessors (called Generation 5 and Generation 6). The Ultrium 7 Tape Drive
performs the following functions:
- a) Reads and writes Generation 7 cartridges to Generation 7 format
- b) Reads and writes Generation 6 cartridges to Generation 6 format
- c) Reads Generation 5 cartridges in Generation 5 format
- d) Does not write cartridges to other generations' format
- e) Does not write Generation 5 cartridges
- f) Does not write or read Generation 1, Generation 2, Generation 3, or Generation 4 cartridges in any format

The Ultrium 6 Tape Drive (Generation 6) uses the IBM TotalStorage 2500 GB Data Cartridge, and is compatible
with the cartridges of its predecessors (called Generation 4 and Generation 5). The Ultrium 6 Tape Drive
performs the following functions:
- a) Reads and writes Generation 6 cartridges to Generation 6 format
- b) Reads and writes Generation 5 cartridges to Generation 5 format
- c) Reads Generation 4 cartridges in Generation 4 format
- d) Does not write cartridges to other generations' format
- e) Does not write Generation 4 cartridges
- f) Does not write or read Generation 1, Generation 2, or Generation 3 cartridges in any format

The Ultrium 5 Tape Drive (Generation 5) uses the IBM TotalStorage 1500 GB Data Cartridge, and is compatible
with the cartridges of its predecessors (called Generation 3 and Generation 4). The Ultrium 5 Tape Drive
performs the following functions:
- a) Reads and writes Generation 5 cartridges to Generation 5 format
- b) Reads and writes Generation 4 cartridges to Generation 4 format
- c) Reads Generation 3 cartridges in Generation 3 format
- d) Does not write cartridges to other generations' format
- e) Does not write Generation 3 cartridges
- f) Does not write or read Generation 1 cartridges or Generation 2 cartridges in any format

## 3.5 Microcode Detection of Errors

The drive microcode is designed to check for logic errors, to handle hardware-detected errors, and to detect and
report microcode-related errors.

### 3.5.1 Fencing Behavior

For a description of the Fencing Behavior and Persistent Error handling, see 4.19.5--Persistent Errors.
