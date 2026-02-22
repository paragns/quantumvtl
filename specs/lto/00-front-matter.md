# IBM TotalStorage LTO Ultrium Tape Drive

# SCSI Reference

> **Note** — Before using this manual and the product it supports, read the information under Annex F. Notices.

Seventh Edition (30 April 2024)
This edition applies to the IBM System Storage LTO Ultrium Tape Drive SCSI Reference and to all subsequent
releases and modifications unless otherwise indicated in new editions.
Copyright International Business Machines Corporation 2011, 2017, 2020, 2021, 2022, 2023, 2024.
US Government Users Restricted Rights -- Use, duplication or disclosure restricted by GSA ADP Schedule
Contract with IBM Corp.

# Read This First

This is the Seventh Edition of the IBM System Storage LTO Tape Drive SCSI Reference.

## 0.1 Summary of Changes

### 0.1.1 First Edition, April 2011

The IBM System Storage LTO Ultrium Tape Drive SCSI Reference describes the SCSI interface for the IBM LTO
1 tape drive through the IBM LTO 4 tape drive. The first edition of the IBM System Storage LTO Tape Drive SCSI
Reference describes the SCSI interface for the IBM LTO 5 tape drive and does not describe the previous
generation tape drives that are described in the IBM System Storage LTO Ultrium Tape Drive SCSI Reference.
The list of Functional Change Requests (FCR) applied to the previous generation device (i.e., LTO4) that are
included follow:
- FCR 3163r3 - IP Address Information Configuration;
- FCR 3164 - LTO Engineering Log (Buffer ID 06h);
- FCR 3165 - LTO5 - TapeAlert 10h behavior;
- FCR 3167 - Persistent Reserve Out SCOPE field;
- FCR 3173 - Device Attributes Mode Pages;
- FCR 3174r2 - LTO5 - Sleep Mode (Mode Page 1Ah);
- FCR 3175r3 - LTO5 - Partitioning SCSI changes;
- FCR 3176r1 - Encryption Selection mode page;
- FCR 3177 - LTO5 - SCSI Identifier updates;
- FCR 3178r2 - LTO5 - SkipSync;
- FCR 3179r2 - LTO5 - Append-only mode (data-safe);
- FCR 3180r2 - LTO5 - Transport Log & Mode pages;
- FCR 3181 - LTO5 Report Supported OpCode;
- FCR 3183r1 - LTO5 Programmable Early Warning;
- FCR 3184 - LTO5 Volume Statistics log page (17h);
- FCR 3185 - LTO5 Device Statistics log page (14h);
- FCR 3186 - LTO5 Data Compression log page (1Bh);
- FCR 3187 - SPIN & SPOUT (OOBE-KMIP-SSC-4);
- FCR 3188r1 - LTO5 Engineering & Speed log pages;
- FCR 3193 - End of partition behavior control;
- FCR 3194 - SAS TLR count in log pages;
- FCR 3197 - Update standard inquiry version field;
- FCR 3202 - CM from EOD dataset Read Buffer;
- FCR 3205 - Drive Type in Inquiry C0h;
- FCR 3208 - Logical block protection;
- FCR 3212 - LOAD ID for LTO HH V2 drives;

### 0.1.2 Second Edition, February 2013

The second edition of the IBM System Storage LTO Tape Drive SCSI Reference describes the SCSI interface for
the IBM LTO 5 and LTO 6 tape drives.

The list of defects and changes applied to the first edition of the IBM System Storage LTO Tape Drive SCSI
Reference to create this edition follow:
- FCR 3178r5 - Update SkipSync for LTO6. See MP 30h[40h]: SkipSync - Device attribute settings (see 6.6.21.5.1 on page 415)
- FCR 3182r2 - Dynamic Runtime Information clean-up. See READ DYNAMIC RUNTIME ATTRIBUTE - A3h[1Eh] or D1h (see 5.2.19 on page 114), WRITE DYNAMIC RUNTIME ATTRIBUTE - A4h[1Eh] or D2h (see 5.2.45 on page 182), and Dynamic runtime attributes (DRA) (see 6.2 on page 219)
- FCR 3206 - Describe Units of measure used in this document
- FCR 3227 - Make OIR saveable in MP 10h: Device Configuration (see 6.6.11 on page 379)
- FCR 3229 - Deferred Check Condition (DCC) (see 4.19.3 on page 65)
- FCR 3233, FCR 3233r1, FCR 3233r2 - LTO6 SCSI Identifier updates
- FCR 3235 - Add IP B1h: Manufacturer-assigned Serial Number (see 6.3.10 on page 244)
- FCR 3237, FCR 3237r1 - Add READ BLOCK LIMITS maximum logical object identifier data (see 5.2.17.2 on page 113)
- FCR 3240 - Add Remaining Native Capacity to LP 17h: Volume Statistics (see 6.4.14 on page 301)
- FCR 3241 - BOP caching (see 4.5.2 on page 34)
- FCR 3242 - Add LTO6 Encryption Algorithm to SPIN (20h[0010h]) - Data Encryption Capabilities page (see 6.8.2.3 on page 450)
- FCR 3244 - Add create FMR tape and update drive From FMR tape to Supported Page 80h Diags (see 6.1.2 on page 198)
- FCR 3246 - Add OEM Specific Inquiry field
- FCR 3248 - Add LTO6 Timeout values to the Command timeouts descriptor (see 5.2.28.3 on page 147) of the REPORT SUPPORTED OPERATION CODES command
- FCR 3249 - Ignore PS bit on MODE SELECT
- FCR 3250 - Add LOCATE to EOD
- FCR 3251 - Encryption Sense Key changes (see Annex B.)
- FCR 3253 - Partition mode page partition size table mods
- FCR 3256 - Add standardized method for reading drive dumps
- FCR 3257 - Update LP31h for 4 partitions
- FCR 3258 - Tape Diagnostic Data - correct PARAMETER CODE field
- Various corrections of editorial and functional documentation issues.
- 31999 - LTO SCSI Ref: ASC/ASCQ EE31 description should be "Key Unknown" (see Annex B.)

### 0.1.3 Third Edition, 28 September 2015

The third edition of the IBM System Storage LTO Tape Drive SCSI Reference describes the SCSI interface for
the IBM LTO tape drives from generation 5 and later. The primary purpose of this edition is to document the new
LTO 7 drive.
The list of significant defects and changes applied to the second edition of the IBM System Storage LTO Tape
Drive SCSI Reference to create this edition follow:
- FCR 3260 - Cache Attributes for READ ATTRIBUTES
- FCR 3268 - Add Potential Conflict List and Extended VHF data to LP 11h: DT Device Status (see 6.4.10 on page 268)
- FCR 3271 - Download ucode additions
- FCR 3273 - Add part number to standard inquiry
- FCR 3282 - LTO7 SCSI Identifiers
- FCR 3284 - READ LOGGED-IN HOST TABLE - A3h[1Fh][01h] (see 5.2.21 on page 122)
- FCR 3285 - Add Encr Policy - Rqst Parms every reposition to Device Hardware Encryption (see 4.15 on page 55)
- FCR 3286 - Add Read Buffer MODE [1Ch] 11h: Mini dump (see 6.7.2.8.3.3 on page 440) and Diag - 0163h: Force Mini Dump (see 6.1.12 on page 208)
- FCR 3287 - Extend OEM field of Standard Inquiry
- FCR 3289 - Automation Device S/N VPD Page
- FCR 3292 - Additional params in LP3Eh
- FCR 3293 - READ END OF WRAP POSITION
- FCR 3297 - Unique Cartridge Identity (MAM 1001h)
- FCR 3299 - Add TA 31h Diminshed Native Capacity to Parameter Definitions (2Eh) (see 6.4.18.2 on page 317)
- FCR 3302 - Correct MP 1Ch: Informational Exceptions Control (see 6.6.17 on page 394)
- FCR 3308 - READ BUFFER non-volatile host buffer
- FCR 3309 - Add option to Disable BOP caching (see 4.5.2 on page 34)
- FCR 3310 - Describe Mode Page Behaviors (see 4.7 on page 40) that are non-standard
- FCR 3311 - Inquiry Allocation Length
- FCR 3314 - LTFS MAM parms 0820h & 0821h
- FCR 3316 - LTO7 increase counter sizes
- FCR 3317 - LTO7 LBP add support for CRC32C
- Various corrections of editorial and functional documentation issues.
- Added SPIN (00h[0002h]) - Security Compliance Information (see 6.8.1.3 on page 446) and SPIN (20h[0031h]) - Device Server Key Wrapping Public Key page (see 6.8.2.9 on page 466) during review

### 0.1.4 Fourth Edition, GA32-0928-03, 16 October 2017

The fourth edition of the IBM System Storage LTO Tape Drive SCSI Reference describes the SCSI interface for
the IBM LTO tape drives from generation 5 and later. The primary purpose of this edition is to document the new
LTO 8 drive.
The list of significant defects and changes applied to the third edition of the IBM System Storage LTO Tape Drive
SCSI Reference to create this edition follow:
- Various corrections of editorial and functional documentation issues.
- f3255, f3255r1 - Correct LOAD UNLOAD command
- f3264 - Terminate Immediate
- f3266 - Correct MultiP bit in standard inquiry
- f3321 - LBP Support Inquiry B5h
- f3327 - LP11 Parm 8001 Medium Encryption Status
- f3331 - Encrypt only
- f3332 - LTO8+ Volume Personality Scheme
- f3333 - Update DRA to support standards
- f3334 - Update SCSI Reference with LP12h
- f3330, f3330r2 - LTO8 SCSI Identifiers
- f3325 - LTO8 Type M SCSI Interface
- f3336 - Additional M8 MAM Changes

### 0.1.5 Fifth Edition, GA32-0928-04, 15 July 2021

The fifth edition of the IBM System Storage LTO Tape Drive SCSI Reference describes the SCSI interface for the
IBM LTO tape drives from generation 5 and later. The primary purpose of this edition is to document the new LTO
9 drive.

The list of significant defects and changes applied to the fourth edition of the IBM System Storage LTO Tape
Drive SCSI Reference to create this edition follow:
- Various corrections of editorial and functional documentation issues.
- 35236 - TA 31h clear on load/unload instead of removal (see 6.4.18)
- 37348 - 4/0302 missing sk/ascq from sense Annex (see B.5.)
- 37479 - PAMR note about library incorrect (see 5.2.14)
- 37537 - DOC:0/0019 missing(locate immediate in progress) (see B.1.)
- 37553 - Correct log pages for missing SAS 12G values (see 6.4.10) and change section style to be consistent with the style used in the SCSI Reference (see 6.4.15).
- 37623 - POST B diagnostic description should list FSC 52E7 (see 6.1.6)
- 37691 - Externalize the mode page to set the preferred cartridge type (see 6.6.21.5.4)
- FCR 3283r2 - Environmental Conditions Log Page 0Dh[01h] - Temperature and Humidity (see 6.4.9)
- FCR 3336r1 - Supported Density Codes Attribute ID change (see 6.5.2.3)
- FCR 3337 - Archive Mode setting MP30[43] (see 6.6.21.5.3)
- FCR 3339 - Long Erase Timeout adjustments (see 5.2.28.3)
- FCR 3344 - LTO SCSI Ref RW Buffer Corrections (see 5.2.18, 5.2.44, and 6.7)
- FCR 3346 - Report Optical Transceiver Information (see 6.4.10.1.9)
- FCR 3347 - SCSI Reference Log Page Corrections (see 6.4.3)
- FCR 3350 - Autoload Mode SCSI Configuration (see 6.6.7)
- FCR 3351r1 - IP C1h Drive Serial Numbers (see 6.3.15)
- FCR 3356 - Add barcode inquiry page IP C2h (see 6.3.16)
- FCR 3360 - Tape Cartridge Type LP11h[8001h] (see 6.4.10.1.11)
- FCR 3366 - LTO9 SCSI Identifiers
- FCR 3367 - RAO-Open (LTO9) (see 4.6)
- FCR 3369 - IDLE_C Saveable (see 6.6.16)
- FCR 3370 - Subcomponent Version List (see 6.3.17)
- FCR 3371 - Design Capacity (Section removed by 38609)
- FCR 3372 - Log Supages (see 6.4)
- FCR 3373 - TA23h - Humidity Tape Alert (see 6.4.18)
- FCR 3376 - Document cmds that reset idle_c timer (see 5.1.2)
- FCR 3377 - LTO9 Device Stats medium descriptor (see 6.4.12.3.2.1.1)
- FCR 3378 - Describe Engineering Log entry formats (see 6.7.2.1 and 6.7.2.2)
- FCR 3381 - Partitioning adjustments (see 4.4.2, 6.6.13, and Annex A.)
- FCR 3383 - Medium Characterization + format (see 4.1, 5.2.3, and E.3.)
- FCR 3384 - LTO Cart Motion Meters MAM Attribute (see 6.5.2.5.6)
- FCR 3385 - Update RSOC for TDScal (see 5.2.28.3)

