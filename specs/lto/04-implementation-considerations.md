# 4. Implementation Considerations


## 4.1 Media Optimization (LTO9)

Media optimization is a feature of the LTO9 tape drive with L9/LZ media. Media optimization has been
implemented in LTO-9 technology to optimize data placement to each LTO-9 cartridge characteristics. Like other
common storage devices, each new LTO-9 cartridge requires a one-time initialization prior to commencing
read/write operations. LTO-9 media optimization enhances LTO tape long-term media durability. 1
The increased number of tracks used to write data on tape requires greater precision. Media optimization creates
a referenced calibration for each cartridge that enables the tape drive's intelligent alignment to optimize data
placement. LTO-9 media optimization enhances LTO tape long-term media durability.
It is important to consider when media optimization is performed:
- Media optimization is performed on first load of L9/LZ media during initialization.
- Due to environmental requirements, that optimization should be performed in the final installation
  destination where the drives and media are to be used to ensure optimized acclimation. 1
- The media optimization process is only required on the first load of a new and unused LTO-9 cartridge,
  subsequent loads do not require initialization. 1

Other considerations for media optimization:
- Media optimization averages 20 minutes per first load of a cartridge to a tape drive. Although most media
  optimizations will complete within 30 minutes some media optimizations may take up to 2 hours.
- Interruption of the process is not recommended.
- A different mount is unlikely to improve the time to complete this one-time optimization.

A re-optimization may be performed on a cartridge that contains no valid data if, for example, there is a desire to
update the optimization to a newer version of media optimization. The MEDIUM OPTIMIZATION VERSION
{MAM 1011h} (see 6.5.2.5.5 on page 361) MAM attribute contains the medium optimization version number. The
Firmware Medium Optimization Version {LP14h:F001h} (see on page 293) log parameter contains the medium
optimization version that the currently running code uses to optimize new cartridges.
Media re-optimization may be requested by:
- Using the FORMAT MEDIUM command. For details on the FORMAT MEDIUM command options that
  re-optimize a cartridge see 5.2.3--FORMAT MEDIUM - 04h.

> **WARNING** -- Performing a FORMAT MEDIUM destroys all data on the cartridge including user MAM data.

> **WARNING** -- The optimization processing is limited to one time per mount. If it has already occurred on the current mount, then it is not invoked a second time, even if a FORMAT MEDIUM command is processed.

- The MEDIUM OPTIMIZATION NEEDED MAM attribute may be used for test purposes to force a Medium
  Optimization by setting it's value to TRUE.
  On first use of an L9/LZ cartridge that has its MEDIUM OPTIMIZATION NEEDED (ATTRIBUTE IDENTIFIER
  1010h) MAM attribute set to TRUE, the load automatically performs the same media optimization
  processing that is performed on a first use of a cartridge. The MEDIUM OPTIMIZATION NEEDED MAM
  attribute may be set to TRUE by using the WRITE ATTRIBUTE command when the tape is empty
  (see 6.5.2.5.4). Upon successful completion of a media optimization the MEDIUM OPTIMIZATION
  NEEDED MAM attribute is set to FALSE.

1. "LTO-9 Media Initialization", https://www.lto.org/faqs-about-lto/

For best practices, see E.3.--LTO-9 Cartridge Optimization.


## 4.2 Write modes

### 4.2.1 Write mode introduction

Write modes of the device entity specify the allowable behaviors for altering logical objects on a mounted
volume. When the write mode rules allow altering of logical objects then the operation shall be processed
following the write protection rules defined in SSC-4 clause 4.2.14 (i.e., Write Protection).

### 4.2.2 Overwrite-allowed mode

Overwrite-allowed mode is used to allow alteration of any logical object on the medium. Overwrite-allowed mode
is enabled or disabled using the WRITE MODE field of the Device Configuration Extension mode page (see 6.6.12).
This mode is set in the device entity to enable device server behaviors. The overwrite-allowed mode does not
modify the volume. When the volume is removed from the device no indication of whether overwrite-allowed
mode is enabled or disabled is carried with the volume.
When overwrite-allowed mode is enabled in the drive, then any command that would result in a write shall be
processed normally. If the mounted volume is a WORM volume, then a write type command shall be processed
following the WORM rules.

### 4.2.3 Append-only mode (also known as Data-safe mode)

Append-only mode is used to protect data from being accidentally overwritten. Sometimes, due to errors in the
configuration of the environment an application client attempts to rewind a drive that it is not transferring data to.
Without append-only mode, if the application client writing to the logical unit has not reserved the logical unit,
then a different application client is allowed to rewind the volume causing an accidental overwrite of the medium.
With append-only mode enabled, the medium is not allowed to be overwritten.
Append-only mode is enabled or disabled using the WRITE MODE field of the Device Configuration Extension
mode page (see 6.6.12).This mode is set in the device server to enable device server behaviors. The
append-only mode does not modify the volume. When the volume is removed from the device it behaves as a
normal volume.
When append-only mode is enabled in the drive, then any command that would result in a write to a location that
is not an append point shall be rejected with a CHECK CONDITION, DATA PROTECT, OPERATOR SELECTED
WRITE PROTECT (7h / 5A02h) and TapeAlert 09h shall be set. An append point shall be:
- the logical position zero if there are no logical objects beyond BOP;
- the current logical position if:
  - the current logical position is at BOP; and
  - there are only filemarks between the current logical position and EOD;
- the current logical position if:
  - the current logical position is between BOP and EOD;
  - there are only filemarks from the current logical position to EOD; and
  - there is at least one filemark immediately before the current logical position;
  or
- the current logical position if the current logical position is at EOD.

The device server maintains an allow_overwrite variable. The allow_overwrite variable defines what operation is
currently allowed when in append-only mode. The allow_overwrite variable values are defined in table 6.

| Name | Description |
|------|-------------|
| Disabled | A write type operation at a position that is not an append point is not allowed. |
| Current Position | A write type operation is allowed at the position specified by the allow_overwrite_position variable. |
| Format | An operation that modifies the format of the medium is allowed |

The allow_overwrite_position variable specifies the position (i.e., partition and logical object identifier) at which a
write to a position that is not an append point is allowed.
Append-only mode is a function of the device server and is not a function of the volume. Append-only mode may
be used when accessing Data Volumes or WORM volumes. An application client may overwrite data by using a
special command called the ALLOW OVERWRITE command (see 5.2.1). The ALLOW OVERWRITE command
specifies the logical position where the overwrite is to occur. After succesfully processing an ALLOW
OVERWRITE command, a write type command at the specified position is processed normally. If the position of
the medium is changed or the volume is unmounted, then the device server shall set the allow_overwrite variable
to Disabled (i.e., 0h) and the allow_overwrite_position variable to invalid. The ALLOW OVERWRITE command
requires the partition number and the logical position to be passed in the CDB. If the position information passed
in the ALLOW OVERWRITE command does not specify the current position of the medium, then the command is
terminated with CHECK CONDITION status with the sense key set to ILLEGAL REQUEST and the additional
sense code set to SEQUENTIAL POSITIONING ERROR. If there is no volume loaded and the device server
processes an ALLOW OVERWRITE command, then the command is terminated with CHECK CONDITION
status with sense key set to NOT READY.
An ALLOW OVERWRITE command that returns GOOD status shall:
- set the allow_overwrite variable to the value in the ALLOW OVERWRITE field of the ALLOW OVERWRITE
  command; and
- set the allow_overwrite_position variable to the current position.

An ALLOW OVERWRITE command that returns a CHECK CONDITION shall:
- set the allow_overwrite variable to Disabled (i.e., 0h); and
- set the allow_overwrite_position to invalid.

If append-only mode is enabled, the mounted volume is a WORM volume, and the allow_ovewrite variable is not
set to Disabled, then a write type command shall be processed following the WORM rules. Figure 2 shows a
representative flowchart of append-only mode behavior.

*Figure 2 -- Append-only mode flowchart*

If the ALLOW OVERWRITE command is received by the device server and append-only mode is not enabled,
the command is rejected with CHECK CONDITION, ILLEGAL REQUEST, ILLEGAL COMMAND WHEN NOT IN
APPEND-ONLY MODE.
When in append-only mode the allow_overwrite variable shall be set to Disabled (i.e., 0h) and the
allow_overwrite_position variable shall be set to invalid if:
- the WRITE MODE field of the Device Configuration Extension mode page changes to a value of 01h (i.e.,
  the write-type operation only allows appends as specified for the append-only mode in 4.2.3);
- a change in logical position occurs;
- a volume changes state from unmounted to mounted;
- the CDB of a write type command is validated and the write processing begins; or
- an ALLOW OVERWRITE command returns a CHECK CONDITION.


## 4.3 Archive mode unthread (LTO7+)

LTO-7 and LTO-8 drives support choosing a trade-off between fast unthread times without optimizing the
preparation of the medium for long term storage and slower unthread times that optimize the preparation of the
medium for long term storage. This trade-off is selected through the Archive mode unthread feature. LTO-9 and
later drives always optimize the preparation of the medium for long term storage.
Archive mode unthread is able to be invoked either by using the RETEN bit of the LOAD/UNLOAD command
(see 5.2.6) or by configuring the drive to use the Archive mode unthread for every unload that occurs. The drive
is configured to use the Archive mode unthread for every unload that occurs by setting the E_ARCHIVE bit to one
in the MP 30h[43h]: Feature switches - Device attribute settings (see 6.6.21.5.3) mode page. This mode
parameter has a mode parameter policy of (changeable-saveable) and may be saved by setting the SP (save
pages) bit to one in the MODE SELECT command. LTO-9 and later drives always behave as if the E_ARCHIVE bit
is set to one (i.e., ignore the bit).

> **WARNING** -- Archive mode unthread should be used for volumes that may be stored for extended periods of time.


## 4.4 Volume partitioning

### 4.4.1 Volume partitioning overview

Starting with LTO 5 volume partitioning is supported by the device on certain media types. A volume is recorded
in the same format for the entire volume as indicated by the primary density code (see 5.2.24) but each partition
may have differences in how it is encrypted.
The device supports wrap-wise partitioning (see 4.4.2).
This clause and its subclauses describe partitioning and its relationship to:
- capacity scaling (see 4.4.3);
- media types (see 4.4.4);
- reformatting (see 4.4.5); and
- encryption (see 4.4.6).

### 4.4.2 Wrap-wise Partitioning

Wrap-wise partitioning uses the length of the medium that is available for user data to create each partition. By
default, this is the full length of the medium but the length available for user data may have been shortened
through capacity scaling (see 4.4.3). Wrap-wise partitioning is shown logically in figure 3.

```
Partition 0
Guard wraps
Partition 1
Partition 2
Partition 3
```

*Figure 3 -- Wrap-wise partitioning*

Wrap-wise partitioning uses a minimum of two wraps and has guard wraps of two wraps or four wraps between
each partition depending on generation and number of partitions. For Ultrium 9+ cartridges with an even number
of equally sized partitions using SDP, the purpose of the guard wraps is fulfilled by the band boundaries thereby
not sacrificing capacity for guard wrap overhead. The amount of usable capacity may be reduced depending on
various factors including volume generation (e.g., up to 2.5% per partition boundary for Ultrium 5 volumes).

When using wrap-wise partitioning an Ultrium 5 volume supports one or two partitions and an Ultrium 6 or later
generation volume supports a maximum of four partitions with any number of partitions between one and four
inclusive.
Table 7, and table 8 show the partition sizes that result from a MODE SELECT of the Medium partition mode
page (see 6.6.13) with the indicated field settings.

```
Table 7 -- Partition sizes for wrap-wise partitioning (selection fields)

                                       ADDITIONAL                              PARTITION SIZE
     Ref a       FDP    SDP    IDP     PARTITIONS
                                        DEFINED
                                                b        (first)       (second)           (third)      (last)

      row1        1      0      0           X
      row2                                 00h
      row3                                 01h             X               X                X            X
                  0      1      0
      row4                                 02h
      row5                                 03h
      row6                                               FFFFh
                                           00h                             0
      row7                                                 sc
      row8                                                  s           FFFFh               0
      row9                                 01h           FFFFh             t
      row10                                                sc             tc                             0
      row11                                                 s              t              FFFFh
      row12                                                 s           FFFFh               u
                  0      0      1          02h
      row13                                              FFFFh             t                u
      row14                                                sc             tc                uc
      row15                                                 s              t                u         FFFFh
      row16                                                 s              t              FFFFh          v
      row17                                03h              s           FFFFh               u            v
      row18                                              FFFFh             t                u            v
      row19                                                sc             tc                uc          vc
      others                                         All other combinations
 a
     The Ref column is the reference that ties the rows in this table to the corresponding rows in table 8.
 b
     When more than one partition is defined there may be overhead that results in a loss of capacity
 c   Value must be exact partition size allowed and when summed with other values in the row equal full capacity.
     This permits a MODE SENSE followed by a MODE SELECT with no change. It is highly recommended that this
     method only be used in the case where the MODE SELECT data is a return of the MODE SENSE data.
```

```
Table 8 -- Partition sizes for wrap-wise partitioning (resultant sizes) (part 1 of 2)

      Ref a           Partition 0 b            Partition 1 b              Partition 2 b             Partition 3 b
                        s=K*n;
       row1                                         t=K                         -                           -
                      where n=N2
                                          Ultrium 5 through Ultrium 8 Cartridges
       row2               CMAX                                                  -
                        s=K*n;
                                                 t=K*m;
       row3        where n=integer of                                                        -
                                             where m=N2+1-n
                      {(N2+1)/2}
                        s=K*n;                    t=K*m;
                                                                           s=K*u;
       row4        where n=integer of       where m=integer of                                              -
                                                                      where u=N3+2-n-m
                      {(N3+2)/3}                {(N3+2)/3}
                        s=K*n;                    t=K*m;                    s=K*u;
                                                                                                       s=K*v;
       row5        where n=integer of       where m=integer of         where u=integer of
                                                                                                 where v=N4+3-n-m-u
                      {(N4+3)/4}                {(N4+3)/4}                {(N4+3)/4}
                                                    Ultrium 9 Cartridge
       row2               CMAX                                                  -
       row3            s=CMAX / 2                t=CMAX / 2                                  -
                                          t=K*ROUNDDOWN(m/2)             u=K*((N3+2) -
                  s=K*ROUNDDOWN(n/3)
       row4                                 where m = (N3+2) -         ROUNDDOWN(n/3) -                     -
                    where n = (N3+2)
                                             ROUNDDOWN(n/3)            ROUNDDOWN(m/2))

       row5            s=CMAX / 4                t=CMAX / 4                u=CMAX / 4                v=CMAX / 4
       row6               CMAX
                                                                                    -
       row7               CMAX
                        s=K*n;               CMAX - overhead-
       row8
                    where 1<=n<=N2           (partition size 0)
                    CMAX - overhead-             t=K*m;
       row9
                    (partition size 1)       where 1<=m<=N2                                  -
                         s=K*n;                  t=K*m;
       row10      where 1<=n<=N2 and      where 1<=m<=N2 and
                      n+m=N2+1                 n+m=N2+1
                                                                       CMAX - overhead-
                                                 t=K*m;
       row11                                                           (partition size 0)-
                                             where 1<=m<=N3
                        s=K*n;                                          (partition size 1)
                    where 1<=n<=N3           CMAX - overhead-
       row12                                 (partition size 0)-
                                             (partition size 2)            u=K*p;
                                                                                                                -
                    CMAX - overhead-                                   where 1<=p<=N3
                                                 t=K*m;
       row13        (partition size 1)-
                                             where 1<=m<=N3
                     (partition size 2)
                         s=K*n;                  t=K*m;                    u=K*p;
       row14      where 1<=n<=N3 and      where 1<=m<=N3 and         where 1<=p<=N3 and
                     n+m+p=N3+2              n+m+p=N3+2                 n+m+p=N3+2
  a      The values in the Ref column refer back to the associated row in table 7.
  b
         The values for CMAX, K, N2, N3, N4, and the sum of existing partitions are specified in table 9.
```

