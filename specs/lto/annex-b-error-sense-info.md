# Annex B. Error Sense Information

This annex lists all possible combinations of Sense Keys, Additional Sense Codes (ASC), and Additional Sense Code Qualifiers (ASCQ) that are reported by this device.

> NOTE 75 - ASCs of EEh and EFh are used for encryption related features and are only supported by encryption capable devices

> NOTE 76 - When using encryption enabled devices in an in-band configuration (System method or key path), ASCs of EFh are used to initiate a key management session via a system proxy.


## B.1. Sense Key 0 (No Sense)

### Table B.1 -- ASC, and ASCQ Summary for Sense Key 0 (No Sense)

| ASC ASCQ | Description |
|----------|-------------|
| 00 00 | NO ADDITIONAL SENSE INFORMATION - (UNSOLICITED, NO CA/CC) |
| 00 00 | NO ADDITIONAL SENSE INFORMATION - EOM=1B (EARLY WARNING) |
| 00 00 | NO ADDITIONAL SENSE INFORMATION - ILI=1B |
| 00 00 | NO ADDITIONAL SENSE INFORMATION - FM=1B |
| 00 01 | FILEMARK DETECTED |
| 00 02 | END-OF-PARTITION/MEDIUM DETECTED, EARLY WARNING |
| 00 04 | BEGINNING-OF-PARTITION/MEDIUM DETECTED |
| 00 07 | PROGRAMMABLE EARLY WARNING DETECTED |
| 00 16 | OPERATION IN PROGRESS |
| 00 18 | ERASE OPERATION IN PROGRESS |
| 00 19 | LOCATE OPERATION IN PROGRESS |
| 00 1C | VERIFY OPERATION IN PROGRESS |
| 5E 00 | Always replaced by 5E 07 |
| 5E 07 | IDLE_C CONDITION ACTIVATED BY TIMER (and LOW POWER CONDITION ON for any reason) |
| 82 82 | DRIVE REQUIRES CLEANING |
| EF 13 | ENCRYPTION - KEY TRANSLATE |

> Note - ASCs of EEh and EFh are used for encryption related features and are only supported by encryption capable devices

> Note - When using encryption enabled devices in an in-band configuration (System method or key path), ASCs of EFh are used to initiate a key management session via a system proxy.


## B.2. Sense Key 1 (Recovered Error)

### Table B.2 -- ASC, and ASCQ Summary for Sense Key 1 (Recovered Error)

| ASC ASCQ | Description |
|----------|-------------|
| 00 00 | NO ADDITIONAL SENSE INFORMATION: Inform test results; Various recovered exception conditions. |
| 00 17 | DRIVE NEEDS CLEANING |
| 0C 00 | WRITE ERROR: A write error occurred, but was recovered. Data was successfully written to tape. |
| 11 00 | READ ERROR: A read error occurred, but was recovered. Data was successfully read from tape. |
| 17 01 | RECOVERED DATA WITH RETRIES |
| 37 00 | ROUNDED PARAMETER |
| 5D 00 | FAILURE PREDICTION THRESHOLD EXCEEDED |
| 5D FF | FAILURE PREDICTION THRESHOLD EXCEEDED (FALSE) |
| 82 52 | DEGRADED MEDIA |
| 83 83 | DRIVE HAS BEEN CLEANED |

> Note - Many additional ASC ASCQ combinations are possible if recovered error reporting is enabled via Mode Select. Recovered Error Reporting Enabled is the default option with some of the device drivers.


## B.3. Sense Key 2 (Not Ready)

### Table B.3 -- ASC, and ASCQ Summary for Sense Key 2 (Not Ready)