### 0.1.6 Sixth Edition, GA32-0928-05, 22 February 2022

The sixth edition of the IBM System Storage LTO Tape Drive SCSI Reference describes the SCSI interface for
the IBM LTO tape drives from generation 5 and later. The primary purpose of this edition is to document the
additions related to the LTO 9 Half-Height drive.

The list of significant defects and changes applied to the fifth edition of the IBM System Storage LTO Tape Drive
SCSI Reference to create this edition follow:
- Various corrections of editorial and functional documentation issues.
- Externalized Medium Calibration Audit (see 6.1.9)
- Added Best Practices to the organization list in the Preface (see 1.1)
- FCR 3382 - MAM Attribs for Medium Characterization (see 6.5.2.5, 4.1, and E.3.) made External
- FCR 3391 - IDLE_C optional 12V disable (see 6.6.16 and 6.6.21.5.3)
- FCR 3392 - LTO9 Cleaning Criteria (see 4.13.3)
- FCR 3393 - ERASE IN PROGRESS ASC/Q (see B.1.)
- FCR 3395 - Clarify Error Counter LPs descriptions (see 6.4.5, 6.4.6, 6.4.22, and 6.4.23)

### 0.1.7 Seventh Edition, GA32-0928-06, 17 October 2023

The Seventh edition of the IBM System Storage LTO Tape Drive SCSI Reference describes the SCSI interface
for the IBM LTO tape drives from generation 5 and later. The primary purpose of this edition is to provide updates
made since the previous revision.
The list of significant defects and changes applied to the sixth edition of the IBM System Storage LTO Tape Drive
Reference to create this edition follow:
- 37851 - Clarify SkipSync validity fields.
- 37924 - Clarify that Media Optimization only occurs a maximum of once per mount (see 4.1, 5.2.3, and 6.5.2.5.4).
  - A) 38101- Corrected M8 SET CAPACITY values (see table 140).
- 38316 - Clarified the meaning of TA49--Diminished Native Capacity {LP2Eh:0031h} and prepended "TA<decimal number>" to the TA flag names for ease of use (see 6.4.18).
- f3394, f3394r1, f3394r3 - Application Design Capacities (see Application design capacity {LP17h:0018h} on page 304 and Volume Lifetime Remaining {LP17h:0019h} on page 304)
- f3398, f3398r1 - SFP Page A2h log pages (see 6.4.27 and 6.4.30)
- f3399 - User Defined Cartridge Identity (UDCI) (see 6.5.2.2.3, 6.5.2.3.8, and 6.5.2.5.3)
- f3400 - Add definition to Maximum Tape Transfer Rate {LP37h:2?A0h} on page 336
- f3403, f3403r1 - Writing Drive Identifying Information of most recently read data set {LP38h:0100h} on page 340
- f3404 - Change default FC EPDC in MP[18h] to disabled (see 6.6.14.1)
- f3409 - Report Task Management mods (see 5.2.29)
- f3413r1 - LP14h Medium Optimization Version in use (see Firmware Medium Optimization Version {LP14h:F001h} (see page 293) and 6.5.2.5.5)
- f3414 - DRA 0014h - Last failed reservation information (see 6.2.2.3.7)
- 38609 - GA32-0928-06 draft review
  - a) reconcile paragraph and font styles for Book;
  - b) removed Design Capacity section due to modifying table 5 to include the application design capacity;
  - c) modified the {nnh:nnnnh} to be {LPnnh:nnnnh};
  - d) appended "{MAM nnnnh}" to MAM headings and "{DRA nnnnh}" to Dynamic Runtime Attributes headings and added them to the Index Of Statistics and Attributes on page 509;
  - e) editorial changes to force items in the Index of Statistics and Attributes that are too long for a single line to wrap lines at convenient places;
  - f) Reformat Table of Contents;
  - g) add additional sense codes and additional descriptions to sense codes;
  - h) make cross-reference formats consistent
  - i) Do thorough consistency check for all Byte*/Bit* list paragraph styles with same level, with parent level, and with child level.
  - j) Incorporate Final Review feedback.


## Contents