```
Table 8 -- Partition sizes for wrap-wise partitioning (resultant sizes) (part 2 of 2)

         Ref a                 Partition 0 b               Partition 1 b             Partition 2 b           Partition 3 b
                                                                                                           CMAX - overhead-
                                                                                       u=K*p;              (partition size 0)-
          row15
                                                                                   where 1<=p<=N4          (partition size 1)-
                                                           t=K*m;                                           (partition size 2)
                                                       where 1<=m<=N4              CMAX - overhead-
                               s=K*n;                                              (partition size 0)-
          row16
                           where 1<=n<=N4                                          (partition size 1)-
                                                                                    (partition size 3)
                                                        CMAX - overhead-
                                                        (partition size 0)-                                    v=K*q;
          row17
                                                        (partition size 2)-                                where 1<=q<=N4
                                                        (partition size 3)             u=K*p;
                             CMAX - overhead-                                      where 1<=p<=N4
                             (partition size 1)-           t=K*m;
          row18
                             (partition size 2)-       where 1<=m<=N4
                              (partition size 3)
                            s=K*n;                          t=K*m;                     u=K*p;                   v=K*q;
          row19      where 1<=n<=N4 and              where 1<=m<=N4 and          where 1<=p<=N4 and      where 1<=q<=N4 and
                       n+m+p+q=N4+3                    n+m+p+q=N4+3                n+m+p+q=N4+3            n+m+p+q=N4+3
          others                            Check Condition, Illegal Request, Invalid Field in Parameter Data
     a
            The values in the Ref column refer back to the associated row in table 7.
     b
            The values for CMAX, K, N2, N3, N4, and the sum of existing partitions are specified in table 9.
```

```
Table 9 -- Partition values for L5, L6, and L7

                                                                              Primary Density Code
      Parameter in table 8
                                                     58h                           5Ah                           5Ch
                         b
                  CMAX                              1.5 TB                        2.5 TB                        6.0 TB
                  K a, b                           37.500 GB                    36.764 GB                   107.142 GB
                   N2                                 38                            66                            54
                   N3                                N/A                            64                            52
                   N4                                N/A                            62                            50
                                                                                   s+t=2 463.235 GB            s+t=5 892.857 GB
     Sum of all partitions a, b
                                               s+t=1 462.500 GB                 s+t+u=2 426.470 GB          s+t+u=5 785.714 GB
          s+t+[u]+[v]
                                                                              s+t+u+v=2 389.705 GB        s+t+u+v=5 678.571 GB
 a
          The precision of capacity values able to be expressed is limited by the 2-byte PARTITION SIZE field and the value
          of the PARTITION UNITS field of the Medium Partition mode page (see 6.6.13). Actual size on medium is not
          limited by the precision of the fields in the mode page.
 b
          The capacity values assume a volume that has not been capacity scaled (see 4.4.3).
```

```
Table 10 -- Partition values for M8, L8, and L9

                                                                   Primary Density Code
       Parameter in table 8
                                                  5Dh                       5Eh                       60h
               CMAX   b                          9.0 TB c                 12.0 TB c                 18.0 TB c
               K a, b                         107.142 GB c              115.384 GB c              128.571 GB c
                 N2                                82                       102                       137
                 N3                                80                       100                       134
                 N4                                78                        98                       131
                                                                                             s+t=17 742.660 GB c
                                                                                          s+t+u=17   485.656 GB c
                                              s+t=8 892.857 GB c    s+t=11 884.615 GB c
      Sum of all partitions a, b                                                        s+t+u+v=17 228.514 GB c
                                           s+t+u=8 785.714 GB c s+t+u=11 769.230 GB c
           s+t+[u]+[v]                                                                          if SDP=1b
                                         s+t+u+v=8 678.571 GB c s+t+u+v=11 653.846 GB c
                                                                                             s+t=18 000.000 GB c
                                                                                        s+t+u+v=18 000.000 GB c
                                         K = 100.000 GB           K = 111.538 GB            K = 124.284 GB
                                         Partitions:              Partitions:               Partitions:
                                         count total size         count total size           count total size
                                           2       8 300.000 GB     2       11 488.461 GB       2     17 151.330 GB
                                           3       8 200.000 GB     3       11 376.923 GB       3     16 902.857 GB
        Design capacity c b                4       8 100.000 GB     4       11 265.384 GB       4     16 654.190 GB
                                                                                            if SDP was 1b during
                                                                                            partition creation
                                                                                                2     17 400.000 GB
                                                                                                3     16 902.857 GB
                                                                                                4     17 400.000 GB
 a
     The precision of capacity values able to be expressed is limited by the 2-byte PARTITION SIZE field and the value
     of the PARTITION UNITS field of the Medium Partition mode page (see 6.6.13). Actual size on medium is not
     limited by the precision of the fields in the mode page.
 b
     The capacity values assume a volume that has not been capacity scaled (see 4.4.3).
 c
     See 3.4--Supported Tape Cartridges
```

### 4.4.3 Partitioning and capacity scaling

Partitioning of volumes is supported on capacity scaled volumes. If a volume has been capacity scaled through
the use of the SET CAPACITY command (see 5.2.37), the medium available for use to record user data is
reduced and partitioning functions within those bounds. The act of processing a SET CAPACITY command
removes any partitions that may exist and changes the medium available for use to record user data. The
resultant volume contains a single partition which may subsequently be partitioned using the Medium partition
mode page (see 6.6.13) and the FORMAT MEDIUM command (see 5.2.3).

### 4.4.4 Partitioning and media types

Partitioning of volumes is supported on media in Ultrium 5 (i.e., primary density code = 58h) and and later logical
formats only.

### 4.4.5 Partitioning and reformatting

Partitions are created and destroyed using the FORMAT MEDIUM command (see 5.2.3). How a volume is
formatted depends on the settings in the Medium Partition mode page (see 6.6.13), if the volume is capacity
scaled (see 4.4.3), and the settings in the FORMAT MEDIUM command (see 5.2.3). The FORMAT MEDIUM
command specifies how to format the volume and the interactions of these conditions using the FORMAT field.

The Medium Partition mode page is used to specify the group of medium partitions. The partitioning of the
mounted volume is not changed until a subsequent FORMAT MEDIUM command is issued while the volume is
mounted.
The device ensures consistency of the partitioning values set in Medium Partition mode page by causing a
CHECK CONDITION status with the sense key set to ILLEGAL REQUEST and the additional sense code set to
PARAMETER VALUE INVALID to be returned to a subsequent FORMAT MEDIUM command attempting to use
the values set in Medium Partition mode page if values in those values become inconsistent between when they
were set and when the FORMAT MEDIUM command is received. The invalidation of the values in this page is
accomplished by setting the FDP, SDP, and IDP bits to zero and the other values in this page to:
- the values dictated by the format of the currently mounted volume, if a volume is mounted; or
- the default values present after power-on, if no volume is mounted.

The invalidation of values in this page occurs if:
- the volume is capacity scaled (see 4.4.3); or
- other events that are determined to make the values in this page inconsistent.

### 4.4.6 Partitioning and encryption

The relationship between partitioning and encryption is described in Device Hardware Encryption (see 4.15 on
page 55).


## 4.5 Object buffer

### 4.5.1 Object buffer introduction

This device contains an object buffer capable of holding logical objects being written to the medium or logical
objects being transferred from the medium in read-ahead operations. The object buffer is used during write
operations when the device is configured to use buffered mode (i.e., the BUFFER MODE field of the mode
parameter header is set to a non-zero value per Mode Parameter Header for Mode Select (6/10) (see 6.6.1.1 on
page 363))) and during read operations regardless of the buffer mode.
When the device is reading logical objects from the medium, it uses the object buffer in a read-ahead fashion to
improve performance. Logical objects are read from the medium and placed into the object buffer such that they
are available to an application that is reading without the application being required to wait for each block to be
read from the medium prior to being transferred on the SCSI interface. Read-ahead operations often occur at the
conclusion of space, locate, and load operations in order to prime the object buffer with logical objects in case a
read operation follows.

### 4.5.2 BOP caching

Devices starting with LTO6 use a small portion of the object buffer as a cache to retain data read at BOP while
the remainder of the object buffer is used for read-ahead operations. The data around BOP, once read, is
generally retained in the BOP cache until a demount or partition change. If D_BOPC of the MP 30h[43h]: Feature
switches - Device attribute settings (see 6.6.21.5.3 on page 419) mode page is set to 0b and a command is
received by the device that requests access to a logical object identifier (aka, logical block address) that is in the
BOP cache, that data is read from the BOP cache without requiring actual access to the medium (i.e., it uses the
cached data without changing the physical location of the medium). This allows for a volume that is located away
from BOP to read the data around BOP very quickly without disturbing the current physical position of the
medium. If D_BOPC of the MP 30h[43h]: Feature switches - Device attribute settings mode page is set to 1b, then
a request to perform positioning changes the physical location of the medium and performs a read-ahead
operation as appropriate.

#### 4.5.2.1 BOP caching side effects

It is important to understand the side effects that BOP caching may present:
- Processing time to position near BOP may be transferred from the positioning command to a subsequent command (e.g., the time for a REWIND could be transferred to a subsequent UNLOAD);
- When reading data in the BOP cache and the command requests a read through the cache boundary to
  data not in the cache, there may be processing time to position the medium to the proper position to read
  the data, as well as the time to read the data from the medium;
- If a sequence of commands like:
  1. REWIND;
  2. READ one block;
  3. LOCATE to position prior to the REWIND;
  4. READ one block;
  5. Go to step 1,

  is performed, the tape will typically move in a sequential fashion like a READ without positioning to BOP
  in each iteration.


## 4.6 Recommended access order (RAO) (LTO-9+ Full-Height)

### 4.6.1 RAO Suitability

The RAO implementation in LTO produces the best results for performance enhancement when there is little
variability in block size or data compression ratio. When the variability in compression ratio or block sizes
increase, the accuracy of the locate estimates may be reduced and any potential performance enhancements
may be diminished.

### 4.6.2 RAO overview

A feature of the LTO-9 and later full-height drives is the ability to accept a list of User Data Segments
(see 4.6.2.1) and reorder those User Data Segments into a recommended access order that minimizes the
locate portion of the time to read those User Data Segments. This sorted list is called a Recommended Access
Order (RAO) list. A User Data Segment (UDS) is defined as a grouping of contiguous logical objects (i.e., logical
blocks and filemarks) and is described by partition number, beginning logical object identifier, and ending logical
object identifier.
An additional capability to optimize this list to minimize the time for a subsequent unload, as well as the capability
to bind to a specific starting point or ending point is included (see 4.6.3.4).

#### 4.6.2.1 User data segments (UDS) in a partition

Within a partition that has recorded logical objects a contiguous sequence of logical blocks or logical files may be
referenced as a user data segment (UDS).

#### 4.6.2.2 User data segment descriptors

UDS descriptors (see 5.2.24.1.3) are used to describe attributes of the UDS and contain the following:
- an application client specified name;
- a partition number;
- a beginning logical object identifier;
- an ending logical object identifier;
- in a returned RAO list, an estimate of the time to locate from the end of the current UDS to the beginning
  of the next UDS to access. This does not include the time required to read the UDS as this has variability
  dictated by the application, the load on the server, and other unknown factors; and
- optionally, in a returned RAO list, the physical geometry of the UDS if requested in the GRAO command.

### 4.6.3 RAO features

#### 4.6.3.1 RAO features overview

The drive accepts a list of User Data Segments (see 4.6.2) in a GRAO parameter list and may reorder those
UDSes into a recommended access order that reduces the time to process the list.
The list of UDSes is sent to the drive using the Generate Recommended Access Order (GRAO) command
(see 5.2.4). The drive creates the RAO list from the list of UDSes and optionally sorts the list of UDSes. The RAO
list may then be read with one or more Receive Recommended Access Order (RRAO) commands (see 5.2.24).
The RRAO command allows retrieval of the ROA list with or without geometry information.
The RAO feature allows the user to:
- determine the UDS limits for the type of RAO list to generate;
- specify the process to use in generating the RAO list; and
- specify binding points in the RAO list.

#### 4.6.3.2 Determining the UDS limits

RAO supports thousands of User Data Segments in the RAO list. The RRAO command (see 5.2.24) with the
UDS_LIMITS bit set to one may be used to determine the number of supported UDSes for the type of RAO list to
generate (i.e., for the specific setting of the UDS_TYPE field) as well as the maximum size of each UDS for the
type of RAO list to generate. At the time this document was published, the maximum number of supported
UDSes for all values of UDS_TYPE is 2730.

#### 4.6.3.3 Specifying the process for generating the RAO list

The GRAO command specifies the process to use in generating the RAO list as defined in table 11

| Value | Description |
|-------|-------------|
| 000b | Not supported |
| 001b | Drive does not reorder the UDSes passed in the GRAO parameter list, but does calculate the time to sequentially locate to each UDS in the list from the end of the prior position. |
| 010b | Drive reorders the UDSes passed in the GRAO parameter list into the recommended access order and calculates the time to sequentially locate to each UDS in its resultant position from the end of the prior position. |
| 011b-111b | Reserved |

#### 4.6.3.4 Specifying binding points in the RAO list

The binding points are limited to:
- the starting point of the sort;
- the ending point of the sort; and
- the unloaded position.

Binding points are specified by placing single-object UDSes in specific positions in the list. A single-object UDS is
one where the BEGINNING LOGICAL OBJECT IDENTIFIER field is identical to the ENDING LOGICAL OBJECT IDENTIFIER
field. The Starting Point UDS must be the first single-object UDS in the list. The Unloaded Position UDS is a
single-object UDS with the PARTITION NUMBER field, the BEGINNING LOGICAL OBJECT IDENTIFIER field, and the
ENDING LOGICAL OBJECT IDENTIFIER field set to zero and must be the last UDS in the list. The Ending Point UDS
must be either the last UDS in the list or immediately precede the Unloaded Position UDS. Other single-object
UDSes are not binding points and are optimally sorted.
The RAO list contains all the UDSes sent in the GRAO parameter list including the binding points. The positions
of the binding point UDSes are unchanged, but the remaining UDSes, including any single-object UDSes that are
not binding points are optionally sorted. The ESTIMATED LOCATE TIME TO UDS field in the Unloaded Position UDS is
the estimated time for the cartridge to be unloaded to the ejected position (i.e., fully unloaded).
Specifying both an Ending Point UDS and an Unload Position UDS is logically contradictory and is not
recommended. The ending point precludes any sort optimization indicated by the unload position.

### 4.6.4 RAO usage

The RRAO command returns the RAO list generated in the last successful GRAO command. The RAO list that is
generated is valid for the state of the currently mounted volume (i.e., logical position, logical objects on media,
etc.) at the time the list is generated. If the logical position of the medium is changed, or if logical objects are
written or erased, then the RAO list becomes out of date. However, the device server takes no action to
invalidate the list or to enforce a specific sequence of operation before returning an RAO list. Therefore, the
responsibility of ensuring the RAO list has not been invalidated by commands since the processing of the GRAO
command rests with the application.
An example of how an application client may use the recommended access order model (see 4.6.1) is to:
1. Read the UDS limits (see 5.2.24.1.1) to determine the number of supported UDSes (see 4.6.2.1);
2. Compose a list of UDSes to be accessed;
3. Generate an RAO list (see 5.2.24.1.2) from the list of UDSes to be accessed using the GRAO command
   (see 5.2.4);
4. Read a portion of the RAO list using the RRAO command (see 5.2.24) with the RAO LIST OFFSET field set
   to zero and the ALLOCATION LENGTH field set as appropriate for the Data-In Buffer;
5. Check the RAO PROCESS field and the STATUS field of the RAO list (see 5.2.24.1.2) to confirm that the
   RAO list was generated as expected;
6. For all user data segment descriptors returned in this portion of the RRAO list (in order):
   - ignore a binding UDS;
   - locate to the BEGINNING LOGICAL OBJECT IDENTIFIER; and
   - read to the ENDING LOGICAL OBJECT IDENTIFIER;