| ASC ASCQ | Description |
|----------|-------------|
| 00 16 | OPERATION IN PROGRESS |
| 04 00 | LOGICAL UNIT NOT READY, CAUSE NOT REPORTABLE |
| 04 01 | LOGICAL UNIT IS IN PROCESS OF BECOMING READY |
| 04 02 | INITIALIZING COMMAND REQUIRED: A tape is present in the drive, but it is not logically loaded |
| 04 04 | NOT READY, FORMAT IN PROGRESS |
| 04 12 | LOGICAL UNIT NOT READY, OFFLINE |
| 04 13 | LOGICAL UNIT NOT READY, SA CREATION IN PROGRESS |
| 0B 01 | WARNING - SPECIFIED TEMPERATURE EXCEEDED: The drive is over temperature. Unload any volume and allow to cool. This should not happen and indicates either environmental conditions are outside allowed range, or there is a problem with this drive's airflow. If no other drives in this library are having temperature problems, then monitor this drive for any repeat temperature issues. If temperature issues repeat, then contact service |
| 30 03 | CLEANING IN PROGRESS |
| 30 07 | CLEANING FAILURE: An expired legacy cleaning cartridge was attempted to be used. Remove the cleaning cartridge |
| 3A 00 | MEDIUM NOT PRESENT |
| 3A 04 | NOT READY - MEDIUM AUXILIARY MEMORY ACCESSIBLE |
| 3E 00 | LOGICAL UNIT HAS NOT SELF-CONFIGURED |
| 53 00 | MEDIA LOAD OR EJECT FAILED |
| 74 11 | SA CREATION PARAMETER VALUE REJECTED |


## B.4. Sense Key 3 (Medium Error)

### Table B.4 -- ASC, and ASCQ Summary for Sense Key 3 (Medium Error)

| ASC ASCQ | Description |
|----------|-------------|
| 04 10 | LOGICAL UNIT NOT READY, AUXILIARY MEMORY NOT ACCESSIBLE |
| 09 00 | TRACK FOLLOWING ERROR |
| 0C 00 | WRITE ERROR |
| 11 00 | UNRECOVERED READ ERROR |
| 11 12 | AUXILIARY MEMORY READ ERROR |
| 14 00 | RECORDED ENTITY NOT FOUND |
| 30 00 | INCOMPATIBLE MEDIUM INSTALLED: The drive detected a problem with the CM or a WORM tape has been tampered with. Replace the volume |
| 30 01 | CANNOT READ MEDIUM, UNKNOWN FORMAT |
| 30 02 | CANNOT READ MEDIUM, INCOMPATIBLE FORMAT |
| 30 0D | WORM MEDIUM - TAMPERING DETECTED |
| 31 00 | MEDIUM FORMAT CORRUPTED |
| 3B 00 | SEQUENTIAL POSITIONING ERROR |
| 50 00 | WRITE APPEND ERROR |
| 51 00 | ERASE FAILURE |
| 52 00 | CARTRIDGE FAULT |
| 53 00 | MEDIA LOAD OR EJECT FAILED |
| 53 04 | MEDIUM THREAD OR UNTHREAD FAILURE |
| EE 60 | ENCRYPTION - PROXY COMMAND ERROR |
| EE D0 | ENCRYPTION - DATA READ DECRYPTION FAILURE |
| EE D1 | ENCRYPTION - DATA READ AFTER WRITE DECRYPTION FAILURE |
| EE E0 | ENCRYPTION - KEY TRANSLATION FAILURE |
| EE E1 | ENCRYPTION - KEY TRANSLATION AMBIGUOUS |
| EE F0 | ENCRYPTION - DECRYPTION FENCED (READ) |
| EE F1 | ENCRYPTION - ENCRYPTION FENCED (WRITE) |

> Note - ASCs of EEh and EFh are used for encryption related features and are only supported by encryption capable devices

> Note - When using encryption enabled devices in an in-band configuration (System method or key path), ASCs of EFh are used to initiate a key management session via a system proxy.


## B.5. Sense Key 4 (Hardware Error)

### Table B.5 -- ASC, and ASCQ Summary for Sense Key 4 (Hardware Error)

| ASC ASCQ | Description |
|----------|-------------|
| 03 02 | EXCESSIVE WRITE ERRORS: The drive is fenced. Contact Service |
| 04 03 | MANUAL INTERVENTION REQUIRED: A tape is present in the drive but could not be loaded or unloaded without manual intervention. Contact Service |
| 10 01 | LOGICAL BLOCK GUARD CHECK FAILED |
| 40 XX | DIAGNOSTIC FAILURE: The Additional Sense Code Qualifier (i.e., XX) indicates the failing component. Contact Service |
| 41 00 | DATA PATH FAILURE |
| 44 00 | INTERNAL TARGET FAILURE / Drive Needs Cleaning, Warning Threshold Exceeded |
| 51 00 | ERASE FAILURE |
| 52 00 | CARTRIDGE FAULT: a) during MTR the tape is detected to be too loose to safely continue. This is normally a high usage tape that is no longer attached to the leader pin; b) volume was prevented from initializing; or c) problem with the CM. |
| 53 00 | MEDIA LOAD OR EJECT FAILED: Contact Service. |
| 53 04 | MEDIUM THREAD OR UNTHREAD FAILURE |
| EE 0E | ENCRYPTION - KEY SERVICE TIME-OUT ^c^ |
| EE 0F | ENCRYPTION - KEY SERVICE FAILURE ^c^ |