| Section | Page |
|---------|------|
| 0.1 Summary of Changes | a |
| 0.1.1 First Edition, April 2011 | a |
| 0.1.2 Second Edition, February 2013 | a |
| 0.1.3 Third Edition, 28 September 2015 | b |
| 0.1.4 Fourth Edition, GA32-0928-03, 16 October 2017 | c |
| 0.1.5 Fifth Edition, GA32-0928-04, 15 July 2021 | c |
| 0.1.6 Sixth Edition, GA32-0928-05, 22 February 2022 | d |
| 0.1.7 Seventh Edition, GA32-0928-06, 17 October 2023 | e |
| 1. Preface | 1 |
| 1.1 Organization | 1 |
| 1.2 Related Publications | 1 |
| 2. Definitions, symbols, abbreviations, and conventions | 3 |
| 2.1 Definitions | 3 |
| 2.2 Conventions | 16 |
| 2.2.1 Radix representation | 16 |
| 2.2.2 Bit Numbering | 16 |
| 2.2.3 Units of measure for data storage | 16 |
| 2.2.4 Subpages | 17 |
| 2.2.5 Hyperlinks | 17 |
| 2.3 Tape Drive Model Names | 17 |
| 3. Introduction | 19 |
| 3.1 Drive Overview | 19 |
| 3.2 Supported Servers and Operating Systems | 20 |
| 3.2.1 Primary Interface Attachment | 20 |
| 3.3 Supported Device Drivers | 21 |
| 3.4 Supported Tape Cartridges | 22 |
| 3.5 Microcode Detection of Errors | 23 |
| 3.5.1 Fencing Behavior | 23 |
| 4. Implementation Considerations | 25 |
| 4.1 Media Optimization (LTO9) | 25 |
| 4.2 Write modes | 26 |
| 4.2.1 Write mode introduction | 26 |
| 4.2.2 Overwrite-allowed mode | 26 |
| 4.2.3 Append-only mode (also known as Data-safe mode) | 26 |
| 4.3 Archive mode unthread (LTO7+) | 29 |
| 4.4 Volume partitioning | 29 |
| 4.4.1 Volume partitioning overview | 29 |
| 4.4.2 Wrap-wise Partitioning | 29 |
| 4.4.3 Partitioning and capacity scaling | 33 |
| 4.4.4 Partitioning and media types | 33 |
| 4.4.5 Partitioning and reformatting | 33 |
| 4.4.6 Partitioning and encryption | 34 |
| 4.5 Object buffer | 34 |
| 4.5.1 Object buffer introduction | 34 |
| 4.5.2 BOP caching | 34 |
| 4.5.2.1 BOP caching side effects | 35 |
| 4.6 Recommended access order (RAO) (LTO-9+ Full-Height) | 35 |
| 4.6.1 RAO Suitability | 35 |
| 4.6.2 RAO overview | 35 |
| 4.6.2.1 User data segments (UDS) in a partition | 35 |
| 4.6.2.2 User data segment descriptors | 35 |
| 4.6.3 RAO features | 36 |
| 4.6.3.1 RAO features overview | 36 |
| 4.6.3.2 Determining the UDS limits | 36 |
| 4.6.3.3 Specifying the process for generating the RAO list | 36 |
| 4.6.3.4 Specifying binding points in the RAO list | 36 |
| 4.6.4 RAO usage | 37 |
| 4.6.4.1 User data segment geometry usage | 39 |
| 4.7 Mode Page Behaviors | 40 |
| 4.7.1 Mode Page Policy -- non-standard | 40 |
| 4.7.1.1 Mode parameter header and block descriptor policy | 40 |
| 4.7.1.2 Mode page policy | 40 |
| 4.7.2 Classification of mode parameters | 41 |
| 4.7.2.1 Save behavior -- non-standard | 42 |
| 4.7.2.2 Parameter Saveable behavior -- non-standard | 42 |
| 4.7.3 Mode parameters and unit attentions | 42 |
| 4.8 Programmable early warning | 43 |
| 4.9 Logical block protection | 43 |
| 4.9.1 Logical block protection overview | 43 |
| 4.9.1.1 Logical block protection | 44 |
| 4.9.2 Protection information on a volume | 44 |
| 4.9.3 Logical blocks and protection information | 46 |
| 4.9.4 Protecting logical blocks transferred during writes | 46 |
| 4.9.5 Protecting logical blocks processed during reads and verifies | 47 |
| 4.10 Multiple Port Behavior | 47 |
| 4.11 Data Transfer, Block Limits, and Fixed Block Option | 48 |
| 4.12 Request Sense Information, ILI, and Command Interactions | 48 |
| 4.12.1 General Read-Type Handling | 48 |
| 4.12.2 Interactions Summary | 50 |
| 4.13 Drive Cleaning | 52 |
| 4.13.1 Cleaning the Drive in a Library | 52 |
| 4.13.2 Drive Cleaning Indicators | 52 |
| 4.13.2.1 Panel Cleaning Indication | 53 |
| 4.13.2.2 Host Interface - Dynamic Cleaning Indicators | 53 |
| 4.13.2.3 Host Interface - Static Cleaning Indicator (Sense Data Byte 70) | 53 |
| 4.13.3 Cleaning Criteria | 53 |
| 4.14 WORM Behaviors | 54 |
| 4.14.1 Conditions for Writing | 54 |
| 4.14.2 Command Behavior When WORM Medium Has Been Tampered With | 54 |
| 4.15 Device Hardware Encryption | 55 |
| 4.15.1 Encryption Control - IBM Proprietary Protocol (IPP) | 55 |
| 4.15.2 Encryption Control - T10 Standards | 56 |
| 4.15.2.1 External Data Encryption Control | 56 |
| 4.16 Attachment Features | 56 |
| 4.16.1 Types of Interface Attachments | 56 |
| 4.16.2 Common Tape LUN Behaviors | 56 |
| 4.16.2.1 Power-On | 56 |
| 4.16.2.2 Reset Strategy | 57 |
| 4.16.2.3 Abort Handling | 57 |
| 4.16.2.4 Multi-initiator Support | 58 |
| 4.16.2.5 Status Codes | 59 |
| 4.16.3 Features of the Fibre Channel Interface | 59 |
| 4.16.3.1 Topology | 60 |
| 4.16.3.1.1 Two-Node Switched Fabric Topology | 60 |
| 4.16.3.1.2 Two-Node Direct Connection Topology | 61 |
| 4.16.3.2 Speed | 61 |
| 4.16.3.3 Addressing Assignments | 61 |
| 4.16.4 Features of the Serial Attached SCSI (SAS) Interface | 61 |
| 4.17 Device Clocks | 61 |
| 4.18 Dynamic runtime information | 62 |
| 4.18.1 Dynamic runtime information overview | 62 |
| 4.18.2 Dynamic runtime information timestamp | 63 |
| 4.18.3 Setting dynamic runtime information into the drive | 63 |
| 4.18.4 Retrieving dynamic runtime information from the drive | 64 |
| 4.18.5 Management of dynamic runtime information | 64 |
| 4.18.5.1 Dynamic Runtime Information Lifetime | 64 |
| 4.19 Error Information | 65 |
| 4.19.1 Sense Data | 65 |
| 4.19.2 Sense Data Management | 65 |
| 4.19.3 Deferred Check Condition (DCC) | 65 |
| 4.19.4 Unit Attention Conditions | 66 |
| 4.19.5 Persistent Errors | 66 |
| 4.19.5.1 Fencing Behavior | 66 |
| 4.19.5.1.1 ALLOW_NO_OPERATION | 67 |
| 4.19.5.1.2 ALLOW_LOCATE | 67 |
| 4.19.5.1.3 ALLOW_UNLOAD | 67 |
| 4.19.5.1.4 MID-TAPE RECOVERY | 67 |
| 4.19.5.1.4.1 Normal operation (i.e., MTR Fence) | 67 |
| 4.19.5.1.4.2 Panic Fence operation | 68 |
| 4.20 Medium auxiliary memory | 68 |
| 4.21 Volume Coherency | 69 |
| 4.22 Error history (i.e., drive dump) | 70 |
| 4.22.1 Error history overview | 70 |
| 4.22.2 Retrieving error history with the READ BUFFER command | 70 |
| 4.23 Potential conflict list (LTO6 and later) | 72 |
| 4.24 Environmental Conditions Thresholding (LTO9 and later) | 73 |
| 5. SCSI Commands | 75 |
| 5.1 SCSI Commands Overview | 75 |
| 5.1.1 Unsupported SCSI Commands | 75 |
| 5.1.2 Supported SCSI Commands | 75 |
| 5.1.2.1 Supported SCSI Commands on LUN 1 | 75 |
| 5.1.2.2 Supported SCSI Commands on LUN 0 | 75 |
| 5.1.2.3 Control Byte Definition | 78 |
| 5.2 SCSI Commands Listing | 79 |
| 5.2.1 ALLOW OVERWRITE - 82h | 79 |
| 5.2.2 ERASE - 19h | 80 |
| 5.2.3 FORMAT MEDIUM - 04h | 80 |
| 5.2.4 GENERATE RECOMMENDED ACCESS ORDER (GRAO) - A4h[1Dh] (LTO9+) | 82 |
| 5.2.4.1 GRAO Parameter Data | 83 |
| 5.2.4.1.1 GRAO User Data Segment descriptor | 84 |
| 5.2.5 INQUIRY - 12h | 84 |
| 5.2.5.1 Standard Inquiry Data | 85 |
| 5.2.6 LOAD/UNLOAD - 1Bh | 90 |
| 5.2.7 LOCATE (10/16) - 2Bh/92h | 92 |
| 5.2.8 LOG SELECT - 4Ch | 94 |
| 5.2.9 LOG SENSE - 4Dh | 94 |
| 5.2.10 MODE SELECT (6/10) - 15h/55h | 95 |
| 5.2.11 MODE SENSE (6/10) - 1Ah/5Ah | 96 |
| 5.2.12 PERSISTENT RESERVE IN (PRIN)- 5Eh | 98 |
| 5.2.12.1 READ KEYS service action | 98 |
| 5.2.12.2 READ RESERVATION service action | 100 |
| 5.2.12.3 REPORT CAPABILITIES service action | 101 |
| 5.2.12.4 READ FULL STATUS service action | 102 |
| 5.2.12.4.1 Full status descriptors | 103 |
| 5.2.13 PERSISTENT RESERVE OUT - 5Fh | 104 |
| 5.2.13.1 Basic PERSISTENT RESERVE OUT parameter list | 105 |
| 5.2.13.2 PERSISTENT RESERVE OUT with REGISTER AND MOVE service action parameters | 106 |
| 5.2.14 PREVENT ALLOW MEDIUM REMOVAL - 1Eh | 106 |
| 5.2.14.1 Medium removal | 107 |
| 5.2.15 READ - 08h | 107 |
| 5.2.16 READ ATTRIBUTE - 8Ch | 108 |
| 5.2.16.1 ATTRIBUTE VALUES service action | 109 |
| 5.2.16.2 ATTRIBUTE LIST service action | 110 |
| 5.2.16.3 LOGICAL VOLUME LIST service action | 110 |
| 5.2.16.4 PARTITION LIST service action | 111 |
| 5.2.16.5 SUPPORTED ATTRIBUTES service action | 111 |
| 5.2.17 READ BLOCK LIMITS - 05h | 112 |
| 5.2.17.1 READ BLOCK LIMITS block length data | 112 |
| 5.2.17.2 READ BLOCK LIMITS maximum logical object identifier data | 113 |
| 5.2.18 READ BUFFER - 3Ch | 113 |
| 5.2.19 READ DYNAMIC RUNTIME ATTRIBUTE - A3h[1Eh] or D1h | 114 |
| 5.2.19.1 READ DYNAMIC RUNTIME ATTRIBUTE Service Action | 116 |
| 5.2.19.2 SUPPORTED ATTRIBUTES service action | 116 |
| 5.2.19.3 ATTRIBUTE VALUES FOR THIS I_T NEXUS service action | 117 |
| 5.2.19.4 ATTRIBUTE VALUES FOR ALL I_T NEXUSES service action | 118 |
| 5.2.20 READ END OF WRAP POSITION - A3h[1Fh][45h] | 119 |
| 5.2.20.1 REOWP Short form parameter data | 121 |
| 5.2.20.2 REOWP Long form parameter data | 121 |
| 5.2.20.2.1 Wrap descriptor | 122 |
| 5.2.21 READ LOGGED-IN HOST TABLE - A3h[1Fh][01h] | 122 |
| 5.2.21.1 READ LOGGED-IN HOST TABLE parameter data | 123 |
| 5.2.21.1.1 Logged-In Host Descriptor | 124 |
| 5.2.22 READ POSITION - 34h | 125 |
| 5.2.22.1 READ POSITION command description | 125 |
| 5.2.22.2 READ POSITION data layout, short form | 125 |
| 5.2.22.3 READ POSITION data layout, long form | 128 |
| 5.2.22.4 READ POSITION data layout, extended form | 129 |
| 5.2.23 RECEIVE DIAGNOSTIC RESULTS - 1Ch | 131 |
| 5.2.24 RECEIVE RECOMMENDED ACCESS ORDER (RRAO) - A3h[1Dh] (LTO9+) | 131 |
| 5.2.24.1 RRAO parameter data | 132 |
| 5.2.24.1.1 UDS Limits page | 133 |
| 5.2.24.1.2 RAO list | 133 |
| 5.2.24.1.3 User Data Segment descriptor | 134 |
| 5.2.24.1.3.1 Additional information descriptor | 136 |
| 5.2.25 RELEASE UNIT (6/10)- 17h/57h | 137 |
| 5.2.26 REPORT DENSITY SUPPORT - 44h | 138 |
| 5.2.26.1 Report Density Support data layout | 138 |
| 5.2.26.1.1 Density descriptor overview | 138 |
| 5.2.26.1.2 Density information | 141 |
| 5.2.27 REPORT LUNS - A0h | 142 |
| 5.2.27.1 Report LUNs data layout | 142 |
| 5.2.28 REPORT SUPPORTED OPERATION CODES - A3h[0Ch] | 143 |
| 5.2.28.1 All_commands parameter data layout | 145 |
| 5.2.28.2 One_command parameter data layout | 146 |
| 5.2.28.3 Command timeouts descriptor | 147 |
| 5.2.28.3.1 Overview | 147 |
| 5.2.28.3.2 WRITE BUFFER command timeouts descriptor command specific field usage | 158 |
| 5.2.29 REPORT SUPPORTED TASK MANAGEMENT FUNCTIONS - A3h[0Dh] | 158 |
| 5.2.29.1 REPORT SUPPORTED TASK MANAGEMENT FUNCTIONS parameter data | 159 |
| 5.2.30 REPORT TIMESTAMP - A3h[0Fh] | 160 |
| 5.2.30.1 REPORT TIMESTAMP parameter data | 161 |
| 5.2.31 REQUEST SENSE - 03h | 161 |
| 5.2.31.1 Sense Data Layout | 162 |
| 5.2.32 RESERVE (6/10)- 16h/56h | 168 |
| 5.2.33 REWIND - 01h | 169 |
| 5.2.34 SECURITY PROTOCOL IN (SPIN) - A2h | 169 |
| 5.2.35 SECURITY PROTOCOL OUT (SPOUT) - B5h | 170 |
| 5.2.36 SEND DIAGNOSTIC - 1Dh | 171 |
| 5.2.37 SET CAPACITY - 0Bh | 172 |
| 5.2.38 SET TIMESTAMP - A4h[0Fh] | 173 |
| 5.2.38.1 SET TIMESTAMP Parameter List | 174 |
| 5.2.39 SPACE (6/16) - 11h/91h | 175 |
| 5.2.40 TEST UNIT READY - 00h | 176 |
| 5.2.41 VERIFY (6) - 13h | 176 |
| 5.2.42 WRITE - 0Ah | 178 |
| 5.2.43 WRITE ATTRIBUTE - 8Dh | 178 |
| 5.2.44 WRITE BUFFER - 3Bh | 181 |
| 5.2.45 WRITE DYNAMIC RUNTIME ATTRIBUTE - A4h[1Eh] or D2h | 182 |
| 5.2.45.1 WRITE DYNAMIC RUNTIME ATTRIBUTE parameter list | 183 |
| 5.2.46 WRITE FILEMARKS - 10h | 184 |
| 6. Parameters for SCSI Commands | 185 |
| 6.1 Diagnostic Parameters (Diag) | 187 |
| 6.1.1 Diag Page Formats | 187 |
| 6.1.1.1 Page 00h | 187 |
| 6.1.1.2 SendDiag Data - Page 00h | 187 |
| 6.1.1.3 RcvDiag Data - Page 00h | 188 |
| 6.1.1.4 Page 80h | 189 |
| 6.1.1.4.1 SendDiag - Page 80h | 189 |
| 6.1.1.5 RcvDiag - Page 80h | 190 |
| 6.1.1.5.1 RcvDiag - Page 80h Typical Results | 191 |
| 6.1.1.5.2 SIM/MIM Message | 193 |
| 6.1.1.5.2.1 SIM/MIM Header Data | 193 |
| 6.1.1.5.2.2 SIM Messages | 194 |
| 6.1.1.5.3 MIM Messages | 196 |
| 6.1.2 Supported Page 80h Diags | 198 |
| 6.1.3 Diag - SelfTest: Self Test | 199 |
| 6.1.3.1 SendDiag Command - Self Test | 199 |
| 6.1.3.2 RcvDiag Data - Self Test | 199 |
| 6.1.4 Diag - 0090h: Primary port wrap test | 199 |
| 6.1.4.1 SendDiag Parm Data - Primary port wrap test | 200 |
| 6.1.4.2 RcvDiag Data - Primary port wrap test | 200 |
| 6.1.5 Diag - 0100h: POST A | 200 |
| 6.1.5.1 SendDiag Parm Data - POST A | 201 |
| 6.1.5.2 RcvDiag Data - POST A | 201 |
| 6.1.6 Diag - 0101h: POST B Performance | 201 |
| 6.1.6.1 SendDiag Parm Data - POST B Performance | 202 |
| 6.1.6.2 RcvDiag Data - POST B Performance | 202 |
| 6.1.7 Diag - 0102h: POST C Media Test | 202 |
| 6.1.7.1 SendDiag Parm Data - POST C Media Test | 202 |
| 6.1.7.2 RcvDiag Data - POST C Media Test | 202 |
| 6.1.8 Diag - 0103h: POST D Head Test | 203 |
| 6.1.8.1 SendDiag Parm Data - POST D Head Test | 203 |
| 6.1.8.2 RcvDiag Data - POST D Head Test | 203 |
| 6.1.9 Diag -0111h: Medium Calibration Audit | 203 |
| 6.1.9.1 SendDiag Parm Data - Medium Calibration Audit | 204 |
| 6.1.9.2 RcvDiag Data - Medium Calibration Audit | 205 |
| 6.1.9.2.1 Common results descriptor | 205 |
| 6.1.10 Diag - 0160h: Force Dump | 206 |
| 6.1.10.1 SendDiag Parm Data - Force Dump | 207 |
| 6.1.10.2 RcvDiag Data - Force Dump | 207 |
| 6.1.11 Diag -0161h: Write Dump to Cartridge | 207 |
| 6.1.11.1 SendDiag Parm Data - Write Dump to Cartridge | 207 |
| 6.1.11.2 RcvDiag Data - Write Dump to Cartridge | 207 |
| 6.1.12 Diag - 0163h: Force Mini Dump | 208 |
| 6.1.12.1 SendDiag Parm Data - Force Mini Dump | 208 |
| 6.1.12.2 RcvDiag Data - Force Mini Dump | 208 |
| 6.1.13 Diag - 0170h: Create FMR Cartridge (not on LTO5) | 208 |
| 6.1.13.1 SendDiag Parm Data - Create FMR Cartridge | 209 |
| 6.1.13.2 RcvDiag Results Data - Create FMR Cartridge | 209 |
| 6.1.14 Diag - 0171h: Unmake FMR Cartridge (not on LTO5) | 209 |
| 6.1.14.1 SendDiag Parm Data - Unmake FMR Cartridge | 210 |
| 6.1.14.2 RcvDiag Results Data - Unmake FMR Cartridge | 210 |
| 6.1.15 Diag - 0175h: Use FMR Cartridge (not on LTO5) | 210 |
| 6.1.15.1 SendDiag Parm Data - Use FMR Cartridge | 211 |
| 6.1.15.2 RcvDiag Results Data - Use FMR Cartridge | 211 |
| 6.1.16 Diag - 0190h: Set Traps | 211 |
| 6.1.16.1 SendDiag Parm Data - Set Traps | 211 |
| 6.1.16.2 RcvDiag Data - Set Traps | 212 |
| 6.1.17 Diag - 0191h: Remove Traps | 212 |
| 6.1.17.1 SendDiag Parm Data - Remove Traps | 212 |
| 6.1.17.2 RcvDiag Data - Remove Traps | 213 |
| 6.1.18 Diag - 0210h: Terminate Immediate Command | 213 |
| 6.1.18.1 Send Data -- Terminate Immed Command | 213 |
| 6.1.18.2 Results Data -- Terminate Immed Command | 215 |
| 6.1.19 Diag - 1002h: Read Thermal Sensor | 215 |
| 6.1.19.1 SendDiag Parm Data - Read Thermal Sensor | 215 |
| 6.1.19.2 RcvDiag Data - Read Thermal Sensor | 216 |
| 6.1.20 Diag - 2002h: Reset Drive | 217 |
| 6.1.20.1 SendDiag Parm Data - Reset Drive | 217 |
| 6.1.20.2 RcvDiag Command - Reset Drive | 217 |
| 6.2 Dynamic runtime attributes (DRA) | 219 |
| 6.2.1 Attribute layout | 219 |
| 6.2.2 Attribute identifier values | 220 |
| 6.2.2.1 I_T_L nexus identifying information descriptor | 220 |
| 6.2.2.2 Attribute identifier values overview | 221 |
| 6.2.2.3 Logical unit type attributes | 222 |
| 6.2.2.4 Target type attributes | 225 |
| 6.2.2.5 Initiator type attributes | 225 |
| 6.3 Inquiry Vital Product Data Parameters (IP) | 227 |
| 6.3.1 IP 00h: Supported Vital Product Data Pages | 227 |
| 6.3.1.1 Returned Data - Inquiry Page 00h: Supported Inquiry Pages | 227 |
| 6.3.2 IP 03h: Firmware Designation | 228 |
| 6.3.2.1 Returned Data - IP 03h: Firmware Designation | 229 |
| 6.3.3 IP 80h: Unit Serial Number | 230 |
| 6.3.3.1 Returned Data - IP 80h: Unit Serial Number | 230 |
| 6.3.4 IP 83h: Device Identification | 230 |
| 6.3.4.1 Returned Data - Inquiry Page 83h: Device Identification | 231 |
| 6.3.4.1.1 T10 vendor ID designation descriptor | 232 |
| 6.3.4.1.2 Logical Unit (NAA) - WWNN designation descriptor | 233 |
| 6.3.4.1.3 Relative target port identifier designation descriptor | 234 |
| 6.3.4.1.4 Port Name (NAA) - WWPN designation descriptor | 235 |
| 6.3.4.1.5 Target Device Name (NAA) designation descriptor (SAS only) | 235 |
| 6.3.5 IP 86h: Extended INQUIRY Data | 236 |
| 6.3.6 IP 87h: Mode Page Policy | 238 |
| 6.3.6.1 Returned Data - IP 87h: Mode Page Policy | 239 |
| 6.3.6.1.1 Mode page policy descriptor | 239 |
| 6.3.7 IP 88h: SCSI ports | 240 |
| 6.3.7.1 Returned Data - IP 88h: SCSI ports | 240 |
| 6.3.8 IP 90h: Protocol-Specific Logical Unit Information | 242 |
| 6.3.8.1 Returned Data - IP 90h: SCSI ports | 242 |
| 6.3.8.2 Logical unit information descriptor | 243 |
| 6.3.9 IP B0h: Sequential-Access device capabilities | 243 |
| 6.3.9.1 Returned Data - IP B0h: Sequential-Access device capabilities | 244 |
| 6.3.10 IP B1h: Manufacturer-assigned Serial Number | 244 |
| 6.3.10.1 Returned Data - IP B1h: Manufacturer-assigned Serial Number | 244 |
| 6.3.11 IP B3h: Automation Device Serial Number | 245 |
| 6.3.11.1 Returned Data - IP B3h: Automation Device Serial Number | 245 |
| 6.3.12 IP B4h: Data Transfer Device Element Address | 245 |
| 6.3.12.1 Returned Data - IP B4h: Data Transfer Device Element Address | 246 |
| 6.3.13 IP B5h: Logical Block Protection | 246 |
| 6.3.13.13 Returned Data - IP B5h: Logical Block Protection | 247 |
| 6.3.14 IP C0h: Drive Component Revision Levels | 248 |
| 6.3.14.1 Returned Data - IP C0h: Drive Component Revision Levels | 249 |
| 6.3.15 IP C1h: Drive Serial Numbers | 250 |
| 6.3.15.1 Returned Data - IP C1h: Drive Serial Numbers | 250 |
| 6.3.16 IP C2h: Drive Bar codes | 251 |
| 6.3.16.1 Returned Data - IP C2h: Drive Bar codes | 251 |
| 6.3.16.1.1 Bar code descriptor | 252 |
| 6.3.16.1.1.1 00h -- 11S Bar code descriptor | 252 |
| 6.3.17 IP C3h: Subcomponent Version List | 253 |
| 6.3.17.1 Returned Data - IP C3h: Subcomponent Version List | 253 |
| 6.3.17.1.1 Subcomponent version descriptor | 254 |
| 6.3.18 IP C7h: Device Unique Configuration Data | 254 |
| 6.3.18.1 Returned Data - IP C7h: Device Unique Configuration Data | 254 |
| 6.3.19 IP C8h: Mode Parameter Default Settings | 254 |
| 6.3.19.1 Returned Data - IP C8h: Mode Parameter Default Settings | 254 |
| 6.4 Log Parameters (LP) | 255 |
| 6.4.1 Log Page Layout | 257 |
| 6.4.2 Log Parameter Layout | 257 |
| 6.4.2.1 Log Parameter Byte 2 -- Control Byte | 257 |
| 6.4.3 General Log Parameter Reset Behavior | 258 |
| 6.4.4 Supported Log Pages, Supported Log pages and Subpages, and Supported Subpages | 258 |
| 6.4.4.1 LP 00h: Supported Log Pages | 258 |
| 6.4.4.2 LP [PAGE CODE][FFh]: Supported subpages | 259 |
| 6.4.5 LP 02h: Write Error Counters | 260 |
| 6.4.5.1 Parameter Definitions (02h) | 261 |
| 6.4.6 LP 03h: Read Error Counters | 261 |
| 6.4.6.1 Parameter Definitions (03h) | 262 |
| 6.4.7 LP 06h: Non-Medium Errors | 262 |
| 6.4.7.1 Parameter Definitions (06h) | 263 |
| 6.4.8 LP 0Ch: Sequential-Access Device | 263 |
| 6.4.8.1 Parameter Definitions (0Ch) | 264 |
| 6.4.9 LP 0Dh[01h]: Environmental Reporting (LTO9 and later) | 265 |
| 6.4.9.1 Parameter Reset Behavior | 265 |
| 6.4.9.2 Parameter Definitions | 265 |
| 6.4.9.2.1 Temperature Report parameter data | 266 |
| 6.4.9.2.2 Relative Humidity Report parameter data | 267 |
| 6.4.10 LP 11h: DT Device Status | 268 |
| 6.4.10.1 Parameter Definitions (11h) | 269 |
| 6.4.10.1.1 Very high frequency data log parameter | 270 |
| 6.4.10.1.2 Very high frequency polling delay log parameter | 274 |
| 6.4.10.1.3 Extended very high frequency data log parameter | 275 |
| 6.4.10.1.4 Primary port status log parameter(s) | 276 |
| 6.4.10.1.5 Fibre Channel port status data | 277 |
| 6.4.10.1.6 Serial Attached SCSI port status data | 279 |
| 6.4.10.1.7 Potential conflict list entries present log parameter | 280 |
| 6.4.10.1.8 Potential conflict list log parameter(s) | 281 |
| 6.4.10.1.9 DT device primary port physical interface information | 282 |
| 6.4.10.1.10 Medium VolSer | 283 |
| 6.4.10.1.11 Medium Status Data | 284 |
| 6.4.10.1.12 Drive Status Data | 285 |
| 6.4.10.1.13 Primary Port Features | 286 |
| 6.4.10.1.14 Encryption Control Descriptor | 287 |
| 6.4.11 LP 12h: TapeAlert Response | 287 |
| 6.4.11.1 Parameter Definitions (12h) | 288 |
| 6.4.12 LP 14h: Device Statistics | 288 |
| 6.4.12.1 Parameter Reset Behavior (14h) | 289 |
| 6.4.12.2 Parameter Definitions (14h) | 289 |
| 6.4.12.3 Log parameter formats | 294 |
| 6.4.12.3.1 Device statistics data counter log parameter layout | 294 |
| 6.4.12.3.2 Device statistics medium type log parameter layout | 295 |
| 6.4.12.3.2.1 Device statistics medium type descriptor | 295 |
| 6.4.12.3.2.1.1 Supported descriptors | 296 |
| 6.4.12.3.3 Device statistics string data log parameter layout | 297 |
| 6.4.13 LP 16h: Tape diagnostic data | 298 |
| 6.4.13.1 Parameter Definitions (16h) | 298 |
| 6.4.14 LP 17h: Volume Statistics | 301 |
| 6.4.14.1 Parameter Definitions | 302 |
| 6.4.14.2 Parameter formats | 307 |
| 6.4.14.2.1 Data counter log parameter layout | 307 |
| 6.4.14.2.2 Volume statistics string data log parameter layout | 307 |
| 6.4.14.2.3 Volume statistics partition record log parameter layout | 308 |
| 6.4.15 LP 18h: Protocol-specific port | 308 |
| 6.4.15.1 Parameter Reset Behavior (18h) | 309 |
| 6.4.15.2 Parameter Definitions (18h) | 309 |
| 6.4.16 LP 1A: Power Condition Transitions | 313 |
| 6.4.16.1 Parameter Definitions (1Ah) | 313 |
| 6.4.17 LP 1Bh: Data Compression | 314 |
| 6.4.17.1 Parameter Definitions | 314 |
| 6.4.17.2 Parameter layout | 316 |
| 6.4.18 LP 2Eh: TapeAlerts | 316 |
| 6.4.18.1 Parameter Reset Behavior (2Eh) | 317 |
| 6.4.18.2 Parameter Definitions (2Eh) | 317 |
| 6.4.19 LP 30h: Tape Usage | 319 |
| 6.4.19.1 Parameter Definitions (30h) | 319 |
| 6.4.20 LP 31h: Tape capacity | 320 |
| 6.4.20.1 Parameter Reset Behavior (31h) | 320 |
| 6.4.20.2 Parameter Definitions(31h) | 320 |
| 6.4.21 LP 32h: Data compression | 321 |
| 6.4.21.1 Parameter Definitions (32h) | 321 |
| 6.4.22 LP 33h: Write Errors | 322 |
| 6.4.22.1 Parameter Definitions (33h) | 322 |
| 6.4.23 LP 34h: Read Forward Errors | 324 |
| 6.4.23.1 Parameter Definitions (34h) | 324 |
| 6.4.24 LP 37h: Performance Characteristics | 326 |
| 6.4.24.1 Parameter Reset Behavior (37h) | 326 |
| 6.4.24.2 Parameter Definitions (37h) | 326 |
| 6.4.25 LP 38h: Blocks/Bytes Transferred | 338 |
| 6.4.25.1 Parameter Definitions (38h) | 338 |
| 6.4.25.2 Identifying information of data set | 341 |
| 6.4.26 LP 39h: Host Port 0 Interface Errors | 341 |
| 6.4.26.1 Parameter Definitions (39h) | 341 |
| 6.4.27 LP 39h[02h]: Host Port 0 Physical Interface | 342 |
| 6.4.27.1 Parameter Definitions (39h[02h]) | 342 |
| 6.4.27.1.1 Host Port SFF-8472 Address A2h | 342 |
| 6.4.28 LP 3Ah: Drive control verification | 343 |
| 6.4.29 LP 3Bh: Host Port 1 Interface Errors | 344 |
| 6.4.29.1 Parameter Definitions (3Bh) | 344 |
| 6.4.30 LP 3Bh[02h]: Host Port 1 Physical Interface | 344 |
| 6.4.30.1 Parameter Definitions (3Bh[02h]) | 344 |
| 6.4.31 LP 3Ch: Drive usage information | 344 |
| 6.4.31.1 Parameter Reset Behavior (3Ch) | 345 |
| 6.4.31.2 Parameter Definitions (3Ch) | 345 |
| 6.4.32 LP 3Dh: Subsystem Statistics | 347 |
| 6.4.32.1 Parameter Reset Behavior (3Dh) | 347 |
| 6.4.32.2 Parameter Definitions (3Dh) | 347 |
| 6.4.33 LP 3Eh: Engineering Use | 349 |
| 6.4.34 LP 3Eh[3Ch]: Drive Control Statistics | 349 |
| 6.5 Medium auxiliary memory attributes (MAM) | 351 |
| 6.5.1 MAM attribute layout | 351 |
| 6.5.2 Attribute identifier values | 352 |
| 6.5.2.1 Attribute identifier values overview | 352 |
| 6.5.2.2 Device type attributes | 352 |
| 6.5.2.3 Medium type attributes | 355 |
| 6.5.2.4 Host type attributes | 357 |
| 6.5.2.5 Vendor-Specific Medium Type Attributes | 360 |
| 6.6 Mode Parameters (MP) | 363 |
| 6.6.1 Mode Parameter List for Mode Select (6/10) | 363 |
| 6.6.1.1 Mode Parameter Header for Mode Select (6/10) | 363 |
| 6.6.1.2 Block Descriptor for Mode Select (6/10) | 365 |
| 6.6.2 Mode Parameter List for Mode Sense (6/10) | 366 |
| 6.6.2.1 Mode Parameter Header for Mode Sense (6/10) | 367 |
| 6.6.2.2 Block Descriptor for Mode Sense (6/10) | 368 |
| 6.6.3 Mode Page Layout | 368 |
| 6.6.4 Supported Mode Pages | 369 |
| 6.6.5 MP 01h: Read-Write Error Recovery | 370 |
| 6.6.6 MP 02h: Disconnect-Reconnect | 371 |
| 6.6.7 MP 0Ah: Control | 373 |
| 6.6.8 MP 0Ah[01h]: Control Extension | 374 |
| 6.6.9 MP 0Ah[F0h]: Control Data Protection | 375 |
| 6.6.10 MP 0Fh: Data Compression | 377 |
| 6.6.11 MP 10h: Device Configuration | 379 |
| 6.6.12 MP 10h[01h]: Device Configuration Extension | 381 |
| 6.6.13 MP 11h: Medium Partition Page | 384 |
| 6.6.14 MP 18h: Protocol-Specific Logical Unit | 389 |
| 6.6.14.1 MP 18h: Fibre Channel Logical Unit | 389 |
| 6.6.14.2 MP 18h: SAS Logical Unit | 390 |
| 6.6.15 MP 19h: Protocol specific port | 390 |
| 6.6.15.1 MP 19h: FCP port | 391 |
| 6.6.15.2 MP 19h: SAS port | 392 |
| 6.6.16 MP 1Ah: Power Condition | 393 |
| 6.6.17 MP 1Ch: Informational Exceptions Control | 394 |
| 6.6.18 MP 1Dh: Medium Configuration | 397 |
| 6.6.19 MP 24h: Vendor-Specific | 398 |
| 6.6.20 MP 2Fh: Behavior Configuration | 401 |
| 6.6.21 MP 30h: Device Attribute Settings | 404 |
| 6.6.21.1 MP 30h: Directory Listing - Device Attribute Settings | 404 |
| 6.6.21.2 Supported subpage list - Device Attribute Settings | 405 |
| 6.6.21.3 MP 30h[01h-02h]: Ethernet attributes - Device attribute settings | 406 |
| 6.6.21.3.1 Ethernet attributes overview | 406 |
| 6.6.21.3.1.1 Ethernet socket address descriptor | 406 |
| 6.6.21.3.1.1.1 Sockaddr for an IPv4 IP address | 407 |
| 6.6.21.3.1.1.2 Sockaddr for an IPv6 address | 407 |
| 6.6.21.3.2 MP 30h[01h]: Drive MAC address - Device attribute settings | 409 |
| 6.6.21.3.3 MP 30h[02h]: Drive IP address and subnet mask - Device attribute settings | 411 |
| 6.6.21.4 MP 30h[20h-(20h)]: Encryption Attributes - Device Attribute Settings | 413 |
| 6.6.21.4.1 MP 30h[20h]: Encryption mode - Device Attribute Settings | 413 |
| 6.6.21.5 MP 30h[40h-44h]: Data processing attributes - Device attribute settings | 415 |
| 6.6.21.5.1 MP 30h[40h]: SkipSync - Device attribute settings | 415 |
| 6.6.21.5.2 MP 30h[42h]: End of partition behavior control - Device attribute settings | 418 |
| 6.6.21.5.3 MP 30h[43h]: Feature switches - Device attribute settings | 419 |
| 6.6.21.5.4 MP 30h[44h]: Preferred Cartridge Type -- Device attribute settings | 421 |
| 6.6.21.5.4.1 Preferred cartridge descriptor | 422 |
| 6.6.21.5.4.2 Preferred Cartridge Type support | 422 |
| 6.6.22 MP 3Eh: Engineering Support | 423 |
| 6.7 Read/Write Buffers (RB) | 425 |
| 6.7.1 Read/Write Buffer Modes | 425 |
| 6.7.1.1 MODE [00h] -- Combined header and data | 425 |
| 6.7.1.2 MODE [02h] -- Data | 425 |
| 6.7.1.3 MODE [03h] (RB) -- Descriptor | 426 |
| 6.7.1.4 MODE [04h] (WB) -- Download microcode and activate | 426 |
| 6.7.1.5 MODE [05h] (WB) -- Download microcode, save, and activate | 427 |
| 6.7.1.6 MODE [06h] (WB) -- Download microcode with offsets and activate | 427 |
| 6.7.1.7 MODE [07h] (WB) -- Download microcode with offsets, save, and activate | 427 |
| 6.7.1.8 MODE [07h] (RB) -- Descriptor with algorithmic offset boundary | 427 |
| 6.7.1.9 MODE [0Ah] -- Echo buffer | 427 |
| 6.7.1.10 MODE [0Bh] (RB) -- Echo buffer descriptor | 427 |
| 6.7.1.11 mode [0Dh] (WB) -- Download microcode with offsets, select activation, save, and defer activate mode | 428 |
| 6.7.1.12 mode [0Fh] (WB) -- Activate deferred microcode mode | 428 |
| 6.7.1.13 MODE [1Ch] (RB) -- Error history | 428 |
| 6.7.1.13.1 Error history overview | 428 |
| 6.7.2 Supported Buffers | 429 |
| 6.7.2.1 RB 06h: Error Log (aka. Engineering Log) | 430 |
| 6.7.2.2 RB 07h: SCSI Log (aka Error log) | 431 |
| 6.7.2.3 RB 08h: World Wide Name | 431 |
| 6.7.2.4 | 432 |
| 6.7.2.5 RB 19h: Host non-volatile | 432 |
| 6.7.2.6 RB 21h: Cartridge Memory from EOD dataset | 432 |
| 6.7.2.7 RB 50h: Active IP addresses | 432 |
| 6.7.2.7.1 Active IP addresses fixed buffer (LTO5 only) | 433 |
| 6.7.2.7.2 Active IP addresses variable buffer (LTO6 and later) | 434 |
| 6.7.2.8 Supported Buffers when the mode field is 1Ch | 436 |
| 6.7.2.8.1 MODE [1Ch] 00h to 03h: Error history directory | 437 |
| 6.7.2.8.2 Error history directory entry | 439 |
| 6.7.2.8.3 MODE [1Ch] 10h to FEh: Error history data buffer | 440 |
| 6.7.2.8.3.1 MODE [1Ch] 10h to FEh: Error history data buffer overview | 440 |
| 6.7.2.8.3.2 MODE [1Ch] 10h: Current error history snapshot | 440 |
| 6.7.2.8.3.3 MODE [1Ch] 11h: Mini dump | 440 |
| 6.7.2.8.3.4 MODE [1Ch] 20h: Emergency dump | 440 |
| 6.7.2.8.3.5 MODE [1Ch] 21h to 28h: Prioritized flash dump | 440 |
| 6.7.2.8.3.6 MODE [1Ch] EFh: Error history names list | 441 |
| 6.7.2.8.3.6.1 Error history names entry | 441 |
| 6.7.2.8.4 MODE [1Ch] FEh: Clear error history I_T_L nexus | 442 |
| 6.7.2.8.5 MODE [1Ch] FFh: Clear error history I_T_L nexus and release snapshot | 442 |
| 6.8 Security Protocol Parameters (SPP) | 443 |
| 6.8.1 SPIN Pages (00h - Security Protocol Information) | 443 |
| 6.8.1.1 SPIN (00h[0000h]) - Supported Security Protocols List | 444 |
| 6.8.1.2 SPIN (00h[0001h]) - Certificate Data | 445 |
| 6.8.1.3 SPIN (00h[0002h]) - Security Compliance Information | 446 |
| 6.8.2 SPIN Pages (20h - Tape Data Encryption) | 447 |
| 6.8.2.1 SPIN (20h[0000h]) - Tape Data Encryption In Support Pages page | 447 |
| 6.8.2.2 SPIN (20h[0001h]) - Tape Data Encryption Out Support Pages page | 449 |
| 6.8.2.3 SPIN (20h[0010h]) - Data Encryption Capabilities page | 450 |
| 6.8.2.3.1 Data Encryption Algorithm Descriptor - Standard Encryption | 451 |
| 6.8.2.4 SPIN (20h[0011h]) - Supported Key Formats page | 456 |
| 6.8.2.4.1 Plaintext Key Format (00h) | 456 |
| 6.8.2.5 SPIN (20h[0012h]) - Data Encryption Management Capabilities | 457 |
| 6.8.2.6 SPIN (20h[0020h]) - Data Encryption Status page | 458 |
| 6.8.2.7 SPIN (20h[0021h]) - Next Block Encryption Status page | 461 |
| 6.8.2.7.1 Key-Associated Data (KAD) Descriptors | 464 |
| 6.8.2.8 SPIN (20h[0030h]) - Random Number page | 465 |
| 6.8.2.9 SPIN (20h[0031h]) - Device Server Key Wrapping Public Key page | 466 |
| 6.8.3 SPOUT Pages (20h - Tape Data Encryption security protocol) | 466 |
| 6.8.3.1 SPOUT (20h[0010h]) - Set Data Encryption | 467 |
| 6.8.3.2 Key-Associated Data (KAD) Descriptors | 470 |
| 6.8.3.2.1 KAD 00h - UKAD (Unathenticated KAD) | 470 |
| 6.8.3.2.2 KAD 01h - AKAD (Authenticated KAD) / DKi (Data Key Identifier) | 470 |
| 6.8.3.2.3 KAD 02h - Nonce | 471 |
| 6.8.3.2.4 KAD 03h - MKAD (Metadata) | 472 |
| Annex A. Summary of Drive Generation Differences | 473 |
| A.1. Differences in Command Timeout Values | 474 |
| A.2. Command and Parameter Differences Between Generations | 478 |
| Annex B. Error Sense Information | 481 |
| B.1. Sense Key 0 (No Sense) | 481 |
| B.2. Sense Key 1 (Recovered Error) | 482 |
| B.3. Sense Key 2 (Not Ready) | 482 |
| B.4. Sense Key 3 (Medium Error) | 484 |
| B.5. Sense Key 4 (Hardware Error) | 485 |
| B.6. Sense Key 5 (Illegal Request) | 486 |
| B.7. Sense Key 6 (Unit Attention) | 488 |
| B.8. Sense Key 7 (Data Protect) | 490 |
| B.9. Sense Key 8 (Blank Check) | 492 |
| B.10. Sense Key B (Aborted Command) | 492 |
| B.11. Sense Key D (Volume Overflow) | 493 |
| Annex C. Firmware Download | 495 |
| C.1. Identifying Level Hardware of Drive | 495 |
| C.2. Identifying the product for which the firmware image is intended | 497 |
| C.3. Download Process | 497 |
| Annex D. Protection Information CRC's | 499 |
| D.1. Reed-Solomon CRC | 499 |
| D.1.1. Reed-Solomon CRC Algorithm | 499 |
| D.1.2. Sample C program to generate Reed-Solomon CRC | 499 |
| D.1.3. Sample C program to compute and append Reed-Solomon CRC to a data block | 500 |
| D.1.4. Sample C program to to verify block with Reed-Solomon CRC | 501 |
| D.2. CRC32C (Castagnoli) | 501 |
| D.2.1. CRC32C Algorithm | 501 |
| D.2.2. Sample C program to generate CRC32C (Castagnoli) | 502 |
| D.2.3. Sample C code to compute and append CRC32C to a data block | 503 |
| D.2.4. Sample C code to verify block with CRC32C CRC | 503 |
| D.3. CRC32-IEEE | 504 |
| D.3.1. CRC32-IEEE Algorithm | 504 |
| Annex E. Best Practices | 505 |
| E.1. Overview | 505 |
| E.2. Handling of Type M cartridges | 505 |
| E.3. LTO-9 Cartridge Optimization | 505 |
| E.3.1. LTO-9 cartridge optimization overview | 505 |
| E.3.2. Usage recommendations | 505 |
| E.3.3. SCSI command additions / updates | 506 |
| Annex F. Notices | 507 |
| F.1. Trademarks | 507 |