7. If the value in the RAO list RAO DESCRIPTOR LIST LENGTH field returned in step 4) is larger than the sum of
   the value in the RAO LIST OFFSET field and the size of the portion of the RAO list returned in response to
   the RAO command, then read another portion of the RAO list using the RRAO command with the RAO
   LIST OFFSET field and the ALLOCATION LENGTH field set as appropriate for the Data-In Buffer;
8. Repeat steps 4) through 7) as necessary until all UDSes have been read; and
9. Unload, if desired.

Two examples--simplified to show only 8 wraps per data band for a total of 32 wraps--of a recall of ten UDSes
are shown for a comparison of potential time savings by using RAO. They are:
- by Logical Object Identifier (LOI) sort order (see figure 4); and
- by RAO with Unload sort order (i.e., GRAO with the RAO process set to 010b order with an Unload
  Position UDS, see figure 5).

*Figure 4 -- Example Logical Object Identifier (LOI) sort order*

*Figure 5 -- Example RAO with Unload sort order*

#### 4.6.4.1 User data segment geometry usage

If the RAO list generated contains UDSes with geometry (see 5.2.24), then the geometry descriptors
(see 5.2.24.1.3.1) may be used to build a representation of the physical layout of the UDSes on tape. This may
be useful for visual feedback or for an application to create its' own algorithm for UDS retrieval based on physical
location.


## 4.7 Mode Page Behaviors

### 4.7.1 Mode Page Policy -- non-standard

This device implements a non-standard behavior related to mode page policy. The mode page policies defined
are:

| Mode page policy | Number of mode page copies |
|------------------|---------------------------|
| \<Shared\> | One copy of the mode page that is shared by all I_T nexuses. |
| \<Per target port\> | A separate copy of the mode page for each target port with each copy shared by all initiator ports. |
| \<Per I_T nexus\> | A separate copy of the mode page for each I_T nexus |

#### 4.7.1.1 Mode parameter header and block descriptor policy

This device implements mode page policy in a manner different than specified in T10/SPC-4. The mode page
policy for mode parameter header and block descriptor values depends on the specific parameter as shown here
for the applicable parameters:

| Field | Mode page policy |
|-------|-----------------|
| BUFFERED MODE | \<Shared\> |
| SPEED | \<Shared\> |
| BLOCK LENGTH | \<Per I_T nexus\> |

#### 4.7.1.2 Mode page policy

The mode page policy implemented by this device is shown in table 12

```
Table 12 -- Mode page policy (part 1 of 2)

                                                                                                               Returned
                                                                              a     Mode page
                           Mode Page                                   MLUS                         IP 87h b      in
                                                                                      policy
                                                                                                               MP 3Fh c
Default mode page policy descriptor returned in IP 87h: Mode Page Policy
                                                                         -           <Shared>          Y          -
(see 6.3.6 on page 238) as MP 3Fh[FFh]
MP 01h: Read-Write Error Recovery (see 6.6.5 on page 370)                    -           <Shared>          -          Y
MP 02h: Disconnect-Reconnect (see 6.6.6 on page 371)                         Y        <Per I_T nexus>      Y          Y
MP 0Ah: Control (see 6.6.7 on page 373)                                      -           <Shared>          -          Y
MP 0Ah[01h]: Control Extension (see 6.6.8 on page 374)                       Y           <Shared>          Y          Y
MP 0Ah[F0h]: Control Data Protection (see 6.6.9 on page 375)                 -        <Per I_T nexus>      Y          Y
MP 0Fh: Data Compression (see 6.6.10 on page 377)                            -           <Shared>          -          Y
MP 10h: Device Configuration (see 6.6.11 on page 379)                        -           <Shared>          -          Y
MP 10h[01h]: Device Configuration Extension (see 6.6.12 on page 381)         -           <Shared>          -          Y
MP 11h: Medium Partition Page (see 6.6.13 on page 384)                       -           <Shared>          -          Y
Key:
       - No
       Y Yes
 a
       The MLUS (multiple logical units share) indicates if this mode page--subpage combination may be shared
       by other logical units (e.g., The FCP port (19h) page controls port related functions)
 b
       A mode page policy descriptor other than the default mode page policy descriptor is returned for this page
       in IP 87h: Mode Page Policy (see 6.3.6 on page 238).
 c
       Whether on not the mode page is returned in mode page 3Fh or mode page 3Fh[FFh] is indicated in this
       column. Some vendor-specific pages are not returned with an all pages request.
```

```
Table 12 -- Mode page policy (part 2 of 2)

                                                                                                                        Returned
                                                                                         a     Mode page
                                 Mode Page                                        MLUS                           IP 87h b      in
                                                                                                 policy
                                                                                                                            MP 3Fh c
MP 18h: Fibre Channel Logical Unit (see 6.6.14.1 on page 389)                       Y        <Per I_T nexus>        Y          Y
MP 18h: SAS Logical Unit (see 6.6.14.2 on page 390)                                 Y        <Per I_T nexus>        Y          Y
MP 19h: FCP port (see 6.6.15.1 on page 391)                                         Y        <Per target port>      Y          Y
MP 19h: SAS port (see 6.6.15.2 on page 392)                                         Y        <Per target port>      Y          Y
MP 1Ah: Power Condition (see 6.6.16 on page 393)                                    -           <Shared>            -          Y
MP 1Ch: Informational Exceptions Control (see 6.6.17 on page 394)                   -        <Per I_T nexus>        Y          Y
MP 1Dh: Medium Configuration (see 6.6.18 on page 397)                               -           <Shared>            -          Y
MP 24h: Vendor-Specific (see 6.6.19 on page 398)                                    -           <Shared>            -          -
MP 2Fh: Behavior Configuration (see 6.6.20 on page 401)                             -           <Shared>            -          Y
MP 30h: Device Attribute Settings (see 6.6.21 on page 404)                          Y           <Shared>            Y          Y
MP 30h[01h]: Drive MAC address - Device attribute settings
                                                                                    Y           <Shared>            Y          Y
(see 6.6.21.3.2 on page 409)
MP 30h[02h]: Drive IP address and subnet mask - Device attribute settings
                                                                                    Y           <Shared>            Y          Y
(see 6.6.21.3.3 on page 411)
MP 30h[20h]: Encryption mode - Device Attribute Settings
                                                                                    Y           <Shared>            Y          Y
(see 6.6.21.4.1 on page 413)
MP 30h[40h]: SkipSync - Device attribute settings
                                                                                    Y           <Shared>            Y          Y
(see 6.6.21.5.1 on page 415)
MP 30h[42h]: End of partition behavior control - Device attribute settings
                                                                                    Y           <Shared>            Y          Y
(see 6.6.21.5.2 on page 418)
MP 30h[43h]: Feature switches - Device attribute settings
                                                                                    Y           <Shared>            Y          Y
(see 6.6.21.5.3 on page 419)
MP 3Eh: Engineering Support (see 6.6.22 on page 423)                                -           <Shared>            -
Key:
       - No
       Y Yes
 a
       The MLUS (multiple logical units share) indicates if this mode page--subpage combination may be shared
       by other logical units (e.g., The FCP port (19h) page controls port related functions)
 b
       A mode page policy descriptor other than the default mode page policy descriptor is returned for this page
       in IP 87h: Mode Page Policy (see 6.3.6 on page 238).
 c
       Whether on not the mode page is returned in mode page 3Fh or mode page 3Fh[FFh] is indicated in this
       column. Some vendor-specific pages are not returned with an all pages request.
```

### 4.7.2 Classification of mode parameters

The page control (PC) field of the MODE SENSE command indicates four classifications of mode pages:

| Value | Description |
|-------|-------------|
| 00b | Current values |
| 01b | Changeable values |
| 10b | Default values |
| 11b | Saved values. |

This device has the following behaviors for mode parameters:

```
Table 13 -- Mode parameter change behavior

                     Values reported for Mode Sense
                                                     Action when a value of a field recieved is different than the
       Term          with page control of Changeable
                                                                      Current values (00b) a
                               values (01b)
    (changeable)                                          The current value is updated.
                                                         See the description of the parameter to determine the action
                                                         (e.g., the parameter may be writeable and change the
  (changeable-special)
                   The bits of this field are set to one behavior to that indicated by the received value, but not update
                   in the parameter data returned to a the Current values).
                   MODE SENSE command with the The current value is updated.
                   PC field set to 01b (i.e.,            If the SP bit in the MODE SELECT CDB is set to one, then the
 (changeable-saveable) Changeable values).
                                                         value for the Saved values (11b) for this page is updated and
                                                         saved to non-volatile memory before SCSI status is returned.
                                                          The current value is unchanged.
  (changeable-ignored)
                                                          No action is taken.
                     The bits of this field are set to zero
                     in the parameter data returned to a The MODE SELECT command is rejected with a 5/2600h (i..e,
      (non-changeable)
                     MODE SENSE with a PC field set ILLEGAL REQUEST, INVALID FIELD IN PARAMETER LIST).
                     to 01b (i.e., Changeable values).
  a      A value in the mode parameter data received with a MODE SELECT command is different than the value
         in the mode parameter data returned to a MODE SENSE command with the PC field set to 00b (i.e.,
         Current values).
```

This device implements the following features differently than specified in SPC-4:
- Save behavior -- non-standard (see 4.7.2.1 on page 42); and
- Parameter Saveable behavior -- non-standard (see 4.7.2.2 on page 42).

#### 4.7.2.1 Save behavior -- non-standard

This device implements mode parameter saving in a manner different than specified in SPC-4. The SP bit of the
MODE SELECT command (see 5.2.10) applies only to the parameters sent in parameter data to that MODE
SELECT command. No other mode parameters' Current values are saved. This is contrary to SPC-4 which
mandates that the Current values of all saveable mode pages be saved if the SP bit is set to one.

#### 4.7.2.2 Parameter Saveable behavior -- non-standard

The parameter saveable (PS) bit in the mode parameters is set to one in the parameter data returned to a MODE
SENSE if at least one mode parameter in the page is saveable. Since only some parameters are saveable and
others are not, it may be possible that some of the changeable parameters in the page are saveable and other
changeable parameters in the page are not. There is no programmatic method for retrieving a list of which
specific mode parameters are saveable.
The parameter saveable (PS) bit in the mode parameters is ignored during the processing of a MODE SELECT
command.

### 4.7.3 Mode parameters and unit attentions

Some mode parameters, including mode parameters in the mode parameter header, in the block descriptor, and
in some mode pages are affected by mounting a volume. When this occurs, there is no unit attention for MODE
PARAMETERS CHANGED (i.e., 6/2A01h) established.


## 4.8 Programmable early warning

When writing, the application client may need an indication prior to early warning to allow for the application client
to prepare to be ready for early warning (e.g., flush buffers in the application client).
Application clients that need this indication may request the device server to create a zone called the
programmable-early-warning zone (PEWZ) by setting the PEWS field (see 6.6.8) to the requested size of the
PEWZ. The EOP side of PEWZ is established at early-warning and extends towards BOP for a distance
indicated by the PEWS field. See figure 6.

```
                                                                                              LEOP
          BOP                                                                    EW
                   Logical objects accessible by the device server   PEWZ                      EOP

                                                                     Maximum logical object identifier
```

*Figure 6 -- Programmable early warning example*

> **WARNING** -- If PEWZ is used, all applications that may access the drive when a PEWZ exists, should support PEWZ or there is a risk of the application that does not support PEWZ detecting an unknown error or a diminished capacity when the PROGRAMMABLE EARLY WARNING error is reported.

The REW bit in the Device Configuration mode page (see 6.6.11) shall have no effect on the device server
behavior in the PEWZ.
The device server shall return CHECK CONDITION status, with the sense key set to NO SENSE, the EOM bit
set to one and the additional sense code set to PROGRAMMABLE EARLY WARNING DETECTED at the
completion of a command that caused the medium to transition into the PEWZ if that command is:
- WRITE(6); or
- WRITE FILEMARKS(6).

Encountering the PEWZ shall not cause the device server to perform a synchronize operation or terminate the
command. If processing this command results in any other exception condition except early-warning, the CHECK
CONDITION status associated with that exception condition shall be reported instead. If early-warning is crossed
prior to the PROGRAMMABLE EARLY WARNING DETECTED additional sense being reported, the
PROGRAMMABLE EARLY WARNING DETECTED additional sense shall be reported before the early-warning
CHECK CONDITION.
If the PROGRAMMABLE EARLY WARNING DETECTED additional sense code was not reported, the next write
in PEWZ or beyond early-warning that would otherwise complete with GOOD status, shall return the
programmable-early-warning CHECK CONDITION instead.
If the PEWZ is entered and exited on the BOP side before the PROGRAMMABLE EARLY WARNING
DETECTED additional sense code is returned, the device server shall not report CHECK CONDITION status
with the additional sense code set to PROGRAMMABLE EARLY WARNING DETECTED.


## 4.9 Logical block protection

### 4.9.1 Logical block protection overview

The device contains hardware or software that is capable of checking and generating protection information (i.e.,
4-byte CRC) that is transferred with logical blocks between the device server and an application client. This
protection information transferred with logical blocks is saved to the medium with each logical block and read
from the medium with each logical block. This protection information is validated at the destination prior to
completing the task thereby ensuring that the logical block has not been corrupted. This level of detection is not
achievable by methods where the application client inserts vendor-specific data protection information in its data.
Some devices support a standardized method of logical block protection (see 4.9.1.1). The protection method (if
any) used to write a given block does not need to be the same as the method (if any) used to read that same
block. This includes where a drive (e.g., prior generation) which does not support the protection method used to
write a given block may read those blocks using any (or no) protection method supported on the reading drive.

#### 4.9.1.1 Logical block protection

Logical block protection support using the CRC32C (Castagnoli) algorithm (see D.2.) was added in LTO7 and
may be used by an LTO7 drive when processing any generation of cartridge supported by the LTO7 drive. When
used with prior generation cartridge it does not affect interoperability with drive generations that do not support
the CRC32C algorithm. In other words, a cartridge written with CRC32C in an LTO7 drive may be read in a
previous generation drive using a different algorithm.
A device that supports using protection information in the standardized method configures this capability using
MP 0Ah[F0h]: Control Data Protection (see 6.6.9 on page 375). Logical block protection is enabled by setting the
LOGICAL BLOCK PROTECTION METHOD field of the Control Data Protection mode page to a non-zero value. Logical
block protection is disabled by setting the LOGICAL BLOCK PROTECTION METHOD field of the Control Data Protection
mode page to zero.
A device server that supports using this protection information shall:
- set the PROTECT bit in standard inquiry (see 5.2.5.1) to one;
- set the SPT field of the Extended INQUIRY Data VPD page (see 6.3.5) to 001b; and
- set the value returned in the MAXIMUM BLOCK LENGTH LIMIT field of the READ BLOCK LIMITS command to
  a value which when added to the largest value supported in the LOGICAL BLOCK PROTECTION INFORMATION
  LENGTH field of the Control Data Protection mode page is less than or equal to the maximum length able
  to be represented in commands that transfer logical blocks between the application client and the device
  server.

### 4.9.2 Protection information on a volume

A recorded volume contains logical objects and format specific symbols. Logical objects are application client
accessible. Format specific symbols are used by the device server to provide methods for recording logical
objects on the medium in a manner that allows them to be successfully read at a later date and may not be
application client accessible. Format specific symbols contain information used to protect logical objects. The
drive includes the protection information field as one of the format specific symbols. The format specific symbol
that is the protection information field is written to the medium with each logical block. The protection information
used as a format specific symbol by the drive is a 4-byte Reed-Solomon CRC (see D.1.). A representation of
logical objects and format specific symbols is shown in figure 7.

```
                                      4-byte CRC (i.e., Protection Information) validated



 Logical     4-byte              Logical      4-byte          Logical      4-byte                  Logical      4-byte
  block       CRC                 block        CRC             block        CRC                     block        CRC



 Protected logical
      block
                      Filemark
                                 Protected logical
                                      block
                                                      ...     Protected logical
                                                                   block
                                                                                    Filemark
                                                                                               Protected logical
                                                                                                    block
                                                                                                                 Filemark   Filemark   EOD



                                                     Stream of logical object and format-specific symbols
```

*Figure 7 -- Protection information shown in relation to logical objects and format specific symbols*