> Note - ASCs of EEh and EFh are used for encryption related features and are only supported by encryption capable devices

> Note - When using encryption enabled devices in an in-band configuration (System method or key path), ASCs of EFh are used to initiate a key management session via a system proxy.

> ^c^ Returned in LTO5 and earlier


## B.6. Sense Key 5 (Illegal Request)

### Table B.6 -- ASC, and ASCQ Summary for Sense Key 5 (Illegal Request)

| ASC ASCQ | Description |
|----------|-------------|
| 00 16 | OPERATION IN PROGRESS |
| 0E 03 | INVALID FIELD IN COMMAND INFORMATION UNIT (e.g., FCP_DL error; mismatch between transport layer and application layer; likely device driver or HBA issue.) |
| 1A 00 | PARAMETER LIST LENGTH ERROR |
| 20 00 | INVALID COMMAND OPERATION CODE |
| 20 0C | ILLEGAL COMMAND WHEN NOT IN APPEND-ONLY MODE |
| 24 00 | INVALID FIELD IN CDB |
| 25 00 | LOGICAL UNIT NOT SUPPORTED |
| 26 00 | INVALID FIELD IN PARAMETER LIST |
| 26 02 | PARAMETER VALUE INVALID |
| 26 04 | INVALID RELEASE OF PERSISTENT RESERVATION |
| 26 11 | ENCRYPTION - INCOMPLETE KEY-ASSOCIATE DATA SET |
| 26 12 | VENDOR SPECIFIC KEY REFERENCE NOT FOUND |
| 29 04 | DEVICE INTERNAL RESET |
| 2A 0B | ERROR HISTORY SNAPSHOT RELEASED |
| 2C 00 | COMMAND SEQUENCE ERROR |
| 2C 0B | NOT RESERVED - The OIR bit of the Sequential Access Device page is set and the I_T nexus attempting to communicate with the drive does not hold a reservation. |
| 3B 00 | SEQUENTIAL POSITIONING ERROR |
| 3B 0C | POSITION PAST BEGINNING OF MEDIUM: A command that required the medium to be at BOP was attempted when the medium was not at BOP (for example, SET CAPACITY) |
| 49 00 | INVALID MESSAGE ERROR |
| 53 02 | MEDIUM REMOVAL PREVENTED |
| 53 06 | AUXILIARY MEMORY OUT OF SPACE |
| 55 08 | MAXIMUM NUMBER OF SUPPLEMENTAL DECRYPTION KEYS EXCEEDED |
| 74 08 | DIGITAL SIGNATURE VALIDATION FAILURE: The digital signature that signs this firmware image failed to validate even though the checksum passed. This is a security error. |
| 74 0C | UNABLE TO DECRYPT PARAMETER LIST |
| 74 0D | CRYPTO ALGORITHM DISABLED |
| 74 10 | SA CREATION PARAMETER VALUE INVALID |
| 74 11 | SA CREATION PARAMETER VALUE REJECTED |
| 74 12 | INVALID SA USAGE |
| 74 21 | CRYPTO CONFIGURATION PREVENTED |
| 74 30 | SA CREATION PARAMETER NOT SUPPORTED |
| 82 83 | BAD MICROCODE DETECTED: The data transferred to the drive during a firmware upgrade is corrupted or incompatible with the drive hardware |
| A3 01 | OEM Vendor-specific |
| EE 00 | ENCRYPTION - KEY SERVICE NOT ENABLED |
| EE 01 | ENCRYPTION - KEY SERVICE NOT CONFIGURED |
| EE 02 | ENCRYPTION - KEY SERVICE NOT AVAILABLE |
| EE 0D | ENCRYPTION - MESSAGE CONTENT ERROR |
| EE 10 | ENCRYPTION - KEY REQUIRED |
| EE 20 | ENCRYPTION - KEY COUNT EXCEEDED |
| EE 21 | ENCRYPTION - KEY ALIAS EXCEEDED |
| EE 22 | ENCRYPTION - KEY RESERVED |
| EE 23 | ENCRYPTION - KEY CONFLICT |
| EE 24 | ENCRYPTION - KEY METHOD CHANGE |
| EE 25 | ENCRYPTION - KEY FORMAT NOT SUPPORTED |
| EE 26 | ENCRYPTION - UNAUTHORIZED REQUEST - DAK |
| EE 27 | ENCRYPTION - UNAUTHORIZED REQUEST - DSK |
| EE 28 | ENCRYPTION - UNAUTHORIZED REQUEST - EAK |
| EE 29 | ENCRYPTION - AUTHENTICATION FAILURE |
| EE 2A | ENCRYPTION - INVALID RDKI |
| EE 2B | ENCRYPTION - KEY INCORRECT |
| EE 2C | ENCRYPTION - KEY WRAPPING FAILURE |
| EE 2D | ENCRYPTION - SEQUENCING FAILURE |
| EE 2E | ENCRYPTION - UNSUPPORTED TYPE |
| EE 2F | ENCRYPTION - NEW KEY ENCRYPTED WRITE PENDING |
| EE 30 | ENCRYPTION - PROHIBITED REQUEST |
| EE 31 | ENCRYPTION - KEY UNKNOWN |
| EE 32 | ENCRYPTION - UNAUTHORIZED REQUEST - dCERT |
| EE 42 | ENCRYPTION - EKM CHALLENGE PENDING |
| EE E2 | ENCRYPTION - KEY TRANSLATION DISALLOWED |
| EE FF | ENCRYPTION - SECURITY PROHIBITED FUNCTION |
| EF 01 | ENCRYPTION - KEY SERVICE NOT CONFIGURED |