## Tables

| Table | Page |
|-------|------|
| 1 Comparison of binary and decimal units and values | 16 |
| 2 Percentage difference between binary and decimal units | 17 |
| 3 Features of the IBM Ultrium Tape Drives and the IBM 3580 Ultrium Tape Drive | 20 |
| 4 Supported Servers and Operating Systems for Primary Interface Attachment | 21 |
| 5 LTO capacities by density, cartridges, and products | 22 |
| 6 allow_overwrite variable definition | 27 |
| 7 Partition sizes for wrap-wise partitioning (selection fields) | 30 |
| 8 Partition sizes for wrap-wise partitioning (resultant sizes) | 31 |
| 9 Partition values for L5, L6, and L7 | 32 |
| 10 Partition values for M8, L8, and L9 | 33 |
| 11 PROCESS for generating recommended access order | 36 |
| 12 Mode page policy | 40 |
| 13 Mode parameter change behavior | 42 |
| 14 Logical block with no protection information | 46 |
| 15 Logical block with protection information | 46 |
| 16 Information and ILI Behavior Summary | 50 |
| 17 ASC/ASCQ Codes Related to Cleaning | 53 |
| 18 Drive Cleaning Criteria to assert Clean Requested | 54 |
| 19 Behavior when the loaded medium has suspect integrity | 55 |
| 20 Abort Condition Handling | 57 |
| 21 Status Codes | 59 |
| 22 Topologies through which this device's port(s) can operate | 60 |
| 23 TIMESTAMP ORIGIN | 62 |
| 24 TIMESTAMP Layout | 62 |
| 25 Types of DRA attributes | 63 |
| 26 DRA attribute states | 63 |
| 27 Error to Fence State mapping | 66 |
| 28 Types of MAM attributes | 69 |
| 29 MAM attribute states | 69 |
| 30 Supported Common SCSI Commands | 76 |
| 31 Control Byte Definition | 78 |
| 32 ALLOW OVERWRITE CDB | 79 |
| 33 allow overwrite field definition | 79 |
| 34 ERASE CDB | 80 |
| 35 FORMAT MEDIUM CDB | 81 |
| 36 FORMAT MEDIUM format field | 81 |
| 37 GENERATE RECOMMENDED ACCESS ORDER CDB | 82 |
| 38 GRAO Parameter List | 83 |
| 39 GRAO - User Data Segment descriptor | 84 |
| 40 INQUIRY CDB | 84 |
| 41 Standard INQUIRY data valid LUN layout | 85 |
| 42 eServer attachment Standard Inquiry Data Valid LUN | 87 |
| 43 Standard Inquiry Product Identification Table | 88 |
| 44 Standard Inquiry protocol identifer values | 89 |
| 45 Standard Inquiry SAS maximum speed supported values | 89 |
| 46 Standard Inquiry Fibre Channel maximum speed supported values | 89 |
| 47 Standard Inquiry fips field values | 90 |
| 48 LOAD/UNLOAD CDB | 90 |
| 49 Behavior for the combinations of the RETEN, LOAD, HOLD bits | 91 |
| 50 Medium removal prevented behavior | 92 |
| 51 LOCATE (10) CDB | 92 |
| 52 LOCATE (16) CDB | 92 |
| 53 LOG SELECT CDB | 94 |
| 54 LOG SENSE CDB | 94 |
| 55 MODE SELECT (6) CDB | 95 |
| 56 MODE SELECT (10) CDB | 96 |
| 57 MODE SENSE (6) CDB | 97 |
| 58 MODE SENSE (10) CDB | 97 |
| 59 PERSISTENT RESERVE IN CDB | 98 |
| 60 PERSISTENT RESERVE IN parameter data for READ KEYS | 99 |
| 61 PERSISTENT RESERVE IN parameter data for READ RESERVATION | 100 |
| 62 PERSISTENT RESERVE IN parameter data for REPORT CAPABILITIES | 101 |
| 63 PERSISTENT RESERVE IN parameter data for READ FULL STATUS | 102 |
| 64 PERSISTENT RESERVE IN full status descriptor layout | 103 |
| 65 PERSISTENT RESERVE OUT CDB | 104 |
| 66 PERSISTENT RESERVE OUT parameter list | 105 |
| 67 PERSISTENT RESERVE OUT with REGISTER AND MOVE service action parameter list | 106 |
| 68 PREVENT ALLOW MEDIUM REMOVAL CDB | 106 |
| 69 READ CDB | 107 |
| 70 READ ATTRIBUTE CDB | 108 |
| 71 READ ATTRIBUTE with ATTRIBUTE VALUES service action parameter list layout | 109 |
| 72 READ ATTRIBUTE with ATTRIBUTE LIST service action parameter list layout | 110 |
| 73 READ ATTRIBUTE with LOGICAL VOLUME LIST service action parameter list layout | 110 |
| 74 READ ATTRIBUTE with PARTITION LIST service action parameter list layout | 111 |
| 75 READ ATTRIBUTE with SUPPORTED ATTRIBUTES service action parameter list layout | 111 |
| 76 READ BLOCK LIMITS CDB | 112 |
| 77 RBL parameter data | 112 |
| 78 READ BLOCK LIMITS maximum logical object identifier data | 113 |
| 79 READ BUFFER CDB | 113 |
| 80 READ DYNAMIC RUNTIME ATTRIBUTE CDB (legacy) | 114 |
| 81 READ DYNAMIC RUNTIME ATTRIBUTE CDB (standardized) | 114 |
| 82 READ DYNAMIC RUNTIME ATTRIBUTE Service Action codes | 116 |
| 83 READ DRA with SUPPORTED ATTRIBUTES service action parameter list layout | 117 |
| 84 Byte one of the parameter list data | 117 |
| 85 READ DRA with ATTRIBUTE VALUES FOR THIS I_T NEXUS service action parameter list layout | 118 |
| 86 Byte one of the parameter list data | 118 |
| 87 READ DRA with ATTRIBUTE VALUES FOR ALL I_T NEXUSES service action parameter list layout | 119 |
| 88 Byte one of the parameter list data | 119 |
| 89 READ END OF WRAP POSTITION CDB | 119 |
| 90 REOWP short form parameter data layout | 121 |
| 91 REOWP long form parameter data layout | 121 |
| 92 REOWP Wrap descriptor layout | 122 |
| 93 READ LOGGED-IN HOST TABLE CDB | 122 |
| 94 READ LOGGED-IN HOST TABLE parameter data layout | 123 |
| 95 Logged-In Host Descriptor layout | 124 |
| 96 READ POSITION CDB | 125 |
| 97 READ POSITION data layout, short form | 126 |
| 98 READ POSITION data layout, long form | 128 |
| 99 READ POSITION data layout, extended form | 129 |
| 100 RECEIVE DIAGNOSTIC RESULTS CDB | 131 |
| 101 RECEIVE RECOMMENDED ACCESS ORDER CDB | 132 |
| 102 UDS Limits page | 133 |
| 103 RAO List | 133 |
| 104 User Data Segment descriptor | 134 |
| 105 Additional information descriptor | 136 |
| 106 RELEASE UNIT (6) CDB | 137 |
| 107 RELEASE UNIT (10) CDB | 137 |
| 108 REPORT DENSITY SUPPORT CDB | 138 |
| 109 REPORT DENSITY SUPPORT data layout | 139 |
| 110 Density support data block descriptor layout | 140 |
| 111 Density information LTO-3 through LTO-6 | 141 |
| 112 Density information LTO-7 and later | 142 |
| 113 REPORT LUNS CDB | 142 |
| 114 RLUNS Logical Unit Numbers Data | 143 |
| 115 REPORT SUPPORTED OPERATION CODES CDB | 144 |
| 116 REPORT SUPPORTED OPERATION CODES reporting options field | 144 |
| 117 RSOC All_commands parameter data | 145 |
| 118 RSOC Command descriptor layout | 145 |
| 119 RSOC One_command parameter data | 146 |
| 120 RSOC One_command support values | 146 |
| 121 RSOC Command timeouts descriptor layout | 147 |
| 122 RSOC Command timeouts descriptor command specific field usage | 148 |
| 123 RSOC Command timeout values for Full-Height (at publication) | 149 |
| 124 RSOC Command timeout values for Full-Height (at publication) not returned to command | 153 |
| 125 RSOC Command timeout values for Half-Height (at publication) | 153 |
| 126 RSOC Command timeout values for Half-Height (at publication) not returned to command | 158 |
| 127 REPORT SUPPORTED TASK MANAGEMENT FUNCTIONS CDB | 159 |
| 128 REPORT SUPPORTED TASK MANAGEMENT FUNCTIONS parameter data | 159 |
| 129 REPORT TIMESTAMP CDB | 160 |
| 130 REPORT TIMESTAMP Timestamp Descriptor | 161 |
| 131 REQUEST SENSE CDB | 161 |
| 132 REQUEST SENSE Sense Data Layout | 162 |
| 133 RESERVE (6) CDB | 168 |
| 134 RESERVE (10) CDB | 168 |
| 135 REWIND CDB | 169 |
| 136 SECURITY PROTOCOL IN - A2h CDB | 169 |
| 137 SECURITY PROTOCOL OUT B5h CDB | 170 |
| 138 SEND DIAGNOSTIC CDB | 171 |
| 139 SET CAPACITY CDB | 172 |
| 140 SET CAPACITY MEDIUM FOR USE PROPORTION VALUE and resultant capacity | 173 |
| 141 SET TIMESTAMP CDB | 174 |
| 142 SET TIMESTAMP parameter list layout | 174 |
| 143 SPACE (6) CDB | 175 |
| 144 SPACE (16) CDB | 175 |
| 145 TEST UNIT READY CDB | 176 |
| 146 VERIFY (6) CDB | 177 |
| 147 WRITE CDB | 178 |
| 148 WRITE ATTRIBUTE CDB | 179 |
| 149 WRITE ATTRIBUTE parameter list layout | 180 |
| 150 WRITE BUFFER CDB | 181 |
| 151 WRITE DYNAMIC RUNTIME ATTRIBUTE CDB (legacy) | 182 |
| 152 WRITE DYNAMIC RUNTIME ATTRIBUTE CDB (standardized) | 182 |
| 153 WRITE DYNAMIC RUNTIME ATTRIBUTE parameter list | 183 |
| 154 WRITE FILEMARKS CDB | 184 |
| 155 SendDiag - Page 00h Parm Data | 187 |
| 156 RcvDiag - Page 00h Parm Data | 188 |
| 157 SendDiag - Page 80h Parm Data | 189 |
| 158 RcvDiag Data - Page 80h Results | 190 |
| 159 RcvDiag Data - Page 80h Results | 191 |
| 160 Diag SIM Data Structure | 193 |
| 161 Diag SIM Data Structure | 194 |
| 162 Diag MIM Data Structure | 196 |
| 163 Supported Page 80 Diag Routines | 198 |
| 164 SendDiag CDB - Self Test | 199 |
| 165 Primary Port Wrap Test SendDiag Parm Data | 200 |
| 166 SendDiag Parm Data - POST A | 201 |
| 167 SendDiag Parm Data - POST B | 202 |
| 168 SendDiag Parm Data - POST C Media Test | 202 |
| 169 SendDiag Parm Data - POST D Head Test | 203 |
| 170 SendDiag Parm Data - Medium Calibration Audit | 204 |
| 171 RcvDiag Data - Medium Calibration Audit | 205 |
| 172 Common results descriptor | 206 |
| 173 SendDiag Parm Data - Force Dump | 207 |
| 174 SendDiag Parm Data - Write Dump to Cartridge | 207 |
| 175 SendDiag Parm Data - Force Mini Dump | 208 |
| 176 Send Diagnostic Parameter Data - Create FMR Cartridge | 209 |
| 177 Send Diagnostic Parameter Data - Unmake FMR Cartridge | 210 |
| 178 Send Diagnostic Parameter Data - Use FMR Cartridge | 211 |
| 179 SendDiag Parm Data - Set Traps | 211 |
| 180 SendDiag Parm Data - Remove Traps | 212 |
| 181 RcvDiag Data - Remove Traps | 213 |
| 182 Send Data -- (Diag 0210h) Terminate Immed Command | 213 |
| 183 Supported Commands in the Terminate Immediate Command diagnostic | 215 |
| 184 Diag parameter data - Read Thermal Sensor | 215 |
| 185 Receive Diag parameter data - Read Thermal Sensor | 216 |
| 186 SendDiag Parm Data - Reset Drive | 217 |
| 187 DRA ATTRIBUTE layout | 219 |
| 188 DRA attribute FORMAT field | 219 |
| 189 DRA I_T_L nexus identifying information layout | 220 |
| 190 DRA attribute identifier range assignments | 221 |
| 191 DRA Logical unit type attributes | 222 |
| 192 Reservation Information Dynamic Runtime Attribute value layout | 222 |
| 193 DRA RESERVATION TYPE values | 223 |
| 194 REGISTRATION INFORMATION DRA attribute value layout | 223 |
| 195 PREVENT ALLOW MEDIUM REMOVAL INFORMATION DRA attribute value layout | 224 |
| 196 DRA Target type attributes | 225 |
| 197 DRA Initiator type attributes | 225 |
| 198 IP 00h Supported Vital Product Data Inquiry Page | 227 |
| 199 IP 03h Firmware Designation Page | 229 |
| 200 IP 80h Unit Serial Number Inquiry Page | 230 |
| 201 IP 83h Device Identification VPD page | 231 |
| 202 T10 vendor ID based designation descriptor of IP 83h | 232 |
| 203 Logical Unit (NAA) - WWNN designation descriptor of IP 83h | 233 |
| 204 Relative target port identifier designation descriptor of IP 83h | 234 |
| 205 Port Name (NAA) - WWPN designation descriptor of IP 83h | 235 |
| 206 Target Device Name (NAA) designation descriptor of IP 83h (SAS only) | 235 |
| 207 IP 86h Extended INQUIRY Data VPD page | 236 |
| 208 IP 87h Mode Page Policy page | 239 |
| 209 Mode page policy descriptor of IP 87h | 239 |
| 210 IP 88h SCSI Ports VPD page | 240 |
| 211 SCSI port designation descriptor of IP 88h | 241 |
| 212 Target port descriptor of IP 88h | 241 |
| 213 IP 90h Protocol-Specific Logical Unit Information VPD page for SAS SSP | 242 |
| 214 Logical unit information descriptor for SAS SSP of IP 90h | 243 |
| 215 IP B0h Sequential-Access Device Capabilities Page | 244 |
| 217 IP B3h Automation Device Serial Number VPD page | 245 |
| 218 IP 84h Data Transfer Device Element Address VPD page | 246 |
| 219 IP 85h Logical Block Protection VPD page | 247 |
| 220 IP C0h: Drive Component Revision Levels | 249 |
| 221 PLATFORM definition of IP C0h | 250 |
| 222 IP C1h: Drive Serial Numbers | 250 |
| 223 IP C2h: Drive Bar codes | 251 |
| 224 Bar code descriptor | 252 |
| 225 Bar code descriptor | 252 |
| 226 IP C3h: Subcomponent Version List | 253 |
| 227 Subcomponent version descriptor | 254 |
| 228 Subcomponent designator-length-description | 254 |
| 229 Supported log pages | 255 |
| 230 Log page layout | 257 |
| 231 Log Parameter Layout | 257 |
| 232 Supported Log Pages | 259 |
| 233 Supported Subpages | 260 |
| 234 Supported Subpage descriptor | 260 |
| 235 LP 02h: Write Error Log Parameters | 261 |
| 236 LP 03h: Read Error Log Parameters | 262 |
| 237 LP 06h: Non-Medium Errors log parameter codes | 263 |
| 238 LP 0Ch: Sequential-Access Device log parameters | 264 |
| 239 LP 0Dh[01h]: Environmental Reporting log parameters | 265 |
| 240 Temperature Report parameter data layout | 266 |
| 241 Relative Humidity Report parameter data layout | 267 |
| 242 LP 11h: DT Device Status log page | 269 |
| 243 DT Device Status log parameters of LP 11h | 269 |
| 244 Very high frequency data log parameter layout of LP 11h | 270 |
| 245 VHF data descriptor | 271 |
| 246 Very high frequency polling delay log parameter layout of LP 11h | 274 |
| 247 Extended very high frequency data log parameter layout of LP 11h | 275 |
| 248 Primary port status log parameter(s) layout of LP 11h | 276 |
| 249 Fibre Channel port status data layout of LP 11h | 277 |
| 250 Serial Attached SCSI port status data layout of LP 11h | 279 |
| 251 NEGOTIATED PHYSICAL LINK RATE values | 279 |
| 252 Potential conflict list entries present log parameter of LP 11h | 280 |
| 253 Potential conflict list log parameter of LP 11h | 281 |
| 254 DT device primary port status log parameter(s) layout | 282 |
| 255 Medium Volume Label Serial Number log parameter of LP 11h | 283 |
| 256 Medium Volume Label Serial Number log parameter of LP 11h | 284 |
| 257 Medium Volume Label Serial Number log parameter of LP 11h | 285 |
| 258 Primary Port Features log parameter of LP 11h | 286 |
| 259 Encryption Control Descriptor log parameter of LP 11h | 287 |
| 260 TapeAlert Response log page | 288 |
| 261 LP 14h: Device Statistics log parameter codes | 289 |
| 262 Device statistics data counter log parameter layout of LP 14h | 294 |
| 263 Device statistics medium type log parameter layout of LP 14h | 295 |
| 264 Device statistics medium type descriptor layout | 295 |
| 265 LTO-5 Device statistics medium descriptor support | 296 |
| 266 LTO-6 Device statistics medium descriptor support | 296 |
| 267 LTO-7 Device statistics medium descriptor support | 296 |
| 268 LTO-8 Device statistics medium descriptor support | 297 |
| 269 LTO-9 Device statistics medium descriptor support | 297 |
| 270 Device statistics string data log parameter layout of LP 14h | 297 |
| 271 LP 16h: Tape diagnostic data log page layout | 298 |
| 272 Tape diagnostic data log parameter layout of LP 16h | 299 |
| 273 Volume Statistics log subpage codes of LP 17h | 301 |
| 274 LP 17h: Volume statistics log parameters | 302 |
| 275 Volume statistics data counter log parameter layout of LP 17h | 307 |
| 276 Volume statistics string data log parameter layout of LP 17h | 307 |
| 277 Volume statistics partition log parameter layout of LP 17h | 308 |
| 278 Volume statistics partition record descriptor layout of LP 17h | 308 |
| 279 LP 18h Protocol-Specific log page for SAS | 309 |
| 280 Protocol-Specific Port log parameter for LP 18h for SAS | 310 |
| 281 Parameter codes for the Power Condition Transitions log page LP 1Ah | 314 |
| 282 LP 1Bh: Data compression log parameters | 315 |
| 283 Data compression counter log parameter layout of LP 1Bh | 316 |
| 284 LP 2Eh Supported TapeAlerts | 318 |
| 285 LP 30h: Tape Usage log parameter codes | 320 |
| 286 LP 31h: Tape capacity log parameters | 321 |
| 287 LP 32h: Data Compression Log Parameters | 322 |
| 288 LP 33h: Write Errors log parameter codes | 322 |
| 289 LP 34h: Read Error Counters log parameter codes | 324 |
| 290 LP 37h: Performance Characteristics: Quality Summary | 329 |
| 291 LP 37h: Performance Characteristics: Device Usage | 329 |
| 292 LP 37h: Performance Characteristics: Host Commands | 330 |
| 293 LP 37h: Performance Characteristics: Host Initiators | 333 |
| 294 LP 37h: Performance Characteristics: Host Recovery (by port) | 334 |
| 295 LP 37h: Performance Characteristics: Mode Phase Timing Windows | 334 |
| 296 LP 37h: Performance Characteristics: Servo Speed Characteristics | 337 |
| 297 LP 37h: Performance Characteristics: Static Capacity | 337 |
| 298 LP 37h: Performance Characteristics: Active Capacity | 337 |
| 299 LP 37h: Performance Characteristics: Static Capacity per Partition | 337 |
| 300 LP 38h: Blocks/Bytes Transferred log parameter codes | 339 |
| 301 Identifying information of data set layout | 341 |
| 302 LP 39h: Host Port Interface Errors log parameter codes | 342 |
| 303 LP 39h[02h]: Host Port 0 Physical Interface log parameter codes | 342 |
| 304 Host Port SFF-8472 Address A2h log parameter(s) layout | 343 |
| 305 LP 3Ch: Drive usage information log parameters | 345 |
| 306 LP 3Dh: Subsystem Statistics log parameter codes | 347 |
| 307 MAM ATTRIBUTE layout | 351 |
| 308 MAM attribute format field | 351 |
| 309 MAM attribute identifier range assignments | 352 |
| 310 MAM Device type attributes | 352 |
| 311 DEVICE VENDOR/SERIAL NUMBER MAM attribute layout | 354 |
| 312 MAM Medium type attributes | 355 |
| 313 Medium density code by product | 356 |
| 314 MAM Capacity by cartridge type | 356 |
| 315 MEDIUM TYPE and MEDIUM TYPE INFORMATION MAM attributes | 357 |
| 316 SUPPORTED DENSITY CODES attribute layout | 357 |
| 317 MAM Host type attributes | 357 |
| 318 TEXT LOCALIZATION IDENTIFIER MAM attribute values | 358 |
| 320 MAM Vendor-Specific Medium Type Attributes | 360 |
| 321 Mode Parameter List for Mode Select (6) | 363 |
| 322 Mode Parameter List for Mode Select (10) | 363 |
| 323 Mode Parameter Header for Mode Select (6) | 364 |
| 324 Mode Parameter Header for Mode Select (10) | 364 |
| 325 Block Descriptor for Mode Select | 365 |
| 326 Mode Parameter List for Mode Sense (6) | 366 |
| 327 Mode Parameter List for Mode Sense (10) | 367 |
| 328 Mode Parameter Header for Mode Sense (6) | 367 |
| 329 Mode Parameter Header for Mode Sense (10) | 367 |
| 330 Block Descriptor for Mode Sense (10) or Mode Sense (6) | 368 |
| 331 Mode Page Layout | 368 |
| 332 Mode Page Subpage Layout | 369 |
| 333 MP 01h Read-Write Error Recovery mode page | 370 |
| 334 MP 02h Disconnect-Reconnect mode page | 372 |
| 335 MP 0Ah Control mode page | 373 |
| 336 MP 0Ah[01h] Control Extension mode page | 375 |
| 337 MP 0Ah[F0h] Control Data Protection mode page layout | 376 |
| 338 MP 0Fh Data Compression mode page | 378 |
| 339 MP 10h Device Configuration mode page | 379 |
| 340 MP 10h[01h] Device Configuration Extension mode page | 382 |
| 341 MP 11h Medium Partition mode page | 385 |
| 342 MP 18h Fibre Channel Logical Unit mode page | 389 |
| 343 MP 18h SAS Logical Unit mode page | 390 |
| 344 MP 19h Fibre Channel Port mode page | 391 |
| 345 MP 19h SAS Port mode page | 392 |
| 346 MP 1Ah: Power Condition mode page layout | 393 |
| 347 MP 1Ch Informational Exceptions Control mode page | 395 |
| 348 MP 1Dh Medium Configuration mode page | 397 |
| 349 Vendor-Specific mode page | 398 |
| 350 MP 2Fh Behavior Configuration Mode Page | 401 |
| 351 MP 30h: Directory Listing - Device Attribute Settings mode page layout | 404 |
| 352 Ethernet socket address descriptor | 406 |
| 353 Sockaddr layout for IPv4 | 407 |
| 354 Sockaddr layout for IPv6 | 407 |
| 355 MP 30h[01h] Drive MAC address | 409 |
| 356 Drive port MAC address descriptor of MP 30h[01h] | 410 |
| 357 MP 30h[02h] Drive IP address and subnet mask subpage | 411 |
| 358 Drive Ethernet port descriptor of MP 30h[02h] | 412 |
| 359 MP 30h[20h] Encryption mode mode page | 413 |
| 360 Expected Encryption settings of MP 30h[20h] | 414 |
| 361 MP 30h[40h] SkipSync - Device attribute settings mode page layout | 415 |
| 362 MP 30h[42h] End of partition behavior control - Device attribute settings mode page layout | 418 |
| 363 MP 30h[43h] Feature switches - Device attribute settings mode page layout | 419 |
| 364 MP 30h[44h] Preferred Cartridge Type -- Device attribute settings mode page layout | 421 |
| 365 Preferred cartridge descriptor layout | 422 |
| 366 Supported "medium type--preferred density" pairs | 422 |
| 367 MP 3Eh: Engineering Support mode page | 423 |
| 368 READ BUFFER header | 425 |
| 369 READ BUFFER descriptor | 426 |
| 370 Echo buffer descriptor | 428 |
| 371 Supported Buffer IDs | 429 |
| 372 Error Log Buffer (06h) | 430 |
| 373 SCSI Log Buffer (07h) | 431 |
| 374 World Wide Name Buffer (08h) | 432 |
| 375 Active IP addresses fixed buffer layout (50h) | 433 |
| 376 Active IP addresses variable buffer layout (50h) | 434 |
| 377 Ethernet port variable descriptor layout | 435 |
| 378 Error history buffer id field | 436 |
| 379 Summary of error history directory device actions | 437 |
| 380 Error history directory | 438 |
| 381 Error history directory entry | 439 |
| 382 MODE[1Ch] EFh: Error history names list layout | 441 |
| 383 Error history names entry | 441 |
| 384 SPIN (00h) Security Protocol Specific Definitions for Security Protocol 00h | 443 |
| 385 SPIN (00h[0000h]) Supported Security Protocols List Structure | 444 |
| 386 SPIN (00h[0001h]) - Certificate Data Structure | 445 |
| 387 SPIN (00h[0002h]) - Security Compliance Information Structure | 446 |
| 388 SPIN (20h[0000h]) - Tape Data Encryption In Support Pages Structure | 447 |
| 389 SPIN (20h[0000h]) - Tape Data Encryption In page codes (SECURITY PROTOCOL SPECIFIC VALUES) | 448 |
| 390 SPIN (20h[0001h]) - Tape Data Encryption Out Support Pages Structure | 449 |
| 391 Tape Data Encryption Out page codes | 449 |
| 392 SPIN (20h[0010h]) - Data Encryption Capabilities page | 450 |
| 393 Data Encryption Algorithm descriptor list returned | 451 |
| 394 Data Encryption Algorithm Descriptor - Standard Encryption Structure | 451 |
| 395 SPIN (20h[0011h]) - Supported Key Formats page Structure | 456 |
| 396 Supported Key Formats | 456 |
| 397 KEY FORMAT 00h - Plaintext Key Format Structure | 456 |
| 398 SPIN (20h[0012h]) - Data Encryption Management Capabilities page | 457 |
| 399 SPIN (20h[0020h]) - Data Encryption Status page | 458 |
| 400 parameters control field | 459 |
| 401 SPIN (20h[0021h]) - Next Block Encryption Status page | 461 |
| 402 compression status field | 461 |
| 403 encryption status field | 462 |
| 404 SPIN (20h[0021h]) - KAD Parameters by Mode | 463 |
| 405 SPIN (20h[0030h]) Random Number page | 465 |
| 406 SPIN (20h[0031h]) Device Server Key Wrapping Public Key page | 466 |
| 407 SPOUT (20h) - Security Protocol Specific Definitions for Security Protocol 20h | 466 |
| 408 SPOUT (20h[0010h]) - Set Data Encryption page | 467 |
| 409 SPOUT (20h[0010h]) - KAD Parameters by Mode | 469 |
| 411 KAD 01h - AKAD (Authenticated KAD) / DKi (Data Key Identifier) | 470 |
| 412 KAD 02h - Nonce | 471 |
| 413 KAD 03h - MKAD (Metadata) | 472 |
| 414 Command Timeout Values (Ultrium 1, 2, and 3 Full-Height) - Alphabetic Sort | 475 |
| 415 Command Timeout Values (Ultrium 3 Half-Height and Ultrium 4) - Alphabetic Sort | 477 |
| A.1 Command and Parameter differences between generations | 478 |
| B.1 ASC, and ASCQ Summary for Sense Key 0 (No Sense) | 481 |
| B.2 ASC, and ASCQ Summary for Sense Key 1 (Recovered Error) | 482 |
| B.3 ASC, and ASCQ Summary for Sense Key 2 (Not Ready) | 482 |
| B.4 ASC, and ASCQ Summary for Sense Key 3 (Medium Error) | 484 |
| B.5 ASC, and ASCQ Summary for Sense Key 4 (Hardware Error) | 485 |
| B.6 ASC, and ASCQ Summary for Sense Key 5 (Illegal Request) | 486 |
| B.7 ASC, and ASCQ Summary for Sense Key 6 (Unit Attention) | 488 |
| B.8 ASC, and ASCQ Summary for Sense Key 7 (Data Protect) | 490 |
| B.9 ASC, and ASCQ Summary for Sense Key 8 (Blank Check) | 492 |
| B.10 ASC, and ASCQ Summary for Sense Key B (Aborted Command) | 492 |
| B.11 ASC, and ASCQ Summary for Sense Key D (Volume Overflow) | 493 |
| C.1 Load ID and RU Name Designation for LTO-9+ | 495 |
| C.2 Load ID and RU Name Designation for LTO-5 through LTO-8 | 496 |
| C.3 Firmware Image | 497 |


## Figures

| Figure | Page |
|--------|------|
| Figure 1 -- IBM System Storage Ultrium Tape Drive Models. | 19 |
| Figure 2 -- Append-only mode flowchart | 28 |
| Figure 3 -- Wrap-wise partitioning | 29 |
| Figure 4 -- Example Logical Object Identifier (LOI) sort order | 38 |
| Figure 5 -- Example RAO with Unload sort order | 39 |
| Figure 6 -- Programmable early warning example | 43 |
| Figure 7 -- Protection information shown in relation to logical objects and format specific symbols | 45 |
| Figure 8 -- Dynamic runtime attributes focus | 64 |
| Figure 9 -- Example of temperature thresholds | 74 |