The device generates the protection information and adds it to a logical block before recording the logical block
to the medium if the command that transferred the logical block being recorded to medium was received on an
I_T_L nexus for which the LOGICAL BLOCK PROTECTION METHOD field of the Control Data Protection mode page:
- is set to zero; or
- is set to a non-zero value and the LBP_W bit of the Control Data Protection mode page is set to zero.

The drive reads the protection information from the medium, validates it, and removes it from the logical block
before transferring the logical block to the application client if the command that is requesting the transfer of a
logical block being read was received on an I_T_L nexus for which the LOGICAL BLOCK PROTECTION METHOD field
of the Control Data Protection mode page:
- is set to zero; or
- is set to a non-zero value and the LBP_R bit of the Control Data Protection mode page is set to zero.

Protection information may be:
- compressed;
- encrypted; or
- included in byte counts in log parameters.

> **NOTE 1** -- Device side counters reported in log pages generally include bytes from the protection information at all times. Host side counters reported in log pages when CRC Protection and Logical block protection are disabled generally do not include bytes from the protection information. Host side counters reported in log pages when CRC Protection is enabled or when Logical block protection is enabled generally include bytes from the protection information.

### 4.9.3 Logical blocks and protection information

If the LOGICAL BLOCK PROTECTION METHOD field of the Control Data Protection mode page is set to zero for a
specific I_T_L nexus, then a logical block transferred between the application client and the device server
through that I_T_L nexus is defined by Table 14.

```
Table 14 -- Logical block with no protection information

                                                              Bit
   Byte
                  7             6             5             4            3             2             1             0
     0
                                                                 Data
    n-1
 n = the TRANSFER LENGTH field specified in CDB for variable length transfers; the BLOCK LENGTH field specified in the
 mode parameter header (see SPC-4) for fixed block transfers.
```

If the LOGICAL BLOCK PROTECTION METHOD field of the Control Data Protection mode page is set to a non-zero
value for a specific I_T_L nexus, then a logical block transferred between the application client and the device
server through that I_T_L nexus is defined by Table 15.

```
Table 15 -- Logical block with protection information

                                                              Bit
   Byte
                  7             6             5             4            3             2             1             0
     0
                                                                 Data
   n-x-1
    n-x
                                                        Protection Information
    n-1
 n = the TRANSFER LENGTH field specified in the CDB for variable length transfers; the BLOCK LENGTH field specified in the
 mode parameter header (see SPC-4) for fixed block transfers.
 x = the LOGICAL BLOCK PROTECTION INFORMATION LENGTH specified in the Control Data Protection mode page.
```

If the protection information to be transferred between the drive and the host is not the Reed-Solomon CRC, then
the protection information is transformed between the Reed-Solomon CRC and the CRC algorithm selected
(see 6.6.9).

### 4.9.4 Protecting logical blocks transferred during writes

If the LOGICAL BLOCK PROTECTION METHOD field and LBP_W bit of the Control Data Protection mode
page(see 6.6.9) is set to a non-zero value for a specific I_T_L nexus, then each logical block transferred from the
application client through that I_T_L nexus due to a WRITE(6) command contains protection information.
For the WRITE(6) command, the device server validates the protection information before the logical block is
written to medium. If the FIXED bit in the CDB is set to one each logical block is validated before being written to
the medium. If the validation of the protection information for a logical block fails, then the processing of the
command terminates prior to writing the failed logical block to the medium. If the validation of the protection
information fails, the device server reports a CHECK CONDITION status with Sense Code of Current or
Deferred, the sense key set to HARDWARE ERROR and the additional sense code set to LOGICAL BLOCK
GUARD CHECK FAILED.
An application client shall add the protection information on each logical block before transferring that logical
block and shall increase the TRANSFER LENGTH field by the length of the logical block protection information if it
has set the LOGICAL BLOCK PROTECTION METHOD field of the Control Data Protection mode page to a non-zero
value and the LBP_W bit of the Control Data Protection mode page to one.
The application client should add the protection information to the logical block at the earliest point possible. If
the data has had the protection information added to the logical block at some point in the application client prior
to the hardware that transfers the logical block, then the protection information should be validated when it is
transferred. If the validation fails, then the application client should abort the command and report a status to the
user that validation failed.

> **NOTE 2** -- The device server treats the LOGICAL BLOCK PROTECTION INFORMATION field as the protection information. If the protection information is not added to the logical block, then the validation fails when the bytes used do not validate (e.g., the last 4-bytes of the logical block are treated as the CRC and the last 4-bytes of the logical block do not calculate as the CRC of the previous data)

### 4.9.5 Protecting logical blocks processed during reads and verifies

Protection information is validated by the device server as logical blocks are processed regardless of the the
logical block protection settings. If the validation of the protection information fails, then the device server reports
a CHECK CONDITION status with Sense Code of Current Sense, the sense key set to HARDWARE ERROR
and the additional sense code set to LOGICAL BLOCK GUARD CHECK FAILED.
When a logical block is transferred to the host, if the LOGICAL BLOCK PROTECTION METHOD field and the LBP_R bit of
the Control Data Protection mode page (see 6.6.9) are set to a non-zero value for a specific I_T_L nexus, then
the protection information is transferred with the logical block to the application client on that I_T_L nexus. An
application client should validate the protection information on each logical block at the latest point possible
before using the data.


## 4.10 Multiple Port Behavior

There are two primary ports in the device and may be either Fibre Channel ports or SAS ports. The two primary
ports provide alternate paths through which the logical unit(s) of the device may be reached. The ports are
referred to as Port 0 (or alternately, Relative Target Port 1) and Port 1 (or alternately, Relative Target Port 2).
Each port maintains its own unique settings and address.
If the device is contained in a library or medium changer, the library may enable (also known as online) or disable
(also known as offline) each port independently.
When an offline port is set online, all initiators on that port receive a Unit Attention condition.
Offline ports do not generate or maintain Unit Attention conditions for initiators while the port is in an offline state.
Usage of the device with both ports online is required for dual port failover to function correctly. Generally, all
initiators, regardless of port, are treated the same as multiple initiators on the same port. The exception to this is
the handling of mode pages and reservations when a hard port reset condition occurs (such as loss of light, etc).
The following rules are described with respect to a local interface (the host port on which the hard reset condition
occurred) and a remote interface (the other host port to which the device is attached).
- If there are no reservations when a hard reset condition occurs, most mode pages are reset. All initiators
  on the local interface receive a Unit Attention condition for Power On, Reset, or Device Reset Occurred.
  All initiators on the remote interface receive a Unit Attention condition for Mode Parameters Changed.
- If there are one or more reservations when a hard reset condition occurs and all reservations were
  granted to initiators on the local interface, all mode pages are reset and all SPC-2 reservations are reset.
  All persistent reservations remain in effect. All initiators on the local interface receive a Unit Attention
  condition for Power On, Reset, or Device Reset Occurred. All initiators on the remote interface receive a
  Unit Attention condition for Mode Parameters Changed.
- If there are one or more reservations when a hard reset condition occurs and one or more of the
  reservations were granted to an initiator on the remote interface, only those mode pages and SPC-2
  reservations unique to each initiator on the local interface are reset. Mode pages and reservations
  unique to each initiator on the remote interface are not reset. Mode pages which are defined as common
  to all initiators are not reset. All initiators on the local interface receive a Unit Attention condition for
  Power On, Reset, or Device Reset Occurred. All initiators on the remote interface see no effects of the
  hard reset condition on the other interface.


## 4.11 Data Transfer, Block Limits, and Fixed Block Option

This device is designed to buffer multiple records. Logical objects may be prefetched to the buffer before they are
requested by a READ command or held in the buffer after they are written by a WRITE command. For
successive sequential-motion operations, the presence of the buffering in the device does not adversely affect
the performance of the subsystem. Non-sequential motion does not result in errors, but may result in delays
because of requirements to synchronize buffers or discard read ahead data. Buffer management in the device
determines when to read additional data from the medium into the buffer, or when to write data from the buffer to
the medium. A logical block is not written to tape until the block is entirely received into the buffer.
When the FIXED bit of the command is set to 1b, each command transfers zero or more logical blocks. The
subsystem takes appropriate action to assemble or disassemble the logical blocks being transferred over the
interface so that they remain independent blocks on the medium. There is no guarantee that the group of blocks
transferred by the Write command is requested as a group by a subsequent Read command, so the device must
be prepared to assemble and disassemble on a block boundary. This is managed by treating all blocks and
filemarks as independent from one another, both for data compaction and for recording.
When the FIXED bit of the command is set to 0b and the TRANSFER COUNT is non-zero, each command processes
a single logical object.
The device supports a minimum logical block length of 1 and a maximum logical block length of 16 777 215
(FF FFFFh) bytes if encryption is not being used and 8 388 608 (80 0000h) bytes if encryption is being used. Any
block length between the limits is also supported. See 5.2.17--READ BLOCK LIMITS - 05h for further
information on block sizes and limitations. The READ BLOCK LIMITS command may report a lower maximum
value depending on the support of Encryption and Logical Block Protection. If the logical object identifier of the
current position on medium is greater than FFFFFF00h and less than FFFFFFF0h, then rules for Logical EOM
processing are applied. If the logical object identifier of the current position on medium is greater than or equal to
FFFFFFF0h, rules for physical end of partition processing are applied.
For read type commands, including READ and VERIFY, transfer lengths larger than the maximum device
supported block size are accepted and the underlength condition rules are applied for transfer requests bigger
than the actual block size. A transfer Length of 000000h indicates that no bytes/blocks are transferred. This
condition is not considered an error and the logical position is not changed.
For write type commands, including WRITE, and WRITE FILEMARKS, a transfer Length of 000000h indicates
that no bytes/blocks are transferred. This condition is not considered an error and the logical position is not
changed.


## 4.12 Request Sense Information, ILI, and Command Interactions

The behavior and interactions between some of the commands and the INFORMATION and ILI fields in Request
Sense are rather complicated. This section details the various commands which may set the information or ILI
fields, and summarizes the relationship between such commands, their parameters, the encountered conditions,
the reported status, and the expected behavior of these fields and the resulting device position.

### 4.12.1 General Read-Type Handling

Commands which return block data from media or the buffer to the host have the same general behavior. These
commands include READ and VERIFY. The major difference between these is whether or not data is returned to
the host.
The block at the current position is processed first, and subsequent blocks are processed in the order they were
written (proceeding towards logical end of partition). The ending position is after the last block processed. For
these commands, "after" will refer to the start of the next block towards the logical end of partition, and "before"
will refer to the start of referenced block.
To illustrate this, from location 'N', a Read operation will return block 'N', and be positioned at 'N+1' ("after" N).

A successful command with a FIXED bit of 1b transfers the requested Transfer Length, times the current block
length in bytes to the initiator. A successful command with a FIXED bit of 0b transfers the requested Transfer
Length in bytes to the initiator. Upon completion, the logical position is "after" the last block transferred.
If SILI bit is 1b and the FIXED bit is 0b, the target performs one of the following actions:
- Reports CHECK CONDITION status for an incorrect block length condition only if the overlength
  condition exists and the BLOCK LENGTH field in the mode parameter block descriptor is non-zero. The
  associated sense data is 0/0000 (INCORRECT LENGTH, NO SENSE DATA).
- Does not report CHECK CONDITION status if the only error is the underlength condition, or if the only
  error is the overlength condition and BLOCK LENGTH field of the mode parameters block descriptor is 0b
  (see Note 4 on page 49).

If the SILI bit is 1b and the FIXED bit is 1b, the target terminates the command with CHECK CONDITION status
with associated sense data of 5/2400 (ILLEGAL REQUEST, INVALID FIELD IN CDB).
If the SILI bit is 0b and an incorrect length block is read, CHECK CONDITION status is returned and the ILI and
VALID bits are set to 1b in the sense data. Upon termination, the logical position is "after" the incorrect length
block. If the FIXED bit is 1b, the INFORMATION field is set to the requested Transfer Length, minus the actual
number of blocks read (not including the incorrect length block).
If the FIXED bit is 0b, the INFORMATION field is set to the requested Transfer Length, minus the actual block length
in two's complement format.
If the logical unit encounters a filemark during a command, CHECK CONDITION status is returned and the
FILEMARK and VALID bits are set to 1b in the sense data. The associated sense data is set to 0/0001 (NO SENSE,
FILEMARK DETECTED). Upon termination, the logical position is "after" the filemark. If the FIXED bit is 1b, the
INFORMATION field is set to the requested Transfer Length, minus the actual number of blocks read (not including
the filemark). If the FIXED bit is 0b the INFORMATION field is set to the requested Transfer Length.
If the logical unit encounters end-of-partition during a command, CHECK CONDITION status is returned and the
EOM and VALID bits are set to 1b in the sense data. Associated sense data is set to 3/0002 (MEDIUM ERROR,
END OF PARTITION/MEDIUM).
If the logical unit encounters early warning and the REW bit is set to 1 in the Device Configuration mode page,
CHECK CONDITION status is returned and the EOM and VALID bits are set to 1b in the sense data. Associated
sense data is set to D/0002 (OVERFLOW, END-OF-PARTITION/MEDIUM DETECTED). If the FIXED bit is 1b, the
INFORMATION field is set to the requested Transfer Length, minus the actual number of blocks transferred. If the
FIXED bit is 0b, the INFORMATION field is set to the requested Transfer Length.

If the drive encounters End-of-Data (EOD) while processing this command, the command is terminated at the
EOD position and CHECK CONDITION status is returned with associated sense data of 8/0005 (BLANK
CHECK, END-OF-DATA DETECTED).
If the logical unit encounters beginning-of-partition during a command, CHECK CONDITION status is returned
and the EOM and VALID bits are set to 1b in the sense data. Associated sense data is set to 0/0004 (NO SENSE,
BEGINNING OF PARTITION/MEDIUM).

> **NOTE 3** -- Because the residue information normally provided in the INFORMATION field of the sense data may not be available when the SILI bit is set, use other methods to determine the actual block length. For example: include length information in the data block itself, or in the case of underlength transfers, the host adapter or device driver may return accurate transfer length information.

> **NOTE 4** -- In the case of the FIXED bit of 1b with an overlength condition, only the position of the incorrect-length logical block can be determined from the sense data. The actual length of the incorrect block is not reported, and also cannot be derived from the transfer length (the device truncates the overlength block to match the current block length from the mode header). Other means may be used to determine the actual length (for example, backspace and read it again with FIXED bit set to 0b).

### 4.12.2 Interactions Summary

The following table summarizes various commands with the specified options, the encountered conditions, and
the expected results.

```
Table 16 -- Information and ILI Behavior Summary (part 1 of 3)

                                   Block      Sense                          Flags
   Scenario        Fixed    SILI                         Information1 2                     Position1
                                   Length     Error1                         IFE1
 reportable UA      X        X       X         UA          not valid (0)       -     unchanged (no command)
   reportable
                    X        X       X         DCC         not valid (0)       -     unchanged (no command)
     DCC
                    1        1       0        5/2400     transfer length       -       unchanged (no read)
  Read (any)
                    1        0       0        5/2400     transfer length       -       unchanged (no read)
      Read
 transfer length    X        X       X         good              -             -       unchanged (no read)
        0
     Read
    (correct        X        X       X         good              -             -          after last block
   length(s))
                                                         transfer length -
                    0        0       X        0/0000                           I            after block
                                                           block size (+)
                    0        1       X         good              -             -            after block
    Read
                                                         transfer length -
  Underlength
                                                         blocks read not
                    1        0     non-0      0/0000         including         I       after incorrect block
                                                          incorrect block
                                                                (+)
                                                         transfer length -
                    0        0       X        0/0000                           I            after block
                                                           block size (-)
                    0        1       0         good              -             -            after block
                                                         transfer length -
    Read            0        1     non-0      0/0000                           I            after block
                                                           block size (-)
  Overlength
                                                         transfer length -
                                                         blocks read not
                    1        0     non-0      0/0000         including         I       after incorrect block
                                                          incorrect block
                                                                (+)
                    0        X       X        0/0001     transfer length      F            after filemark
                                                         transfer length -
   Read FM                                               blocks read not
                    1        0     non-0      0/0001                          F            after filemark
                                                        including filemark
                                                                (+)
                    0        X       X        8/0005     transfer length      E7       unchanged (at EOD)
  Read EOD                                               transfer length -
                    1        0     non-0      8/0005                          E7     after last block (at EOD)
                                                          blocks read (+)
                    0        X       X        perm       transfer length       -       unchanged (at perm)
  Read Perm                                              transfer length -
                    1        0     non-0      perm                             -     after last block (at perm)
                                                          blocks read (+)
```