> Note - ASCs of EEh and EFh are used for encryption related features and are only supported by encryption capable devices

> Note - When using encryption enabled devices in an in-band configuration (System method or key path), ASCs of EFh are used to initiate a key management session via a system proxy.


## B.7. Sense Key 6 (Unit Attention)

### Table B.7 -- ASC, and ASCQ Summary for Sense Key 6 (Unit Attention)

| ASC ASCQ | Description |
|----------|-------------|
| 28 00 | NOT READY TO READY TRANSITION, MEDIUM MAY HAVE CHANGED |
| 28 01 | LUN 1--IMPORT OR EXPORT ELEMENT ACCESSED |
| 29 00 | POWER ON, RESET, OR BUS DEVICE RESET OCCURRED |
| 29 01 | POWER ON OCCURRED |
| 29 03 | BUS DEVICE RESET FUNCTION OCCURRED |
| 29 04 | LUN 0--DEVICE INTERNAL RESET / LUN 1--library reset occurred on path to this drive |
| 29 05 | TRANSCEIVER MODE CHANGED TO SINGLE-ENDED |
| 29 06 | TRANSCEIVER MODE CHANGED TO LVD |
| 2A 00 | PARAMETERS CHANGED |
| 2A 01 | MODE PARAMETERS CHANGED |
| 2A 02 | LOG PARAMETERS CHANGED |
| 2A 03 | RESERVATIONS PREEMPTED |
| 2A 04 | RESERVATIONS RELEASED |
| 2A 05 | REGISTRATIONS PREEMPTED |
| 2A 0A | ERROR HISTORY I_T NEXUS CLEARED |
| 2A 10 | CRYPTO CAPABILITIES CHANGED |
| 2A 11 | ENCRYPTION - DATA ENCRYPTION PARAMETERS CHANGED BY ANOTHER I_T NEXUS |
| 2A 12 | ENCRYPTION - DATA ENCRYPTION PARAMETERS CHANGED BY VENDOR SPECIFIC EVENT |
| 2A 14 | SA CREATION CAPABILITIES DATA HAS CHANGED |
| 2F 00 | COMMANDS CLEARED BY ANOTHER INITIATOR |
| 3B 12 | LUN 1--MEDIUM MAGAZINE REMOVED |
| 3B 13 | LUN 1--MEDIUM MAGAZINE INSERTED |
| 3B 1A | LUN 1--DRIVE REMOVED |
| 3B 1B | LUN 1--DRIVE INSERTED |
| 3F 01 | MICROCODE HAS BEEN CHANGED |
| 3F 02 | CHANGED OPERATING DEFINITION |
| 3F 03 | INQUIRY DATA HAS CHANGED |
| 3F 05 | LUN 1--DEVICE IDENTIFIER CHANGED |
| 3F 0E | REPORTED LUNS DATA HAS CHANGED |
| 5D 00 | FAILURE PREDICTION THRESHOLD EXCEEDED |
| 5D FF | FAILURE PREDICTION THRESHOLD EXCEEDED (FALSE) |
| EE 11 | ENCRYPTION - KEY GENERATION |
| EE 12 | ENCRYPTION - KEY CHANGE DETECTED |
| EE 13 | ENCRYPTION - KEY TRANSLATION |
| EE 18 | ENCRYPTION - CHANGED (READ) |
| EE 19 | ENCRYPTION - CHANGED (WRITE) |
| EE 40 | ENCRYPTION - EKM IDENTIFIER CHANGED |
| EE 41 | ENCRYPTION - EKM CHALLENGE CHANGED |
| EE 50 | ENCRYPTION - INITIATOR IDENTIFIER CHANGED |
| EE 51 | ENCRYPTION - INITIATOR RESPONSE CHANGED |
| EF 01 | ENCRYPTION - KEY SERVICE NOT CONFIGURED |
| EF 10 | ENCRYPTION - KEY REQUIRED |
| EF 11 | ENCRYPTION - KEY GENERATION |
| EF 13 | ENCRYPTION - KEY TRANSLATION |
| EF 1A | ENCRYPTION - KEY OPTIONAL (i.e., chose encryption enabled/disabled) |

