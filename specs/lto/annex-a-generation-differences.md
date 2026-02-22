# Annex A. Summary of Drive Generation Differences

This chapter provides a summary of the differences in host attachment protocol between:

a) the System Storage Ultrium 9 Tape Drive (Generation 9);
b) the System Storage Ultrium 8 Tape Drive (Generation 8);
c) the System Storage Ultrium 7 Tape Drive (Generation 7);
d) the System Storage Ultrium 6 Tape Drive (Generation 6); and
e) the System Storage Ultrium 5 Tape Drive (Generation 5).

The features of the Ultrium 9 Tape Drive that differ from those of the previous generations include the following:

a) Increased maximum sustained native data rate on the full-height drive
b) Makes Archive Mode Unthread enabled and non-changeable
c) Adds volume optimization of a cartridge (adds significant time to first-time load & FORMAT MEDIUM command)
d) Adds support for Recommended Access Order (RAO) on the full-height drive
e) Adds support for 12 Gb SAS
f) Maximizes the capacity of partitions by using band boundaries to guard between partitions instead of guard wraps when there are an even number of equally sized partitions created with SDP.

The features of the Ultrium 8 Tape Drive that differ from those of the previous generations include the following:

a) Increased maximum sustained native data rate on the full-height drive
b) Supports a Type M cartridge. Type M cartridge is an Ultrium 7 cartridge that is used at a 9 000 GB capacity, and is called an M8 cartridge. See 3.4--Supported Tape Cartridges.

The features of the Ultrium 7 Tape Drive that differ from those of the previous generations include the following:

a) Larger read-and-write cache
b) 32 data channels
c) Improved rewrite methodology decreases capacity loss due to errors while writing
d) Ability to disable BOP caching

The features of the Ultrium 6 Tape Drive that differ from those of the previous generations include the following:

a) Larger read-and-write cache
b) Partition capability up to four partitions (see 6.6.13--MP 11h: Medium Partition Page)
c) Improved SkipSync capability (see 6.6.21.5.1--MP 30h[40h]: SkipSync - Device attribute settings)
d) Larger compression history buffer enabling improved nominal compression ratio

The features of the Ultrium 5 Tape Drive that differ from those of the previous generations include the following:

a) Full-Height and Half-Height drive option with:
   A) Fibre Channel 8Gbit/sec Interface, or
   B) Serial Attached SCSI (SAS) 6Gbit/sec Interface
b) Larger read-and-write cache
c) Encryption of data on Ultrium 4 and Ultrium 5 cartridges
d) T10 key management method
e) Transparent management method
   A) when using IBM device driver,
   B) when in an IBM library, or
   C) when using T10 External Encryption Control
f) 14 Speeds in Digital Speed Matching
g) Partition capability up to two partitions (see 6.6.13--MP 11h: Medium Partition Page)
h) Append-only mode (also known as Data-safe mode) (see 4.2.3 on page 26)
i) MP 30h[40h]: SkipSync - Device attribute settings (see 6.6.21.5.1 on page 415)
j) MP 30h[42h]: End of partition behavior control - Device attribute settings (see 6.6.21.5.2 on page 418)
k) Dynamic runtime attributes (DRA) (see 6.2 on page 219)
l) MP 1Ah: Power Condition (see 6.6.16 on page 393) for user controlled power management
m) Ethernet port for configuration/debug (see 6.6.21.3--MP 30h[01h-02h]: Ethernet attributes - Device attribute settings)
n) Standards based setting of time (see 4.17--Device Clocks)
o) Logical block protection (see 4.9 on page 43) (i.e., CRC on bus; fixity checks; tape checksum)
p) Command timeout values reported at runtime with REPORT SUPPORTED OPERATION CODES - A3h[0Ch] (see 5.2.28 on page 143) command
q) Standards based retrieval of drive error logs (i.e., drive dump) (see 6.7.1.13--MODE [1Ch] (RB) -- Error history)
r) New standardized log pages with expanded counters:
   A) LP 14h: Device Statistics (see 6.4.12 on page 288)
   B) LP 17h: Volume Statistics (see 6.4.14 on page 301)
   C) LP 1Bh: Data Compression (see 6.4.17 on page 314)


## A.1. Differences in Command Timeout Values

Due to differences between the of the various Ultrium drive products, the maximum amount of time it takes for various SCSI commands to process and return status may be different. A list of all recommended host command time-outs from commands defined by the referenced SCSI-3 standard or by this product as vendor-unique for sequential access devices are listed with the following information for each command: the operation code, recommended timeout, and notes.