```
Table 16 -- Information and ILI Behavior Summary (part 2 of 3)

                                 Block       Sense                             Flags
  Scenario       Fixed    SILI                             Information1 2                       Position1
                                 Length      Error1                            IFE1
                  0        X       X         3/1404        transfer length       -     crossed EOD (position may
 Read after                                                                             change in non-predictable
 EOD/Perm                                                 transfer length -            fashion, limited commands
                  1        0     non-0       3/1404                              -
                                                           blocks read (+)                      available)
                  0        X       X         0/0004        transfer length      E               at BOP (0)
Read (reverse)
   BOP                                                    transfer length -
                  1        0     non-0       0/0004                             E               at BOP (0)
                                                           blocks read (+)
 Write (any)      1        -       0         5/2400        transfer length       -        unchanged (no write)
Write transfer
                  X        -       X            -                 -              -        unchanged (no write)
  length 0
                  0        -       X      0/0000 0/0002           0             E               after block
Write in Early                                            transfer length -
  Warning         1        -     non-0    0/0000 0/0002     blocks written      E          after blocks written
                                                              (usually 1)
Write at EOM      X        -       X         D/0002        transfer length      E         unchanged (no write)
                                                          transfer length or
                  0        -       X          perm          0 (if data is in     -       after last block in buffer
                                                                 buffer)
 Write Perm                                               transfer length -
                                                               blocks
                  1        -     non-0        perm                               -       after last block in buffer
                                                          transferred into
                                                               buffer
 Write after
                  X        -       X         3/3100        transfer length       -        unchanged (no write)
   Perm
   Locate
(target after      encountered EOD           8/0005         not valid (0)5      E7               at EOD5
    EOD)
                                                                                       indeterminate (unchanged or
   Locate          encountered Perm           perm          not valid (0)5       -
                                                                                                at perm)5
                      encountered FM         0/0001                             F                after FM
                   encountered EOD           8/0005                             E6               at EOD
   Space                                                   Count - blocks
   blocks          encountered BOP           0/0004         traversed3          E               at BOP (0)
                                                                                       indeterminate (unchanged or
                   encountered perm           perm                               -
                                                                                                at perm)3
                   encountered EOD           8/0005                             E6               at EOD4
    Space          encountered BOP           0/0004         Count - FMs         E               at BOP (0)
  filemarks                                                  traversed4
                                                                                       indeterminate (unchanged or
                   encountered perm           perm                               -
                                                                                                at perm)4
```

```
Table 16 -- Information and ILI Behavior Summary (part 3 of 3)

                                     Block        Sense                              Flags
  Scenario        Fixed       SILI                               Information1 2                        Position1
                                     Length       Error1                             IFE1
                     encountered EOD              8/0005             Count -          E6                at EOD5
                                                                 sequential FMs
     Space           encountered BOP              0/0004                               E               at BOP (0)
                                                                   traversed
  sequential
                                                                immediately prior
   filemarks                                                                                 indeterminate (unchanged or
                     encountered perm              perm             to ending          -
                                                                                                      at perm)5
                                                                    position5
                     encountered EOD               good                  -5            -                at EOD5
     Space
      EOD                                                                                    indeterminate (unchanged or
                     encountered perm              perm            not valid (0)5      -
                                                                                                      at perm)5
                                                         Legend:
 Flags:
      I        ILI bit                 #/####    CC, sense of Sense Key/ASC ASCQ
      E        EOM bit                 perm      CC, sense as per perm
      F        Filemark bit            good      No CC (no sense)

      -        None set
      -        Not applicable
                                                          Notes:
 1     These fields are outputs (results) from the scenario operation.
 2     Partial blocks are not considered read, written or traversed.
 3     Information field will accurately reflect the ending position.
 4     Information field will accurately reflect the ending position but it is not in units of logical blocks, so additional
       means of determining absolute location, such as Read Position, must be used.
 5     Information field does not accurately reflect the ending position, another means of determining absolute
       location, such as Read Position, must be used.
 6     The EOM bit may be set only if the current position is in the early warning region or if the end of partition is
       encountered.
 7     The EOM bit will only be set if end of partition is encountered (this condition should never occur), so EOM
       should not be set in this case. The standard specifies that EOM bit shall be set only if the current position is in
       the early warning region or if the end of partition is encountered.
```


## 4.13 Drive Cleaning

### 4.13.1 Cleaning the Drive in a Library

In a library, the drive may be automatically cleaned. If the library configures the drive for automatic cleaning, then
the drive behaves as follows:
When the drive determines that either maintenance cleaning is required, or that the SARS thresholds have been
reached, a message is sent to the library (via the Library/Drive interface) to request cleaning. This occurs when
the Cleaning message is normally sent to the SCD (Single Character Display). The library schedules the
mounting of the cleaning cartridge. Thus, the host operating system and application are freed of any
responsibility to facilitate the cleaning.

### 4.13.2 Drive Cleaning Indicators

For stand-alone drive models, automatic cleaning of the drive is not possible. For library models, automatic
cleaning of the drives by the library may be disabled (although it is not recommended). For either case, cleaning
of the drives must be managed by the host application or manually, by the operator.

> **NOTE 5** -- Failure to clean a drive may result in data loss.

This section describes how cleaning indicators are presented from the drive. The cleaning indicators may be
presented even with automatic cleaning enabled in a library environment. The cleaning indicators can be
presented through the following:
- Panel Cleaning Indication (see 4.13.2.1)
- Host Interface - Dynamic Cleaning Indicators (see 4.13.2.2)
- Host Interface - Static Cleaning Indicator (Sense Data Byte 70) (see 4.13.2.3)

#### 4.13.2.1 Panel Cleaning Indication

A CLEAN message is displayed on the SCD (Single Character Display) when cleaning with a cleaning cartridge
is required. For additional details, see the Operator Guide for this product.

#### 4.13.2.2 Host Interface - Dynamic Cleaning Indicators

Dynamic cleaning indicators that are sent across the host interface include:
- ASC/ASCQ codes related to cleaning in Error Sense Information (see Annex B. on page 481). Cleaning
  Indicators reported with Sense Key 1 may only be reported in certain situations, see 6.6.5--MP 01h:
  Read-Write Error Recovery.

| Code Description | Sense Key | ASC ASCQ |
|------------------|-----------|----------|
| Drive Requires Cleaning | 0 | 82 82 |
| Cleaning in Progress (cleaner cartridge) | 2 | 30 03 |

- TapeAlert codes related to cleaning as described in LP 2Eh: TapeAlerts (see 6.4.18 on page 316).

> **NOTE 6** -- If the device driver shields the application from dynamic notifications, the information is usually available from the system error log.

#### 4.13.2.3 Host Interface - Static Cleaning Indicator (Sense Data Byte 70)

The bit significance of sense data byte 21 follows:

| Bit | Description |
|-----|-------------|
| 3 | Set to 1b "Cleaning Required: Normal Maintenance" when cleaning is required because of the normal preventive maintenance guideline, see 6.6.20--MP 2Fh: Behavior Configuration. Reset to 0b when the cleaning cartridge is loaded. |

### 4.13.3 Cleaning Criteria

There are two main criteria used by the drive to call for cleaning:
- Clean Required (also known as Clean Now TapeAlert 14h)
  Clean Required is triggered when the drive posts specific permanent errors or is running degraded. It is
  not based on temporary or permanent error rates. The permanent errors are typically read/write perms
  or servo related perm failures. Not all read/write or servo perms will trigger a clean. The errors are
  typically sticky, which means that the drive may not allow data operations unless a clean is performed,
  even if a power cycle occurs;
- Clean Requested (also know as Clean Periodic TapeAlert 15h)
  Clean Requested is based on usage, but not media motion hours. Two criteria are used, Data sets
  processed or Meters of tape pulled across the head. If another cartridge is inserted after Clean
  Requested is asserted, the drive continues to operate. However, the 'C' on the Single Character Display
  (SCD) of the drive persists until the drive is cleaned or power cycled. If the drive is power cycled, the 'C'
  will reappear on the SCD until the drive is cleaned. Periodic clean events continue to be posted to the
  engineering log (see 6.7.2.1) after every cartridge.

```
Table 18 -- Drive Cleaning Criteria to assert Clean Requested

                                          Data Sets Processed                    Head Tape Meters Pulled

             Generation                                    Equivalent                                Equivalent
                                        Criterion           Full File           Criterion             Full File
                                                            Passes a                                  Passes a
    LTO 5 HH                                 5 000 000          8                     2 500 000           39
    LTO 5 FH                                 7 500 000         12                     3 750 000           58
    LTO 6 FH & HH                           15 000 000         15                     3 750 000           34
    LTO 7 FH & HH                           18 000 000         15                     3 750 000           36
    LTO 8 FH & HH                           18 000 000         7.5                    3 750 000           19
    LTO 9 FH & HH                           18 000 000         10                     3 750 000           13
    Key:
    HH - Half-Height
    FH - Full-Height
     a
          Equivalent Full File Passes is an estimate and are not used as criteria. This information provides a
          feel for the criteria used. Note that these criteria do not consider whether or not the tape is actually
          used in a full-file manner or whether the tape is only used repeatedly around a short area of tape.
```


## 4.14 WORM Behaviors

### 4.14.1 Conditions for Writing

If the following condition is met, writing is allowed:
- the cartridge is uninitialized

If all the following conditions are met, writing is allowed:
- the current logical position is at BOP
- there are only filemarks between here and EOD

If all of the following conditions are met, writing is allowed:
- if the current logical position is at BOP
- there are exactly 1 or 2 data records, followed by 0 to infinite number of filemarks, followed by no data
  records, followed by EOD

If all of the following conditions are met, writing is allowed:
- the current logical position is between BOP and EOD:
- there are only filemarks from the current logical position to EOD
- there is at least one filemark immediately before the current logical position

If the following condition is met, writing is allowed:
- the current logical position is at EOD

### 4.14.2 Command Behavior When WORM Medium Has Been Tampered With

Table 19 specifies the behavior of the device when it has detected the WORM medium that is loaded in the drive
has been tampered with, see 6.6.11--MP 10h: Device Configuration.

| Command | WTRE=01b | WTRE=00b or 10b |
|---------|----------|-----------------|
| WRITE | 7/300Dh | 7/300Dh |
| WRITE FILEMARK n (n !=0) | 7/300Dh | 7/300Dh |
| WRITE FILEMARK 0 (buffered data) | 7/300Dh | 7/300Dh |
| WRITE FILEMARK 0 (no buffered data) | GOOD | GOOD |
| ERASE | 7/300Dh | 7/300Dh |
| READ | GOOD | 3/300Dh |
| VERIFY | GOOD | 3/300Dh |
| SPACE | GOOD | 3/300Dh |
| LOCATE to (block !=0) | GOOD | 3/300Dh |
| LOCATE to 0 | GOOD | GOOD |
| REWIND | GOOD | GOOD |
| UNLOAD | GOOD | GOOD |
| LOAD | GOOD | GOOD |


## 4.15 Device Hardware Encryption

This device contains hardware which performs user data write encryption and read decryption, protecting all user
data written to the medium from unauthorized use [provided it is integrated into a secure system design]. Device
support for encryption may be determined by reading MP 24h: Vendor-Specific (see 6.6.19 on page 398) with the
MODE SENSE command.
This device supports multiple ways of controlling encryption settings. These encryption control methodologies
are called:
- Encryption Control - IBM Proprietary Protocol (IPP) (see 4.15.1 on page 55); and
- Encryption Control - T10 Standards (see 4.15.2 on page 56).

On volumes with multiple partitions, the drive handles encryption in each partition as determined by the state of
the partition, position of the write and the current method / mode / policy:

- if the encryption method in the drive is set to the IBM proprietary methods (see 4.15.1) and the position is
  at BOP (logical object 0), then the block encryption is determined by the BOP write policy (if set to
  encrypt the write will not be allowed if there is no current key);
- if the encryption method in the drive is set to the IBM proprietary methods (see 4.15.1) and the current
  position is not at BOP (logical object greater than 0), then encryption is required only if there is at least
  one encrypted block anywhere on that partition; and
- if the encryption method in the drive is set to AME-T10 (see 4.15.2), then an intermix of encrypted and
  unencrypted blocks are allowed.

### 4.15.1 Encryption Control - IBM Proprietary Protocol (IPP)

The following terms are used to describe the methods of control that fall into the IPP:
- Library Managed Encryption (LME);
- System Managed Encryption (SME); and
- Application Managed Encryption - IBM (AME-IBM).

When a device is enabled to perform encryption using one of the IBM Proprietary Protocols (i.e., LME, SME, or
AME-IBM) encryption parameters are determined at first write from BOP. On volumes with multiple partitions this
means that on a write from BOP (i.e. LBA 0) of each partition the encryption parameters are determined. Writes
away from BOP use the existing encryption parameters. If any logical block on the partition is encrypted, then all
logical blocks subsequently written to the partition must be encrypted. If no logical blocks on the partition are
encrypted then subsequent logical blocks are not required to be encrypted. When a partition change occurs the
encryption parameters are cleared.

Please see IBM for additional information on IPP.

### 4.15.2 Encryption Control - T10 Standards

The T10 standards method of controlling encryption are described in SSC-5 as well as in this document. Note
that not all methods described in SSC-5 are supported.
This device uses the term Application Managed Encryption - T10 (AME-T10) to signify that it is using this
standards based method.
When this device is enabled to perform encryption using AME-T10 the encryption parameters are set by either
the application or the library (see 4.15.2.1) depending on how AME-T10 is configured. When the encryption
parameters are set to encrypt, logical blocks are encrypted. When encryption parameters are set to not encrypt,
logical blocks are not encrypted. Changing partitions when enabled for AME-T10 does not affect the encryption
parameters.
This device supports the T10 method of passing the key in clear text. Some generations support RSA key
wrapping with KEY FORMAT 02h. For specifics on support see 5.2.34--SECURITY PROTOCOL IN (SPIN) -
A2h, 5.2.35--SECURITY PROTOCOL OUT (SPOUT) - B5h, and 6.8--Security Protocol Parameters (SPP).

#### 4.15.2.1 External Data Encryption Control

This device supports control of encryption via the Automation/Drive Interface (ADI) using some of the methods
described in ADC-4. Refer to the ADI Implementation Reference for a description of how to enable these
methods.


## 4.16 Attachment Features

### 4.16.1 Types of Interface Attachments

This device communicates with servers that use Fibre Channel or SAS interfaces. The interfaces share certain
tape LUN behaviors, but also possess unique features. This chapter describes the common and unique features
of these interfaces.

### 4.16.2 Common Tape LUN Behaviors

Fibre Channel and SAS attached devices share the following tape LUN behaviors:
- Power-On (see 4.16.2.1 on page 56);
- Reset Strategy (see 4.16.2.2 on page 57);
- Abort Handling (see 4.16.2.3 on page 57);
- Multi-initiator Support (see 4.16.2.4 on page 58); and
- Status Codes (see 4.16.2.5 on page 59).

#### 4.16.2.1 Power-On

The first UAT eligible command (see table 30) from any initiator gets a CHECK CONDITION status with UNIT
ATTENTION sense data for the power-on. After this, any medium access command is reported with a Sense Key
of NOT READY and an additional sense code of LUN HAS NOT SELF-CONFIGURED YET (3E00).

If a cartridge is mounted in the drive when it powers up, the cartride is unloaded, and once the drive has
completed its self test and setup procedures, the drive attempts to load the cartridge. During this unmount,
self-test, and remount processing, medium access commands are reported with an additional sense code of
DRIVE IN PROCESS OF BECOMING READY (0401).

