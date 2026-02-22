# Annex E. Best Practices


## E.1. Overview

This Annex provides recommendations for best practices when using IBM Tape drives.


## E.2. Handling of Type M cartridges

Since LTO-2, the LTO industry has had one type of cartridge per generation. The cartridges are called Type A cartridges (see 2.1.254). LTO-8 introduced an alternate cartridge called a Type M cartridge (see 2.1.255). A Type M cartridge, called M8, is allowed to be created from an Ultrium 7 DATA cartridge (i.e., L7) that is new and has not been used yet. Once a cartridge is changed to an M8, it remains an M8. This creation happens on the first write. Whether an M8 cartridge gets created from a new L7 cartridge depends on how the drive is configured, usually by a library or support tool. Because of this, an application does not have a definitive way to know a priori if a new L7 cartridge that is mounted into an LTO-8 drive will remain an L7 cartridge or will be changed to an M8 cartridge. When an L7 cartridge is mounted into an LTO-8 drive, the REPORT DENSITY SUPPORT - 44h (see 5.2.26 on page 138) command with the MEDIA bit set to 1b shows the default density the drive is configured to write to the cartridge. However, the best practice for determining definitively whether a new cartridge becomes an L7 or an M8, during the ingest and mounting of a new L7 cartridge, is to follow this ordered sequence:

1. mount the cartridge;
2. perform either:
   A) a FORMAT MEDIUM command; or
   B) a short erase (i.e., ERASE with LONG = 0b);
3. perform a REPORT DENSITY SUPPORT command with the MEDIA bit set to 1b and examine the density code; and
4. update the cartridge type in the application.


## E.3. LTO-9 Cartridge Optimization

### E.3.1. LTO-9 cartridge optimization overview

The cartridge initialization in LTO-9 performed on L9 and LZ media, also performs a media optimization. See 4.1--Media Optimization (LTO9) for a description.

### E.3.2. Usage recommendations

- **Timing**: The best practice as to when a cartridge is optimized is, as stated in 4.1--Media Optimization (LTO9), to allow the drive to automatically perform media optimization in the location in which it is to be deployed. This environment should be stable and meet the recommended environmental specification. Media optimization averages 20 minutes per first load of a cartridge to a tape drive. Although most optimizations will complete within 30 minutes some optimizations may take up to 2 hours. Interruption of the process is not recommended; a different mount will not usually improve the time to complete the one-time optimization.

- **Software Application Information**: As a best practice, the following items are recommendations for software applications:
  - A) Update SCSI command timeout values by utilizing command timeout descriptors in the REPORT SUPPORTED OPERATION CODES - A3h[0Ch] (see 5.2.28 on page 143) command. This value is valid for both a blocking LOAD command on first load and as the time to continue polling for completion of any load of the cartridge during media optimization.
  - B) While waiting for device readiness (e.g., completion of an asynchronous load (e.g. MOVE MEDIUM, manual insertion)) use TUR polling for the duration of the load.
  - C) If a clean juncture is desired to allow access to MAM prior to beginning the optimization, then load the cartridge using:
    1. a LOAD command with HOLD=1 (LOAD=1 (ignored)) IMMED=0;
    2. perform needed operations; then
    3. issue LOAD command with HOLD=0 LOAD=1 IMMED=1.
  - D) When issuing SCSI LOAD command, use the IMMED bit set to one and poll with TUR for command completion.
  - E) While media optimization is in process, the DT DEVICE ACTIVITY field of VHF data described in 6.4.10--LP 11h: DT Device Status indicates "CALIBRATING".
  - F) When issuing the FORMAT MEDIUM - 04h (see 5.2.3 on page 80) command with FORMAT Field = 0 or 2, use the IMMED bit set to one and poll unsolicited sense for command completion.
  - G) There are no new failure conditions attributed to media optimization. There is no new additional sense code (i.e., ASC/ASCQ) specific to media optimization.

- **Drive Status Indicator**: During the media optimization operation, the drive provides external indication via the Single Character Display (SCD). The SCD displays 'c' (a lower case c) and the green LED is blinking at a 1 Hz interval.

### E.3.3. SCSI command additions / updates

General differences in command and parameter differences between generations are listed in A.2.--Command and Parameter Differences Between Generations. This clause is specifically related to LTO-9 cartridge initialization.

- **UPDATE to FORMAT MEDIUM - 04h** (see 5.2.3 on page 80) command: In certain instances, the FORMAT MEDIUM command also performs media optimization. Use of the FORMAT MEDIUM command for partitioning using the FORMAT field set to 1h does not perform media optimization.

- **UPDATE to REPORT SUPPORTED OPERATION CODES - A3h[0Ch]** (see 5.2.28 on page 143): As part of the RSOC timeout values, due to additional time required for cartridge initialization, information for LOAD timeout has been updated. This time may be used to know how long to allow polling before calling out an error, as well as for a blocking LOAD command and a blocking FORMAT MEDIUM command.

- **NEW MAM Attributes**: MEDIUM OPTIMIZATION VERSION and MEDIUM OPTIMIZATION NEEDED are added to the Vendor-Specific Medium Type Attributes (see 6.5.2.5 on page 360) MAM attributes. The current state of whether characterization occurs on the next cartridge LOAD, is given by the MEDIUM OPTIMIZATION NEEDED {MAM 1010h} (see 6.5.2.5.4 on page 360) MAM attribute:
  - Upon successful completion of a cartridge optimization, the attribute is set automatically by the drive to FALSE(0). This indicates that optimization does not occur on the next load.
  - To indicate optimization is needed on the next load, the attribute is set to TRUE(1). This can be set to TRUE(1) using the WRITE ATTRIBUTE command when the tape is empty. After setting the attribute, the cartridge must be unloaded prior to usage.