> Note - ASCs of EEh and EFh are used for encryption related features and are only supported by encryption capable devices

> Note - When using encryption enabled devices in an in-band configuration (System method or key path), ASCs of EFh are used to initiate a key management session via a system proxy.

> Note - A few LUN 1 Unit Attentions are also listed. These are not all inclusive. When there is a description for a LUN 1 ASC/ASCQ, the descriptions have LUN 1-- prepended. If there is also a LUN 0 ASC/ASCQ it has a LUN 0-- prepended.


## B.8. Sense Key 7 (Data Protect)

### Table B.8 -- ASC, and ASCQ Summary for Sense Key 7 (Data Protect)

| ASC ASCQ | Description |
|----------|-------------|
| 26 10 | ENCRYPTION - DATA DECRYPTION KEY FAIL LIMIT |
| 27 00 | WRITE PROTECTED: Volume is write protected, either by the cartridge write protect switch, problems with the CM, or attempted write in an incorrect diagnostic mode |
| 2A 13 | ENCRYPTION - DATA ENCRYPTION KEY INSTANCE COUNTER HAS CHANGED |
| 30 05 | CANNOT WRITE MEDIUM, INCOMPATIBLE FORMAT: Check to see if the cartridge is an uninitialized read-only generation cartridge |
| 30 06 | CANNOT FORMAT MEDIUM - INCOMPATIBLE MEDIUM |
| 30 0C | DATA PROTECT/WORM MEDIUM - OVERWRITE ATTEMPTED: Set when the drive rejects a Write operation because the rules for allowing WORM writes have not been met |
| 30 0D | DATA PROTECT/WORM MEDIUM - INTEGRITY CHECK: Set when the drive rejects a Write operation because the current cartridge is a Suspicious WORM cartridge |
| 50 01 | WRITE APPEND POSITION ERROR (WORM) |
| 52 00 | CARTRIDGE FAULT (invalid uninitialized cleaning cartridge--should never be seen): Bad cleaner cartridge. |
| 5A 02 | OPERATOR SELECTED WRITE PROTECT: Append-only mode is enabled and an attempt was made to write at a non-append point. |
| 74 00 | SECURITY ERROR |
| 74 01 | ENCRYPTION - UNABLE TO DECRYPT DATA |
| 74 02 | ENCRYPTION - UNENCRYPTED DATA ENCOUNTERED WHILE DECRYPTING |
| 74 03 | ENCRYPTION - INCORRECT DATA ENCRYPTION KEY |
| 74 04 | ENCRYPTION - CRYPTOGRAPHIC INTEGRITY VALIDATION FAILED |
| 74 05 | ENCRYPTION - ERROR DECRYPTING DATA |
| 74 06 | UNKNOWN SIGNATURE VERIFICATION KEY |
| 74 07 | ENCRYPTION PARAMETERS NOT USEABLE |
| 74 09 | ENCRYPTION MODE MISMATCH ON READ |
| 74 0A | ENCRYPTED BLOCK NOT RAW READ ENABLED |
| 74 0B | INCORRECT ENCRYPTION PARAMETERS |
| 74 6F | EXTERNAL DATA ENCRYPTION CONTROL ERROR |
| EE 0E | ENCRYPTION - KEY SERVICE TIME-OUT ^d^ |
| EE 0F | ENCRYPTION - KEY SERVICE FAILURE ^d^ |
| EF 10 | ENCRYPTION - KEY REQUIRED |
| EF 11 | ENCRYPTION - KEY GENERATION |
| EF 13 | ENCRYPTION - KEY TRANSLATE |
| EF 1A | ENCRYPTION - KEY OPTIONAL |
| EF A0 | ENCRYPTION - KEY REQUIRED (T10) |
| EF A1 | ENCRYPTION - KEY GENERATION (T10) |
| EF C0 | ENCRYPTION - NO OPERATION |