#### 4.16.2.2 Reset Strategy

The drive supports the hard reset option as is required by SCSI-3. On receiving a reset, the following actions are
taken:
- The current I/O process is aborted, as in 4.16.2.3--Abort Handling.
- Any queued I/O processes from other initiators are removed.
- All SPC-2 reservations are cleared, but Persistent Reservations remain in effect.
- Most mode values are reset to their defaults.
- A unit attention condition is set.
- A logical position is established that may or may not be the same as the position prior to the reset.
  Where possible, the logical position prior to reset is maintained.
- The next command that is eligible for the UNIT ATTENTION CHECK CONDITION from each initiator
  gets a CHECK CONDITION STATUS, with UNIT ATTENTION sense data for the reset. However, other
  commands may not be processed until the internal state of the drive has been reset.

#### 4.16.2.3 Abort Handling

Table 20 specifies the abort processing for this device.

```
Table 20 -- Abort Condition Handling (part 1 of 2)

            Command                                               Abort Processing
ALLOW OVERWRITE                     The Command completes
                                    Long erase is aborted as quickly as possible without corrupting tape format.
ERASE
                                    Short erase completes.
                                    If modification to medium has started then the command is completed;
FORMAT
                                    otherwise, no action is taken.
INQUIRY                             None.
                                    Load completes (e.g., if the HOLD bit is zero, logically positions tape at BOP 0).
                                    Unload is aborted, leaving logical position at BOP 0 unless operation is past
LOAD/UNLOAD
                                    the 'point of no return', in which case the unload completes (e.g., if the HOLD bit
                                    is zero, the tape is ejected).
                                    The logical position is set back to that at the start of the operation unless the
LOCATE
                                    operation is past its 'point of no return', in which case the operation completes.
                                    If data transfer is completed, command is completed; otherwise, no action is
LOG SELECT
                                    taken.
LOG SENSE                           None.
                                    If data transfer is completed, command is completed; otherwise, no action is
MODE SELECT
                                    taken.
MODE SENSE                          None.
PERSISTENT RESERVE IN               None.
                                    If data transfer is completed, the command is completed; otherwise, no action
PERSISTENT RESERVE OUT
                                    is taken.
PREVENT ALLOW MEDIUM
                                    The command completes.
REMOVAL
```

```
Table 20 -- Abort Condition Handling (part 2 of 2)

           Command                                                Abort Processing
                                    The current position is set after the last logical block to be completely
 READ
                                    transferred to the host.
 READ ATTRIBUTE                    None.
 READ BLOCK LIMITS                 None.
 READ BUFFER                       None.
 READ POSITION                     None.
 RECEIVE DIAGNOSTIC RESULTS        None.
 RELEASE UNIT                      The command completes.
 REPORT DENSITY SUPPORT            None.
 REPORT LUNs                       None.
 REPORT SUPPORTED OPERATION
                                    None.
 CODE
 REPORT SUPPORTED TASK
                                    None.
 MANAGEMENT FUNCTIONS
 REQUEST SENSE                     Sense data is discarded.
 RESERVE UNIT                      The command completes.
 REWIND                            The command completes.
 SECURITY PROTOCOL IN              None.
                                    If data transfer is completed, the command is completed; otherwise, no action
 SECURITY PROTOCOL OUT
                                    is taken.
 SEND DIAGNOSTIC                   Vendor unique.
                                    If modification to medium has started then the command is completed;
 SET CAPACITY
                                    otherwise, no action is taken.
                                    The logical position is set back to that at the start of the operation unless the
 SPACE
                                    operation is past its 'point of no return', in which case the operation completes.
 TEST UNIT READY                   None.
                                    The logical position is set to the next record boundary after the point where the
 VERIFY
                                    verify was aborted.
                                    Depending on where in the processing of the command the drive is, either no
                                    logical blocks, the logical block, or some of the logical blocks, if transfering in
 WRITE
                                    fixed block mode, are written to the buffer. The logical position is set to the
                                    point where the last block was written.
                                    If data transfer is completed, the command is completed; otherwise, no action
 WRITE ATTRIBUTE
                                    is taken.
                                    If data transfer is completed, the command is completed; otherwise, no action
 WRITE BUFFER
                                    is taken.
 WRITE FILEMARKS                   The command completes.
```

#### 4.16.2.4 Multi-initiator Support

This device supports an infinite number of I_T nexuses, but the device has a limit on how many I_T nexuses can
be logged in processing commands concurrently. If this limit is exceeded, then the device implicitly logs out the
least recently used (LRU) I_T nexus that:
- is not reserved;
- is not registered;
- is not the I_T nexus that last processed a medium access command; and
- does not have an outstanding command.

The device supports untagged queuing when operating with multiple initiators. If a command from one initiator is
being processed when a command other than INQUIRY, REPORT LUNs, REQUEST SENSE, and TEST UNIT
READY is received from a second initiator, the new command may be queued. Media access commands (for
example, WRITE, WRITE FILEMARKS, READ, VERIFY, REWIND, MODE SELECT that changes block size) are
always processed in strict order of receipt.
The INQUIRY, REPORT LUNS, REQUEST SENSE, and TEST UNIT READY commands are always processed
immediately, irrespective of whether a command from another initiator is being processed.
The drive maintains sense data for the supported number of initiators. If an additional initiator connects to the
drive and causes an initiator to be implicitly logged out, the drive erases all sense data for that initiator before
processing the command for the new initiator. See 4.19.2--Sense Data Management for more details of sense
data management.

#### 4.16.2.5 Status Codes

| Status Code | Value | Circumstance |
|-------------|-------|-------------|
| GOOD | 00h | The command completed without problems. |
| CHECK CONDITION | 02h | A problem occurred during command processing. The sense data should be examined to determine the nature of the problem. |
| BUSY | 08h | The drive is unable to accept the command at this time. This status is returned during the power-on sequence or if there are commands from too many I_T nexuses outstanding, see 4.16.2.4--Multi-initiator Support. |
| RESERVATION CONFLICT | 18h | This status is returned if the drive is reserved for an I_T nexus other than the one sending the command. |
| QUEUE FULL | 28h | Not normally returned. |

### 4.16.3 Features of the Fibre Channel Interface

This device is compliant with the American National Standard, Project T10/Project 1828-D, Information
Technology - Fibre Channel Protocol for SCSI, Fourth Version (FCP-4), Revision 02b, January 3, 2011. The key
features of the FC-Tape Technical Report of the Accredited Standard Committee NCITS that were found useful
are included in FCP-4. IBM recommends that a server's device driver and host bus adapter (HBA) use:
- Precise delivery of commands;
- Confirmed completion of FCP I/O operations;
- Retransmission of unsuccessfully transmitted IUs; and
- Task retry identification as defined in FCP-4.

These features may be listed in HBA settings either individually or as a group and called:
- FCP-2 support;
- Class-3 Error Recovery;
- FC-Tape;
- Confirmed Completion; or
- Task retry identification.

The World Wide Node Name and Port Name that are used by the device follow the format of the Institute of
Electrical and Electronics Engineers (IEEE).

#### 4.16.3.1 Topology

Fibre Channel devices (such as this device and a server) are known as nodes and have at least one port through
which to receive and send data. The collection of components that connect two or more nodes is called a
topology. Fibre Channel systems consist solely of two components: nodes with ports and topologies.
Each port uses a pair of fibers: one fiber carries data into the port, and the other carries data out of the port. The
fibers in the channel are optical strands. The fiber pair is called a link and is part of the topology. Data is
transmitted over the links in units known as frames. A frame contains an address identifier that gives the fabric
and node for which the frame is destined.
This device can be attached in a two-node configuration, either directly to a switch as a public device (switched
fabric) or directly to a host bus adapter (HBA) as a private device (direct connection). This device may be
configued to any supported topology via a library interface, or configured by using vital product data (VPD)
settings. The type of connection also depends on whether the drive recognizes the connection as a loop or a
fabric connection:
- An L_port supports a Fibre Channel Arbitrated Loop connection to an L_port or FL_port.
- An N_port supports direct connection to an F_port (for example, a director-class switch) in a fabric
  topology.

Regardless of the port to which this device is connected, it automatically configures to a public device (through
an F_port or FL_port to a switch) or to a private device (through an L_port by using direct attachment to a
server). This device supports two topologies:
- Two-Node Switched Fabric Topology (see 4.16.3.1.1 on page 60); and
- Two-Node Direct Connection Topology (see 4.16.3.1.2 on page 61).

Table 22 lists the topologies in which this device is able to operate, the Fibre Channel server connections that are
available, and the port (NL, N, FL, or F) through which communication must occur. The sections that follow
describe each topology.

```
Table 22 -- Topologies through which this device's port(s) can operate

                                                 Type of Fibre Channel Port to Which the Drive Port Connects
                                                     Server Port (HBA)                      Switch Port
                                                (Private - Direct Connection)        (Public - Switched Fabric)
         Drive Port Configuration
                                               Point-to-Point     Arbitrated Loop Topology (FC-AL)         Fabric
                                                 Topology                                                Topology
                                                 (N_Port)            L_Port            FL_Port            (F_Port)
                                                Invalid system                                         Invalid system
 Drive port configured to operate as L_Port                           L_Port            L_Port
                                                 configuration                                          configuration
                                                   N_Port         Invalid system        N_Port
 Drive port configured to operate as N_Port                                                               N_Port
                                                (not supported)     configuration   (switched fabric)
                                                    N_Port
 Drive port configured to operate as LN_Port    (not supported;       L_Port            L_Port            N_Port
                                               attempts L_Port)
                                                    N_Port
 Drive port configured to operate as NL_Port    (not supported;       L_Port           N_Port             N_Port
                                               attempts L_Port)
```

##### 4.16.3.1.1 Two-Node Switched Fabric Topology

The two-node switched fabric topology supports two protocols:
- Use the two-node switched fabric loop protocol when attaching the device to an FL_port; and
- Use the two-node switched fabric protocol when attaching the device to an F_port.

##### 4.16.3.1.2 Two-Node Direct Connection Topology

A two-node direct connection occurs when two Fibre Channel end points are connected together. Either
Arbitrated Loop or Point-to-Point topology is defined as usable in the standards, but both end points must use the
same topology. Most Fibre Channel adapters have settings that allow selection of the topology or they default to
the loop topology when they are not directly connected to a fabric. While this device allows you to set the port to
any of these topologies (see the instructions of the library in which this device is contained) this device supports
only the use of the Arbitrated Loop (L_port) topology in a two-node direct connection. Use of the Point-to-Point
topology in a two-node direct connection to an N_port is not supported, but not prohibited.

#### 4.16.3.2 Speed

This device is an 8GFC device. This device also allows operation using previous Fibre Channel generations:
1GFC, 2GFC, and 4GFC. Each Fibre Channel generation transfers data at the following rates:
- 1GFC transfers data at a max burst rate of 100 MB/s;
- 2GFC transfers data at a max burst rate of 200 MB/s;
- 4GFC transfers data at a max burst rate of 400 MB/s;
- 5GFC transfers data at a max burst rate of 800 MB/s;

This device may be configured via a library interface or vital product data (VPD) in one of the following speed
configurations:
- 1GFC;
- 2GFC;
- 4GFC;
- 8GFC; or
- speed negotiate

#### 4.16.3.3 Addressing Assignments

Each Fibre Channel interface port for this device can be independently assigned a specific speed and topology,
or may be set to auto-negotiate.
When the topology is set to or negotiates to L-port, a hard or soft ALPA ID may be assigned. The hard ALPA ID
is in the range of 01h to EFh with only certain valid values (a total of 126 addresses). Validity is enforced by the
entry process. This value should be unique to each device on the Fibre Channel loop. Fibre Channel loop
protocol will detect an addressing conflict on the loop, and one of the conflicting drives will not be available for
use.

### 4.16.4 Features of the Serial Attached SCSI (SAS) Interface

The World Wide Node Name and Port Name that are used by this device follow the format of the Institute of
Electrical and Electronics Engineers (IEEE).
This device is compliant with the American National Standard, Project T10/1760-D, Information technology -
Serial Attached SCSI - 2 (SAS-2), Revision 16, 18 April 2009.


## 4.17 Device Clocks

The drive supports a Device Clock that maintains a timestamp for various items. This timestamp gets recorded in
drive error logs.
The TIMESTAMP ORIGIN is one of those specified in table 23.

| Value | Definition |
|-------|-----------|
| 000b | Timestamp initialized to zero at power-on |
| 001b | Reserved |
| 010b | Timestamp initialized by the SET TIMESTAMP command |
| 011b | Timestamp initialized by the Library over the Library Drive Interface (i.e. RS-422) |
| 100b - 111b | Reserved |

Once a timestamp is initialized it begins counting from that time forward. Once the timestamp is initialized it
remains in effect until one of the following occurs:
- A SET TIMESTAMP command (see 5.2.38) is processed;
- An LDI command is processed that modifies the timestamp; or
- A Hard Reset event occurs.

The method used is indicated in the Extended Ctl mode page.
The Timestamp is not affected by an I_T nexus loss or a Logical Unit reset.
The TIMESTAMP is specified in table 24.

```
Table 24 -- TIMESTAMP Layout

                                                               Bit
   Byte
                 7           6            5             4                3         2            1            0
     0        (MSB)
                                                            TIMESTAMP
     5                                                                                                      (LSB)
```

The TIMESTAMP field contains the value established at the last action that set the timestamp incremented by one
for every millisecond that has elapsed since the timestamp was set.


## 4.18 Dynamic runtime information

### 4.18.1 Dynamic runtime information overview

Dynamic runtime information allows an initiator to set dynamic runtime attributes (DRA) about itself into a device
server. The device server then associates those attributes to the I_T_L nexus and uses the information and
associations for enhanced data collection and debugging. This information and the associations are added to
device error logs (e.g., drive dump) and are provided for retrieval by an application client through the READ
DYNAMIC RUNTIME ATTRIBUTE command (see 5.2.19).
The Ultrium 5 and later devices support dynamic runtime attributes with the READ DYNAMIC RUNTIME
ATTRIBUTE command (see 5.2.19) and the WRITE DYNAMIC RUNTIME ATTRIBUTE command (see 5.2.45).
These commands are used to retrieve and store information in the form of dynamic runtime attributes.
A DRA is represented in the layout described in 6.2.1.

There are three types of DRA attributes (see table 25).

| Attribute Type | Focus | Attribute Source | Readable with READ DYNAMIC RUNTIME ATTRIBUTE | Writable with WRITE DYNAMIC RUNTIME ATTRIBUTE | Reference |
|----------------|-------|-----------------|----------------------------------------------|-----------------------------------------------|-----------|
| Logical unit | Device | Set by the device server. | Yes | No | 6.2.2.1 |
| Target | I_T nexus | Set by the device server. | Yes | No | 6.2.2.4 |
| Initiator | I_T nexus | Set by the application client | Yes | Yes | 6.2.2.5 |

DRA attributes have the states shown in table 26.

| Attribute State | Description |
|----------------|-------------|
| Read Only | An application client may read the contents of the DRA attribute with the READ DYNAMIC RUNTIME ATTRIBUTE command, but an attempt to clear or change the DRA attribute using the WRITE DYNAMIC RUNTIME ATTRIBUTE command shall result in the command being terminated with CHECK CONDITION status with the sense key set to DATA PROTECT and the additional sense code set to WRITE PROTECTED. When in the read only state the READ ONLY bit (see 6.2.1) is one. |
| Unsupported | The device server does not support the DRA attribute and shall not return it in response to a READ DYNAMIC RUNTIME ATTRIBUTE command. |
| Nonexistent | An initiator attribute does not exist in the dynamic runtime attributes until a WRITE DYNAMIC RUNTIME ATTRIBUTE command creates it. |
| Read/Write | The DRA attribute has been created using the WRITE DYNAMIC RUNTIME ATTRIBUTE command. After the DRA attribute has been created, the contents may be altered using subsequent WRITE DYNAMIC RUNTIME ATTRIBUTE commands. A Read/Write DRA attribute may be returned to the nonexistent state using a WRITE DYNAMIC RUNTIME ATTRIBUTE command with the attribute length set to zero. When in the Read/Write state the READ ONLY bit (see 6.2.1) is zero. |