It is strongly recommended that device drivers or host software implement device reservations using the Reserve or Persistent Reserve commands. Due to the sequential nature of tape devices, many host commands are serialized, and command time-outs consequently have an additive effect. Using reservations prevents this from causing application disruptions in a multi-initiator or SAN environment. Similar additive timeout effects can occur if the host is using command Queuing (that is, simple queuing).

The time-outs are based on the time from the start of command processing, to its reported completion. Since applications are generally concerned with the time from the command being issued, to its reported completion, it should be noted that this overall time may be affected by currently processing operations. Some of these conditions include:

a) A prior command was issued with the Immediate bit set in the CDB
b) Multiple concurrent commands with Simple queuing are processed
c) Multi-initiator configurations without reservations
d) Non-host operations, such as manual unloads, power-on self tests, and so on
e) Commands issued shortly after certain aborted commands
f) Commands that force flushes when unwritten write data is in the buffer

Ultrium 5 and later tape drives support the REPORT SUPPORTED OPERATION CODES - A3h[0Ch] (see 5.2.28 on page 143) command and provide command timeout values at run time. See the REPORT SUPPORTED OPERATION CODES command for Ultrium 5 and later tape drive command timeouts. Command timeout values for Ultrium 1 through Ultrium 4 tape drives are listed in the following tables:

- Table 414, Command Timeout Values (Ultrium 1, 2, and 3 Full-Height) - Alphabetic Sort, on page 475
- Table 415, Command Timeout Values (Ultrium 3 Half-Height and Ultrium 4) - Alphabetic Sort, on page 477

### Table 414 -- Command Timeout Values (Ultrium 1, 2, and 3 Full-Height) - Alphabetic Sort

| OpCode | Command | Ultrium (min) | U2 Gen1 | U2 Gen2 | U3FH Gen1 | U3FH Gen2 | U3FH Gen3 |
|--------|---------|---------------|---------|---------|-----------|-----------|-----------|
| 19h | ERASE | 204 | 138 | 151 | N/A | 160 | 134 |
| 12h | INQUIRY | 1 | 1 | 1 | 1 | 1 | 1 |
| 1Bh | LOAD (Cartridge Insert -> BOM) | 11 | 12 | 12 | 8 | 8 | 8 |
| 1Bh | LOAD (LP4 -> BOM) | 8 | 9 | 8 | 8 | 8 | 9 |
| 2Bh/92h | LOCATE(10/16) (Normal) | 16 | 15 | 14 | 14 | 14 | 16 |
| 2Bh/92h | LOCATE(10/16) (Slow) | 173 | 138 | 151 | 127 | 165 | 140 |
| 4Ch | LOG SELECT | 1 | 1 | 1 | 1 | 1 | 1 |
| 4Dh | LOG SENSE | 1 | 1 | 1 | 1 | 1 | 1 |
| 15h/55h | MODE SELECT(6/10) | 1 | 1 | 1 | 1 | 1 | 1 |
| 1Ah/5Ah | MODE SENSE(6/10) | 1 | 1 | 1 | 1 | 1 | 1 |
| 5Eh | PERSISTENT RESERVE IN (PRIN) | 1 | 1 | 1 | 1 | 1 | 1 |
| 5Fh | PERSISTENT RESERVE OUT (PROUT) | 1 | 1 | 1 | 1 | 1 | 1 |
| 1Eh | PREVENT/ALLOW MEDIUM REMOVAL | 1 | 1 | 1 | 1 | 1 | 1 |
| 08h | READ | 18 | 18 | 18 | 16 | 16 | 17 |
| 8Ch | READ ATTRIBUTE | 1 | 1 | 1 | 1 | 1 | 1 |
| 05h | READ BLOCK LIMITS | 1 | 1 | 1 | 1 | 1 | 1 |
| 3Ch | READ BUFFER | 8 | 8 | 8 | 7 | 7 | 8 |
| 34h | READ POSITION | 1 | 1 | 1 | 1 | 1 | 1 |
| 1Ch | RECEIVE DIAGNOSTIC RESULTS | 1 | 1 | 1 | 1 | 1 | 1 |
| 17h/57h | RELEASE UNIT(6/10) | 1 | 1 | 1 | 1 | 1 | 1 |
| 44h | REPORT DENSITY SUPPORT | 1 | 1 | 1 | 1 | 1 | 1 |
| A0h | REPORT LUNS | 1 | 1 | 1 | 1 | 1 | 1 |
| A3h:0Ch | REPORT SUPPORTED OPERATION CODES | N/A | N/A | N/A | 1 | 1 | 1 |
| A3h:0Dh | REPORT SUPPORTED TASK MANAGEMENT FUNCTIONS | N/A | N/A | N/A | 1 | 1 | 1 |
| A3h:0Fh | REPORT TIMESTAMP | N/A | N/A | N/A | 1 | 1 | 1 |
| 03h | REQUEST SENSE | 1 | 1 | 1 | 1 | 1 | 1 |
| 16h/56h | RESERVE UNIT(6/10) | 1 | 1 | 1 | 1 | 1 | 1 |
| 01h | REWIND | 8 | 9 | 8 | 8 | 8 | 9 |
| A2h | SECURITY PROTOCOL IN (SPIN) | N/A | N/A | N/A | N/A | N/A | 1 |
| B5h | SECURITY PROTOCOL OUT (SPOUT) | N/A | N/A | N/A | N/A | N/A | N/A |
| 1Dh | SEND DIAGNOSTIC | 29 | 35 | 35 | 13 | 39 | 34 |
| 0Bh | SET CAPACITY | N/A | 13 | 13 | N/A | 11 | 12 |
| A4h:0Fh | SET TIMESTAMP | N/A | N/A | N/A | 1 | 1 | 1 |
| 91h | SPACE(16) (Normal) | N/A | N/A | N/A | 14 | 14 | 16 |
| 91h | SPACE(16) (Slow) | N/A | N/A | N/A | 127 | 165 | 140 |
| 11h | SPACE(6) (Normal) | 16 | 15 | 14 | 14 | 14 | 16 |
| 11h | SPACE(6) (Slow) | 173 | 138 | 151 | 127 | 165 | 140 |
| 00h | TEST UNIT READY | 1 | 1 | 1 | 1 | 1 | 1 |
| 1Bh | UNLOAD (BOM -> Cartridge Eject) | 10 | 10 | 10 | 10 | 10 | 11 |
| 1Bh | UNLOAD (LP4 -> Cartridge Eject) | 11 | 12 | 11 | 11 | 11 | 12 |
| 13h | VERIFY | 18 | 18 | 18 | | | |
| 0Ah | WRITE | 18 | 18 | 18 | N/A | 16 | 18 |
| 8Dh | WRITE ATTRIBUTE | 1 | 1 | 1 | 1 | 1 | 1 |
| 3Bh | WRITE BUFFER | 8 | 8 | 8 | 8 | 8 | 8 |
| 10h | WRITE FILEMARK | 15 | 15 | 15 | N/A | 15 | 17 |