> Note - ASCs of EEh and EFh are used for encryption related features and are only supported by encryption capable devices

> Note - When using encryption enabled devices in an in-band configuration (System method or key path), ASCs of EFh are used to initiate a key management session via a system proxy.

> ^d^ Returned in LTO6 and later


## B.9. Sense Key 8 (Blank Check)

### Table B.9 -- ASC, and ASCQ Summary for Sense Key 8 (Blank Check)

| ASC ASCQ | Description |
|----------|-------------|
| 00 05 | END-OF-DATA DETECTED |


## B.10. Sense Key B (Aborted Command)

### Table B.10 -- ASC, and ASCQ Summary for Sense Key B (Aborted Command)

| ASC ASCQ | Description |
|----------|-------------|
| 00 1E | CONFLICTING SA CREATION REQUEST |
| 0B 01 | WARNING - SPECIFIED TEMPERATURE EXCEEDED: The drive is over temperature. Unload any volume and allow to cool. This should not happen and indicates either environmental conditions are outside allowed range, or there is a problem with this drive's airflow. If no other drives in this library are having temperature problems, then monitor this drive for any repeat temperature issues. If temperature issues repeat, then contact service |
| 11 0A | MISCORRECTED ERROR |
| 2C 00 | COMMAND SEQUENCE ERROR |
| 3D 00 | INVALID BITS IN IDENTIFY MESSAGE |
| 3F 0F | ECHO BUFFER OVERWRITTEN |
| 43 00 | MESSAGE ERROR |
| 45 00 | SELECT OR RESELECT FAILURE |
| 47 00 | SCSI PARITY ERROR |
| 47 03 | INFORMATION UNIT iuCRC ERROR DETECTED |
| 48 00 | INITIATOR DETECTED ERROR MESSAGE RECEIVED |
| 49 00 | INVALID MESSAGE ERROR |
| 4A 00 | COMMAND PHASE ERROR |
| 4B 00 | DATA PHASE ERROR |
| 4B 02 | TOO MUCH WRITE DATA |
| 4B 03 | ACK/NAK TIMEOUT |
| 4B 04 | NAK RECEIVED |
| 4B 05 | DATA OFFSET ERROR |
| 4B 06 | INITIATOR RESPONSE TIMEOUT |
| 4E 00 | OVERLAPPED COMMANDS ATTEMPTED |
| 74 40 | AUTHENTICATION FAILED |


## B.11. Sense Key D (Volume Overflow)

### Table B.11 -- ASC, and ASCQ Summary for Sense Key D (Volume Overflow)

| ASC ASCQ | Description |
|----------|-------------|
| 00 02 | END-OF-PARTITION/MEDIUM DETECTED |