### 4.18.2 Dynamic runtime information timestamp

Some dynamic runtime attributes have a timestamp associated with them. The timestamp used is described in
Device Clocks (see 4.17 on page 61). If no timestamp is set by either a SCSI command (i.e., SET TIMESTAMP
or ) or by the library, then the timestamp is power-on time and may not be able to be correlated to external logs
(e.g., device driver logs, application logs).

### 4.18.3 Setting dynamic runtime information into the drive

An application client may set attributes into the drive using the WRITE DYNAMIC RUNTIME ATTRIBUTE
command (see 5.2.45) to set one or more of the initiator type attributes defined in 6.2.2.5. The application client
may write these values at any time and may change these values at any time. If an application client attempts to
create a new attribute by writing an attribute that was previously in the non-existent state and the device server
does not have the resources necessary to create that attribute the device server shall reject the command with a
CHECK CONDITION with the sense code set to ILLEGAL REQUEST and the additional sense code set to
INSUFFICIENT RESOURCES (i.e., 5h / 5503h).

### 4.18.4 Retrieving dynamic runtime information from the drive

An application client may read attributes by using the READ DYNAMIC RUNTIME ATTRIBUTE command
(see 5.2.19). The application client may request a single attribute or multiple attributes in a single command. The
application client may read any existent attribute (see 6.2).

### 4.18.5 Management of dynamic runtime information

Dynamic runtime attributes have either a focus of a logical unit (i.e., logical unit type attributes) or a focus of I_T
nexus (i.e., target type attributes and initiator type attributes). This relationship is shown in figure 8.

```
                                            Logical Unit Attributes



               I_T nexus                              I_T nexus                            I_T nexus



            Target Attributes                     Target Attributes                    Target Attributes



           Initiator Attributes                   Initiator Attributes                 Initiator Attributes
```

*Figure 8 -- Dynamic runtime attributes focus*

For each dynamic runtime attribute (see 6.2) that the device server supports, if a command is received that
should cause an update of one or more of the dynamic runtime attributes (e.g., Reserve, Persistent Reserve Out,
Prevent/Allow Medium Removal), then the device server shall update that dynamic runtime attribute. If one or
more of the Initiator type attributes that are supposed to be used to update the dynamic runtime attribute is in the
nonexistent state, then all information that is known is used. The TransportID of the I_T_L nexus and the target
port identifier of the I_T_L nexus is always known from the transport layer and are presented as target type
attributes.

#### 4.18.5.1 Dynamic Runtime Information Lifetime

Dynamic Runtime Attributes are maintained separate from the device server's management of I_T nexuses. The
I_T_L nexus identifying information (see 6.2.2.1) remains in existence inside dynamic runtime attributes even
after the I_T nexus referenced is no longer known by the SCSI target port.
Logical unit type attributes (see 6.2.2.1) are created if an event occurs as described in the description of each
logical unit type attribute and are destroyed if an event occurs as described in the description of each specific
logical unit type attribute.

Target type attributes (see 6.2.2.4) are created by the drive if it detects communication from a new I_T nexus
(e.g., Fibre Channel PLOGI/PRLI sequence) and are destroyed by the drive if it detects the disappearance of the
I_T nexus (e.g., Fibre Channel LOGO, I_T nexus resources are released to allow a new I_T nexus to
communicate with the drive).
Initiator type attributes (see 6.2.2.5) are created if a WRITE DYNAMIC RUNTIME ATTRIBUTE command is
received requesting a new attribute and are destroyed if the drive detects the destruction of the target type
attribute associated with the I_T nexus that created the Initiator type attribute or the drive processes a WRITE
DYNAMIC RUNTIME ATTRIBUTE command specifying that attribute with a length of zero..
Dynamic Runtime Attributes do not persist across a device power off.


## 4.19 Error Information

### 4.19.1 Sense Data

For a description of Sense data, see 5.2.31.1--Sense Data Layout.

### 4.19.2 Sense Data Management

Sense data returned by the device contains one of two types of errors. These errors are:

| Type | Description |
|------|-------------|
| Current | The error condition associated with the command that is currently being processed (i.e., SCSI Status for the currently processing command is the status being returned); and |
| Deferred | The error condition resulting from a command that has been reported as GOOD, but has generated sense data after being reported. This may be a command with the Immediate bit set or may be a buffered write. |

Sense data returned is described by the Sense Key (i.e., bits 3-0 of byte 2 of Sense data). Commands that
terminate in an error generate Sense data and set the Sense Key depending on the the specifics of the error.
Table 30 --Supported Common SCSI Commands, indicates which commands are allowed to be processed in
the presence of specific error conditions and which return an error.
This device communicates on transports that use the autosense protocol. This means that any Sense data
generated for return to a command is returned with the SCSI status. Once a particular set of sense data has
been returned, that sense data is cleared and a REQUEST SENSE command is not required to be issued to
collect the Sense data. Any other sense data that is still pending may still cause CHECK CONDITION status for
subsequent commands. When a REQUEST SENSE command is received, typically the only Sense data
available is the default Sense data. While it is possible that a Deferred error may have generated Sense data or
that a Unit Attention (see 4.19.4) has been established since the status to the last command, Sense data is not
likely to exist.

### 4.19.3 Deferred Check Condition (DCC)

Deferred errors are generated by processing that occurs when that process is not attached to the currently
processing command (see 4.19.2). Deferred errors are reported as sense data to a deferred check condition
(DCC) eligible command (i.e., DCC column of table 30 on page 76 is set to 'Y').
In the case of a deferred write error if buffered mode 1h is selected and a DCC eligible command is received,
then the error is reported to the SCSI initiator device (i.e., I_T nexus) that has deferred error affinity.
If the drive receives a deferred error affinity command (i.e., DEA column of table 30 on page 76 is set to 'Y'), then
the drive performs actions in the following order:
1. performs initial checking (e.g., Reservation Conflict, all pending Unit Attentions, and all pending errors to
   be reported to this I_T nexus) and reports these conditions, if any;
2. if none of the above conditions are reported, then all pending deferred errors are migrated to the I_T
   nexus through which this command was received;
3. the deferred error affinity is set to this I_T nexus;
4. if the command is DCC eligible, then pending deferred errors, if any, are reported; and
5. if no deferred errors were reported process the command.

### 4.19.4 Unit Attention Conditions

The drive generates a Unit Attention condition under the following circumstances:
- Reset condition (for example, power-on, SCSI reset, bus device reset);
- Tape Loaded condition (for example, media inserted, LOAD command from another initiator);
- Mode parameters changed by another initiator; and
- Drive firmware has been upgraded.

The drive only maintains one instance of each type of Unit Attention condition at any one time for any one
initiator. If a subsequent Unit Attention condition of the same type is generated, it replaces the existing one. Unit
Attentions are returned in priority order. The priorities are in the order listed above, with a reset being highest
priority and a firmware upgrade being lowest priority.

### 4.19.5 Persistent Errors

When errors occur that prevent tape operation, they are reported persistently until the problem is cleared. For
medium-related errors (usually reported with a Sense Key of 3), the error is reported until the cartridge is
successfully unloaded. For hardware-related errors (usually reported with a Sense Key of 4), the error is reported
until the drive successfully performs a power-on self test. These persistent errors are only reported on those
commands that are eligible for deferred Check Condition reporting (see table 30 on page 76). The error may or
may not be reported as Deferred.

#### 4.19.5.1 Fencing Behavior

The device fences the drive (i.e., prevents certain operations) when errors are detected that could endanger
customer data if further usage is allowed. The operations that are prevented depend on the nature of the error
encountered and the current drive state. The drive will post an FSC (see bytes 16 and 17 of REQUEST SENSE
data in Sense Data Layout (see 5.2.31.1 on page 162)) for the original error that caused the fence condition.
Then, CHECK CONDITION status, with the fencing FSC in the sense data is reported to an attempted command
that is not allowed due to the fence condition.
Table 27 lists which errors trigger which fence state.

| Error that triggers Fence State | Fence State |
|---------------------------------|-------------|
| Severe Drive Hardware problem | ALLOW_NO_OPERATION (see 4.19.5.1.1 on page 67) |
| Severe Media Hardware problem | ALLOW_NO_OPERATION (see 4.19.5.1.1 on page 67) |
| Temperature Overrange | ALLOW_NO_OPERATION (see 4.19.5.1.1 on page 67) |
| Load or Unload Hardware problem | ALLOW_NO_OPERATION (see 4.19.5.1.1 on page 67) |
| Severe Firmware Problem | ALLOW_NO_OPERATION (see 4.19.5.1.1 on page 67) |
| Hardware Problem detected that could affect Writing | ALLOW_LOCATE (see 4.19.5.1.2 on page 67) |
| Hardware Problem detected that could affect Reading | ALLOW_LOCATE (see 4.19.5.1.2 on page 67) |
| Serious Drive Hardware problem -- May be recovered on a different mount | ALLOW_UNLOAD (see 4.19.5.1.3 on page 67) |
| Serious Media problem -- Drive may be recovered on different mount | ALLOW_UNLOAD (see 4.19.5.1.3 on page 67) |
| Serious Firmware problem -- May be recovered on different mount | ALLOW_UNLOAD (see 4.19.5.1.3 on page 67) |
| Power on occurs and the device detects a volume is loaded. This may be due to a device panic/exception. | MID-TAPE RECOVERY (see 4.19.5.1.4 on page 67) |

##### 4.19.5.1.1 ALLOW_NO_OPERATION

- All medium access commands (Read/Write/Motion) are rejected.
- (SCSI/Panel/LDI) Unload is accepted.
- After the cartridge is ejected:
  - When load is attempted, the cartridge stays at mount position and Good status is returned for TUR.
  - From the above state, the cartridge can be ejected normally.
  - Other medium access commands are rejected.

##### 4.19.5.1.2 ALLOW_LOCATE

- All medium access commands (Read/Write/Implicit Motion) except explicit positioning command (i.e.,
  LOCATE, REWIND, LOAD) are rejected.
- (SCSI/Panel/LDI) Unload is accepted.
- Once a cartridge is ejected, Fence state is cleared. A new cartridge is allowed to be loaded and all
  medium access commands are allowed to be performed.
- Space command is rejected while in Fence state.

##### 4.19.5.1.3 ALLOW_UNLOAD

- All medium access commands (Read/Write/Motion) are rejected.
- (SCSI/Panel/LDI) Unload is accepted.
- Once a cartridge is ejected, Fence state is cleared. A new cartridge is allowed to be loaded and all
  medium access commands are allowed to be performed.

##### 4.19.5.1.4 MID-TAPE RECOVERY

The Mid-Tape Recovery (MTR) fence behavior is configured in MP 2Fh: Behavior Configuration (see 6.6.20 on
page 401). There are two different behaviors:
- Normal operation (i.e., MTR Fence) (see 4.19.5.1.4.1 on page 67); and
- Panic Fence operation (see 4.19.5.1.4.2 on page 68).

###### 4.19.5.1.4.1 Normal operation (i.e., MTR Fence)

When the device powers up and no cartridge is detected, no special behavior is required and the device:
1. responds to the first UNIT ATTENTION eligible command with 6/2900;
2. responds to a CHECK CONDITION eligible command, if any, with 2/3E00 during POST; and
3. enters normal operation when POST is completed successfully.

When the device powers up and detects a mounted cartridge, Mid-Tape Recovery (MTR) is required and the
device:
1. responds to the first UNIT ATTENTION eligible command with 6/2900;
2. responds to a CHECK CONDITION eligible command, if any, with 2/3E00 until POST is complete;
3. responds to a CHECK CONDITION eligible command, if any, with 2/0401 during MTR/Unload;
4. responds to a CHECK CONDITION eligible command, if any, with 2/0401 during MTR/Load; and then
5. enters the MTR Fence State when the MTR/Load has completed. In the MTR Fence State, the device:
   - responds to the first UNIT ATTENTION eligible command received after entering the MTR Fence
     State (i.e., after cartridge is loaded) with 6/2800;
   - responds to TUR commands, if any, with GOOD status;
   - responds to any medium access command not listed in the next step, if any, with 5/2C00; and
   - exits the MTR Fence State if an explicit positioning command completes successfully (i.e., LOCATE,
     REWIND, LOAD).

###### 4.19.5.1.4.2 Panic Fence operation

When the device powers up after a Panic or Exception and no cartridge is detected the device:
1. responds to the first UNIT ATTENTION eligible command with 6/2900;
2. responds to a CHECK CONDITION eligible command, if any, with 2/3E00 during POST; and
3. when POST is complete, enters into the Panic Fence state. In the Panic Fence state the device:
   - rejects SCSI commands other than RSNS/INQ/RLUNs/Read Buffer with 5/2904 sense, indicating
     Panic Fence state;
   - rejects a TUR command with 5/2904;
   - rejects attempts to Load a cartridge through any means;
   - accepts, at any time, a SCSI Read Buffer to read dump data;
   - exits the Panic Fence state after a dump has been successfully read and transitions to normal mode.

When the device powers up and detects a mounted cartridge, Mid-Tape Recovery (MTR) is required and the
device:
1. responds to the first UNIT ATTENTION eligible command with 6/2900;
2. responds to a CHECK CONDITION eligible command, if any, with 2/3E00 until POST is complete;
3. responds to a CHECK CONDITION eligible command, if any, with 2/0401 during MTR/Unload;
4. responds to a CHECK CONDITION eligible command, if any, with 2/0401 during MTR/Load; and then
5. enters the Panic Fence State when the MTR/Load has completed. In the Panic Fence state the device:
   - responds to the first UNIT ATTENTION eligible command received after entering the Panic Fence
     State (i.e., after cartridge is loaded) with 6/2800;
   - rejects SCSI commands other than RSNS/INQ/RLUNs/Read Buffer with 5/2904 sense, indicating
     Panic Fence state;
   - returns GOOD status to a TUR command while cartridge is loaded;
   - rejects a TUR command with 5/2904;
   - allows processing of Unload command through SCSI/LDI/Button;
   - rejects attempts to Load a cartridge through any means;
   - accepts, at any time, a SCSI Read Buffer to read dump data;
   - exits the Panic Fence state after a dump has been successfully read and:
     - if there is no cartridge loaded, then transitions to normal mode; or
     - if there is a cartridge loaded, then transitions to the MTR Fence State. In the MTR Fence State,
       the device:
       - responds to TUR commands, if any, with GOOD status;
       - responds to any medium access command not listed in the next step, if any, with 5/2C00;
         and
       - exits the MTR Fence State if an explicit positioning command completes successfully (i.e.,
         LOCATE, REWIND, LOAD).


## 4.20 Medium auxiliary memory

Some types of media, especially removable media, include a non-volatile memory referred to as MAM (medium
auxiliary memory). Medium auxiliary memory is used to store data describing the media and its contents. This
standard supports medium auxiliary memory with the READ ATTRIBUTE command (see 5.2.16) and the WRITE
ATTRIBUTE command (see 5.2.43). These commands are used to retrieve and store information in the medium
auxiliary memory in the form of MAM attributes.
A MAM attribute is represented in the layout described in MAM attribute layout (see 6.5.1 on page 351).

There are three types of MAM attributes (see table 28).

| Attribute Type | Attribute Source | Example | Readable with READ ATTRIBUTE | Writable with WRITE ATTRIBUTE |
|----------------|-----------------|---------|------------------------------|-------------------------------|
| Medium | Permanently stored in the medium auxiliary memory during manufacture. | Media Serial Number | Yes | No |
| Device | Maintained by the device server. | Load Count | Yes | No |
| Host | Maintained by the application client. | Backup Date | Yes | Yes |

Depending on that attribute type, MAM attributes have the states shown in table 29.