### Table 415 -- Command Timeout Values (Ultrium 3 Half-Height and Ultrium 4) - Alphabetic Sort

| OpCode | Command | U3HH Gen1 | U3HH Gen2 | U3HH Gen3 | U4 Gen2 | U4 Gen3 | U4 Gen4 |
|--------|---------|-----------|-----------|-----------|---------|---------|---------|
| 19h | ERASE | N/A | 191 | 255 | N/A | 134 | 180 ^a^ |
| 12h | INQUIRY | 1 | 1 | 1 | 1 | 1 | 1 |
| 1Bh | LOAD (Cartridge Insert -> BOM) | 9 | 9 | 9 | 8 | 8 | 8 |
| 1Bh | LOAD (LP4 -> BOM) | 11 | 11 | 13 | 8 | 9 | 9 |
| 2Bh/92h | LOCATE(10/16) (Normal) | 20 | 20 | 22 | 14 | 16 | 21 |
| 2Bh/92h | LOCATE(10/16) (Slow) | 199 | 264 | 201 | 165 | 140 | 183 |
| 4Ch | LOG SELECT | 1 | 1 | 1 | 1 | 1 | 1 |
| 4Dh | LOG SENSE | 1 | 1 | 1 | 1 | 1 | 1 |
| 15h/55h | MODE SELECT (6/10) | 1 | 1 | 1 | 1 | 1 | 1 ^a^ |
| 1Ah/5Ah | MODE SENSE (6/10) | 1 | 1 | 1 | 1 | 1 | 1 |
| 5Eh | PERSISTENT RESERVE IN | 1 | 1 | 1 | 1 | 1 | 1 |
| 5Fh | PERSISTENT RESERVE OUT | 1 | 1 | 1 | 1 | 1 | 1 |
| 1Eh | PREVENT/ALLOW MEDIUM REMOVAL | 1 | 1 | 1 | 1 | 1 | 1 |
| 08h | READ | 21 | 21 | 23 | 16 | 17 | 22 ^b^ |
| BCh | READ ATTRIBUTE | 1 | 1 | 1 | 1 | 1 | 1 |
| 05h | READ BLOCK LIMITS | 1 | 1 | 1 | 1 | 1 | 1 |
| 3Ch | READ BUFFER | 9 | 9 | 10 | 7 | 8 | 8 |
| 34h | READ POSITION | 1 | 1 | 1 | 1 | 1 | 1 |
| 1Ch | RECEIVE DIAGNOSTIC RESULTS | 1 | 1 | 1 | 1 | 1 | 1 |
| 17h/57h | RELEASE UNIT (6/10) | 1 | 1 | 1 | 1 | 1 | 1 |
| 44h | REPORT DENSITY SUPPORT | 1 | 1 | 1 | 1 | 1 | 1 |
| A0h | REPORT LUNS | 1 | 1 | 1 | 1 | 1 | 1 |
| A3h:0Ch | REPORT SUPPORTED OP CODES | 1 | 1 | 1 | 1 | 1 | 1 |
| A3h:0Dh | REPORT SUPPORTED TASK MANAGEMENT FUNCTIONS | 1 | 1 | 1 | 1 | 1 | 1 |
| A3h:0Fh | REPORT TIMESTAMP | 1 | 1 | 1 | 1 | 1 | 1 |
| 03h | REQUEST SENSE | 1 | 1 | 1 | 1 | 1 | 1 |
| 16h/56h | RESERVE UNIT (6/10) | 1 | 1 | 1 | 1 | 1 | 1 |
| 01h | REWIND | 11 | 11 | 13 | 8 | 9 | 9 |
| A2h | SECURITY PROTOCOL IN | N/A | N/A | N/A | N/A | N/A | 1 |
| B5h | SECURITY PROTOCOL OUT | N/A | N/A | N/A | N/A | N/A | 1 |
| 1Dh | SEND DIAGNOSTIC | 13 | 39 | 40 | 13 | 34 | 35 |
| 0Bh | SET CAPACITY | N/A | 14 | 16 | N/A | 12 | 12 |
| A4h:0Fh | SET TIMESTAMP | 1 | 1 | 1 | 1 | 1 | 1 |
| 11h/91h | SPACE(16) (Normal) | 20 | 20 | 22 | 14 | 16 | 21 |
| 11h/91h | SPACE(16) (Slow) | 199 | 264 | 201 | 165 | 140 | 183 |
| 00h | TEST UNIT READY | 1 | 1 | 1 | 1 | 1 | 1 |
| 1Bh | UNLOAD (BOM -> Cartridge Eject) | 12 | 12 | 14 | 10 | 11 | 11 |
| 1Bh | UNLOAD (LP4 -> Cartridge Eject) | 14 | 14 | 16 | 11 | 12 | 13 |
| 13h | VERIFY | 21 | 21 | 23 | 16 | 17 | 22 ^b^ |
| 0Ah | WRITE | N/A | 21 | 24 | N/A | 18 | 23 ^b^ |
| BDh | WRITE ATTRIBUTE | 1 | 1 | 1 | 1 | 1 | 1 |
| 3Bh | WRITE BUFFER | 10 | 10 | 11 | 8 | 8 | 8 |
| 10h | WRITE FILEMARK | N/A | 21 | 23 | N/A | 17 | 22 ^a^ |

> ^a^ When positioned at BOP: These commands require an increased timeout when encryption is active and an out-of-band key manager is used. The command timeout should be increased by 300 seconds.

> ^b^ These commands require an increased timeout when encryption is active and an out-of-band key manager is used. The command timeout should be increased by 300 seconds.


## A.2. Command and Parameter Differences Between Generations

Table A.1 shows commands and parameters added since LTO5 and in which generation(s) it is applicable.

### Table A.1 -- Command and Parameter differences between generations

| Command or Parameter | Gen 5 | Gen 6 | Gen 7 | Gen 8 | Gen 9 |
|----------------------|-------|-------|-------|-------|-------|
| IP B5h: Logical Block Protection (see 6.3.13) | Y ^b^ | Y ^b^ | Y ^b^ | Y | Y |
| IP B1h: Manufacturer-assigned Serial Number (see 6.3.10 on page 244) | - | Y | Y | Y | Y |
| BOP caching (see 4.5.2 on page 34) | - | Y | Y | Y | Y |
| READ END OF WRAP POSITION - A3h[1Fh][45h] (see 5.2.20 on page 119) | - | - | Y | Y | Y |
| READ LOGGED-IN HOST TABLE - A3h[1Fh][01h] (see 5.2.21 on page 122) | - | - | Y | Y | Y |
| MP 30h[43h]: Feature switches - Device attribute settings (see 6.6.21.5.3 on page 419) | - | - | Y | Y | Y |
| Logical block protection (see 4.9 on page 43) using the CRC32C (Castagnoli) (see D.2. on page 501) was added. | - | - | Y | Y | Y |
| Many counter sizes were increased. See 6.4--Log Parameters (LP) | - | - | Y | Y | Y |
| LTFS MAM parms MEDIUM GLOBALLY UNIQUE IDENTIFIER {MAM 0820h}: (see 6.5.2.4.12 on page 359) and MEDIA POOL GLOBALLY UNIQUE IDENTIFIER {MAM 0821h} (see 6.5.2.4.13 on page 360) | - | - | Y | Y | Y |
| Inquiry Allocation Length expanded. See 5.2.5--INQUIRY - 12h | - | - | Y | Y | Y |
| Mode Page Behaviors (see 4.7 on page 40) were made consistent and non-standard behaviors better described. | - | - | Y | Y | Y |
| RB 19h: Host non-volatile (see 6.7.2.5 on page 432) | - | - | Y | Y | Y |
| MP 1Ch: Informational Exceptions Control (see 6.6.17 on page 394) behaviors were modified | - | - | Y | Y | Y |
| IP B3h: Automation Device Serial Number (see 6.3.11 on page 245) | - | - | Y | Y | Y |
| IP B4h: Data Transfer Device Element Address (see 6.3.12 on page 245) | - | - | Y | Y | Y |
| SPIN (00h[0002h]) - Security Compliance Information (see 6.8.1.3 on page 446) | - | - | Y | Y | Y |
| SPIN (20h[0031h]) - Device Server Key Wrapping Public Key page (see 6.8.2.9 on page 466) | - | - | Y | Y | Y |
| Archive mode unthread (LTO7+) (see 4.3 on page 29) | - | - | Y | Y ^b^ | Y ^b^ |
| DT device primary port physical interface information (see 6.4.10.1.9 on page 282) | - | - | Y | Y ^b^ | Y ^b^ |
| IP C1h: Drive Serial Numbers (see 6.3.15 on page 250) | - | - | Y | Y ^b^ | Y ^b^ |
| IP C2h: Drive Bar codes (see 6.3.16 on page 251) | - | - | Y | Y ^b^ | Y ^b^ |
| IP C3h: Subcomponent Version List (see 6.3.17) | - | - | Y | Y ^b^ | Y ^b^ |
| LP 0Dh[01h]: Environmental Reporting (LTO9 and later) (see 6.4.9 on page 265) | - | - | - | - | Y |
| GENERATE RECOMMENDED ACCESS ORDER (GRAO) - A4h[1Dh] (LTO9+) (see 5.2.4 on page 82) | - | - | - | - | Y ^a^ |
| RECEIVE RECOMMENDED ACCESS ORDER (RRAO) - A3h[1Dh] (LTO9+) (see 5.2.24 on page 131) | - | - | - | - | Y ^a^ |
| SFP Page A2h log pages (see 6.4.27--LP 39h[02h]: Host Port 0 Physical Interface and 6.4.30--LP 3Bh[02h]: Host Port 1 Physical Interface) | - | - | Y ^b^ | Y ^b^ | Y ^b^ |
| Writing Drive Identifying Information of most recently read data set {LP38h:0100h} on page 340 | - | - | Y ^b^ | Y ^b^ | Y ^b^ |
| LAST FAILED RESERVATION INFORMATION {DRA 0014h} (see 6.2.2.3.7 on page 224) | - | - | Y ^b^ | Y ^b^ | Y ^b^ |
| Application design capacity {LP17h:0018h} on page 304 | - | - | - | Y ^b^ | Y ^b^ |
| Volume Lifetime Remaining {LP17h:0019h} on page 304 | - | - | - | Y ^b^ | Y ^b^ |
| USER DEFINED CARTRIDGE IDENTITY (UDCI) {MAM 1002h} (see 6.5.2.5.3 on page 360) | - | - | - | Y ^b^ | Y ^b^ |
| Firmware Medium Optimization Version {LP14h:F001h} (see page 293) | - | - | - | - | Y ^b^ |

Key:
- `-` Not Supported
- ^a^ Full-Height drive only
- ^b^ Added after GA