| Attribute Type | Attribute State | Description |
|----------------|----------------|-------------|
| Medium or Device | Read Only | An application client may read the contents of the MAM attribute with the READ ATTRIBUTE command, but an attempt to clear or change the MAM attribute using the WRITE ATTRIBUTE command shall result in the command being terminated with CHECK CONDITION status. When the READ ONLY bit (see 6.5.1) is one, the attribute is in the read only state. |
| Medium or Device | Unsupported | The device server does not support the MAM attribute and shall not return it in response to a READ ATTRIBUTE command. |
| Host | Nonexistent | A host attribute does not exist in the medium auxiliary memory until a WRITE ATTRIBUTE command creates it. |
| Host | Read/Write | The MAM attribute has been created using the WRITE ATTRIBUTE command. After the MAM attribute has been created, the contents may be altered using subsequent WRITE ATTRIBUTE commands. A read/write MAM attribute may be returned to the nonexistent state using a WRITE ATTRIBUTE command with the attribute length set to zero. When the READ ONLY bit (see 6.5.1) is zero, the MAM attribute is in the read/write state. |


## 4.21 Volume Coherency

An application client may need to be able to determine if all logical objects on a volume are coherent with the last
time an application client wrote to this volume. The VOLUME COHERENCY INFORMATION attribute
(see 6.5.2.4.11) of MAM is provided for an application client to collect and save information necessary for this
determination.
The VOLUME COHERENCY INFORMATION attribute for each partition is written to MAM by the application
client when it has completed a write job (e.g., the volume is demounted). The VOLUME COHERENCY
INFORMATION attribute contains references to a volume coherency set that the application client has written to
logical objects on a partition. An application client should not create a VOLUME COHERENCY INFORMATION
attribute unless it has written a volume coherency set to that partition. The volume coherency set shall include a
volume coherency count. The application client shall maintain one volume coherency count for an entire volume
and shall monotonically increase the volume coherency count when the state of the volume coherency set
changes (e.g., writing identical volume coherency sets on each partition does not force a change of volume
coherency count). When the application client writes the VOLUME COHERENCY INFORMATION attribute to
MAM for a specific partition the VOLUME CHANGE REFERENCE VALUE field of the VOLUME COHERENCY
INFORMATION attribute for a partition shall contain the value returned in the ATTRIBUTE VALUE field of the
VOLUME CHANGE REFERENCE attribute after the last volume coherency set was written to the volume. The
VOLUME COHERENCY COUNT field of the VOLUME COHERENCY INFORMATION attribute shall contain the
volume coherency count that was written to the last volume coherency set written to that partition. The VOLUME
COHERENCY SET IDENTIFIER field of the VOLUME COHERENCY INFORMATION attribute for a partition contains
the logical object identifier of the first byte of the last volume coherency set written to that partition. The
APPLICATION CLIENT SPECIFIC INFORMATION field of the VOLUME COHERENCY INFORMATION attribute for a
partition contains information the application client binds with the coherency set referenced by the VOLUME
COHERENCY SET IDENTIFIER field.

> **NOTE 7** -- The application client needs to guarantee that no other application client updates the logical objects on the volume between the time it completes writing and the time it updates the MAM parameter (e.g., use reservations)

An application client may verify that the volume coherency set written in a partition has not changed since the
VOLUME COHERENCY INFORMATION attribute was written when the application client reads the VOLUME
COHERENCY INFORMATION attribute for a partition (e.g., when a volume is mounted) and compares the value
in the VOLUME CHANGE REFERENCE VALUE field with the value returned in the ATTRIBUTE VALUE field of the
VOLUME CHANGE REFERENCE attribute. If the values match, then the volume coherency set written in that
partition is unchanged.
To find the most recently written volume coherency set, the application client searches the VOLUME
COHERENCY INFORMATION attributes of the partitions for which the volume coherency set is unchanged and
finds the largest value in the VOLUME COHERENCY COUNT field. The application client then verifies the largest value
in the VOLUME COHERENCY COUNT field with the volume coherency count stored in the volume coherency set
beginning at the logical object specified by the VOLUME COHERENCY SET IDENTIFIER field. If this matches, then this
is the volume coherency set that was most recently written.
The APPLICATION CLIENT SPECIFIC INFORMATION field may also be used by the application client as part of this
coherency check. If the information verifies for a partition, then the volume is coherent with the last access by this
application. If the information does not verify for a partition, then the volume is not coherent with the last access
by this application.


## 4.22 Error history (i.e., drive dump)

### 4.22.1 Error history overview

Error history is data collected by a device to aid in troubleshooting errors.
The READ BUFFER command (see 5.2.18) provides a method for retrieving error history from the device
(see 4.22.2).
All Ultrium devices support retrieving a drive dump using data mode (i.e., 02h) with buffer ID 01h. This drive
dump contains a snapshot of the current debug information (i.e., the contents of the operation tracing at a
specific point in time) as well as additional snapshots using development specific algorithms designed to provide
the best chance of capturing data to debug problems.
Error history may be retrieved using the method defined in SPC-4 and described in the rest of this clause. Note
that there are some areas which differ from the behavior specified in SPC-4.

### 4.22.2 Retrieving error history with the READ BUFFER command

The error history is retrieved using a sequence of READ BUFFER commands on one I_T_L nexus.
Tracing of drive operation is returned using error history snapshots. An error history snapshot is the contents of
the operation tracing at a specific point in time, created by the device at vendor specific times or requested by the
application client using the READ BUFFER command with certain buffer IDs.
The I_T_L nexus being used to retrieve an error history snapshot is called the error history I_T_L nexus. Only
one I_T_L nexus is allowed to retrieve an error history snapshot at a time.

To retrieve the complete error history, an application client uses one I_T_L nexus to:
1. create an error history snapshot if one does not already exist, establish the I_T_L nexus as the error
   history I_T_L nexus, and retrieve the drive tracing directory by sending a READ BUFFER command
   (see 5.2.18) with:
   - the MODE field set to 1Ch (i.e., error history);
   - the BUFFER ID field set to one of the following:
     - If the error history I_T_L nexus is expected to be valid:
       - 00h (i.e., return error history directory);
       - 01h (i.e., return error history directory and create new snapshot);
     - if the application client has knowledge that the error history I_T_L nexus is no longer valid:
       - 02h (i.e., return error history directory and establish new error history I_T_L nexus); or
       - 03h (i.e., return error history directory, establish new error history I_T_L nexus, and create
         new snapshot);
   - the BUFFER OFFSET field set to 000000h; and
   - the ALLOCATION LENGTH field set to at least 2 088 (i.e., large enough to transfer the complete error
     history directory (see 6.7.2.8.1));
2. retrieve the error history. The application client uses a Data-In Buffer size that is a multiple of the offset
   boundary indicated in the READ BUFFER descriptor (see 6.7.1.3). Each buffer ID indicated in the error
   history directory is a different type of trace or error history. Buffer ID EFh contains a description of each
   trace (i.e., error history) that is available (see 6.7.2.8.3.6). For each buffer ID indicated in the error history
   directory in the range of 10h to EFh, the application client may retrieve the trace by sending one or more
   READ BUFFER commands (see 5.2.18) as follows:
   1. send the first READ BUFFER command with:
      - the MODE field set to 1Ch (i.e., error history);
      - the BUFFER ID field set to the buffer ID (i.e., an error history data buffer);
      - the BUFFER OFFSET field set to 000000h; and
      - the ALLOCATION LENGTH field set to the size of the Data-In Buffer;
   2. until the number of bytes returned by the previous READ BUFFER command does not equal the
      specified allocation length and/or the total number of bytes returned from the buffer ID equals the
      maximum available length indicated in the error history directory, send zero or more additional READ
      BUFFER commands with:
      - the MODE field set to 1Ch (i.e., error history);
      - the BUFFER ID field set to the buffer ID (i.e., an error history data buffer);
      - the BUFFER OFFSET field set to the previous buffer offset plus the previous allocation length; and
      - the ALLOCATION LENGTH field set to the size of the Data-In Buffer;
   and
3. clear the error history I_T_L nexus and, depending on the buffer ID, release the error history snapshot by
   sending a READ BUFFER command with:
   - the MODE field set to 1Ch (i.e., error history);
   - the BUFFER ID field set to:
     - FEh (i.e., clear error history I_T_L nexus) (see 6.7.2.8.4); or
     - FFh (i.e., clear error history I_T_L nexus and release snapshot) (see 6.7.2.8.5);
   - the BUFFER OFFSET field set to any value allowed by table 378 (e.g., 000000h); and
   - the ALLOCATION LENGTH field set to any value allowed for the chosen BUFFER ID field value (see
     6.7.2.8.4 or 6.7.2.8.5) (e.g., 000000h).

While an error history snapshot exists, the device does not modify the error history snapshot to reflect any
changes to the error history. This does not include the emergency dump (see 6.7.2.8.3.4) or prioritized flash
dumps (see 6.7.2.8.3.5). These dumps are generated internally and may be generated or modified at any time,
even while an error history snapshot exists.
The device clears the established error history I_T_L nexus and does not release the error history snapshot:
- upon processing of a READ BUFFER command on the error history I_T_L nexus with:
  - the MODE field set to 1Ch (i.e., error history); and
  - the BUFFER ID field set to FEh (i.e., clear error history I_T_L nexus) (see 6.7.2.8.4);
  or
- if an I_T nexus loss occurs on the error history I_T_L nexus.

The device clears the established error history I_T_L nexus and releases the error history snapshot:
- upon processing of a READ BUFFER command using the same I_T_L nexus that was used to establish
  the snapshot with:
  - the MODE field set to 1Ch (i.e., error history); and
  - the BUFFER ID field set to FFh (i.e., clear error history I_T_L nexus and release snapshot)
    (see 6.7.2.8.5);
- if a power on occurs;
- if a hard reset occurs; or
- if a device reset occurs.

If a new error history snapshot is created by one of the supported methods or by internal algorithms while an
error history snapshot exists, the new snapshot overwrites the existing error history snapshot (i.e., MODE [1Ch]
10h: Current error history snapshot (see 6.7.2.8.3.2 on page 440)) or drive dump (i.e., Buffer ID 01h with MODE
set to 2h), then an attempt to read an error history at a non-zero offset is rejected with a CHECK CONDITION
with the sense key set to ILLEGAL REQUEST and the additional sense code set to ERROR HISTORY
SNAPSHOT RELEASED. This notifies the application that the snapshot (i.e., buffer ID) being retrieved has been
overwritten.


## 4.23 Potential conflict list (LTO6 and later)

This device may maintain a potential conflict list. A potential conflict list is a list of entries describing I_T nexuses
and commands that have been received, where the operations requested by one I_T nexus may conflict with the
operations requested by a different I_T nexus (e.g. a rewind requested by one I_T nexus while a different I_T
nexus is requesting data transfers). The potential conflict list is reported in potential conflict list log parameters
(see 6.4.10.1.8).
A potential conflict list command is a command that:
- has an entry of Conflict under the Excl Access column of the commands that are allowed in the presence
  of various reservations table of the command standard in which that command is defined; and
- is not one of:
  - LOG SELECT;
  - PERSISTENT RESERVE IN;
  - PERSISTENT RESERVE OUT;
  - READ ATTRIBUTE;
  - RESERVE UNIT (see SPC-2);
  - SECURITY PROTOCOL IN;
  - SECURITY PROTOCOL OUT;
  - TEST UNIT READY; and
  - commands chosen for vendor-specific reasons.

This device maintains an owner_ITN variable that is the I_T nexus through which a PCL command was most
recently received by the RMC logical unit or the ADC logical unit.
The owner_ITN is set to NULL, the PCL_P bit of the extended very high frequency log parameter (see 6.4.10.1.3)
is set to zero, the potential conflict list log parameter(s) are destroyed (i.e., no longer exist; the response to a
LOG SENSE command does not return the parameter), and the value in the NUMBER OF POTENTIAL CONFLICT LIST
ENTRIES field of the potential conflict list entries present log parameter (see 6.4.10.1.7) is set to zero, if:

- a Hard Reset occurs; or
- a volume is inserted (i.e., the MPRSNT (medium present) bit of the VHF parameter data transitions from
  0b to 1b).

The owner_ITN is set to NULL on a reservation loss or a reservation preempt.
If a PCL command is received through an I_T nexus that is not the owner_ITN, the command is not terminated
with RESERVATION CONFLICT and:
- the owner_ITN is non-NULL; or
- the owner_ITN is NULL, there is no reservation holder, and the addressed LUN is not an ADC LUN,

then the DT device shall:
1. if that I_T nexus is not listed in one of the potential conflict list log parameter(s), then:
   - if all the potential conflict list log parameters supported by the DT device have been created, then
     manage the potential conflict list in a vendor specific manner (e.g., stop adding entries to the list or
     replace an existing entry); or
   - create a new potential conflict list entry for this I_T nexus in the potential conflict list and add the new
     entry to the list of potential conflict list log parameters in a vendor-specific order (e.g., entries in the
     potential conflict list log parameters may be reordered) with:
     - the TRANSPORTID field set to the TransportID (see SPC-4) of that I_T nexus;
     - the RELATIVE TARGET PORT IDENTIFIER field set to the relative target port (see SPC-4) of that I_T
       nexus;
     - all other fields set to zero; and
     - increment the value in the NUMBER OF POTENTIAL CONFLICT LIST ENTRIES field of the potential conflict
       list entries present log parameter (see 6.4.10.1.7);
2. select the potential conflict list log parameter with the TRANSPORTID field value and
   RELATIVE TARGET PORT IDENTIFIER field value that match the I_T nexus through which the PCL command
   was received and update the fields in that log parameter as follows:
   - increment the OWNER ITN COUNT field, if not saturated at its maximum value;
   - set the COMMAND OPERATION CODE field to the operation code of the command;
   - set the COMMAND SERVICE ACTION field to the service action, if any, of the command; and
   - set the OWNER ITN TIME field to the parameter data for a REPORT TIMESTAMP command addressed
     to the ADC device server;
3. set the owner_ITN to identify the I_T nexus through which the command was received; and
4. set the PCL_P bit of the extended very high frequency log parameter to one.


## 4.24 Environmental Conditions Thresholding (LTO9 and later)

This device reports current and historical limits of temperature and relative humidity in the Environmental
Reporting log page (see 6.4.9). In addition, this device monitors temperature and humidity and takes protective
actions such as:
- warning the application/library through TapeAlerts;
- fencing the drive from any medium access commands; and
- ejecting the volume and fencing load commands.

These protective actions are centered around a set of thresholds with built in hysteresis. When a threshold is
met, the drive performs the indicated action. When the environmental condition (i.e., temperature or humidity)
changes such that the threshold is no longer met, there is hysteresis built in prior to the resetting of that
threshold. This hysteresis is designed to provide a stable return to operation and may be substantially different
than the value of the threshold. These thresholds and reset points are managed by the drive.
Figure 9 shows an example of how the thresholds are set. This is only an example to describe the concept and
not intended to show the relative difference in values of the various thresholds. This example describes the
progression through the various thresholds.
1. The Normal Operation Condition range of temperature is where the drive is designed to operate. The
   drive operates normally in this range;
2. When the temperature rises towards an unsafe value, the TapeAlert Asserted threshold is crossed and
   the Drive Temperature TapeAlert (i.e., 0024h) trigger is activated. The drive asserts the Drive
   Temperature TapeAlert;
3. As the temperature continues to rise it passes through the Fence Commands threshold. When this
   occurs, the Fence Commands trigger is activated and the drive disallows commands determined to be
   unsafe in this condition (e.g., disallows medium access commands). These commands are rejected with
   a CHECK CONDITION with the sense key set to ABORTED COMMAND and the additional sense code
   set to WARNING - SPECIFIED TEMPERATURE EXCEEDED (i.e., B/0B01h);
4. As the temperature continues to rise, it passes through the Eject Volume threshold. When this occurs,
   the Eject Volume trigger is activated and the drive ejects the volume. An attempt to load the volume is
   rejected with a CHECK CONDITION with the sense key set to NOT READY and the additional sense
   code set to WARNING - SPECIFIED TEMPERATURE EXCEEDED (i.e., 2/0B01h);
5. If the temperature decreases, it eventually passes through the temperatures of each of the reset
   thresholds and each of the threshold triggers are reset.

*Figure 9 -- Example of temperature thresholds*
