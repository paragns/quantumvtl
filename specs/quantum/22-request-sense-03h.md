# Request Sense - 03h

## What the Library Does With This Command

The library returns eighteen bytes of sense data to the requesting initiator. The data is preserved until
either the REQUEST SENSE command or any other command is received. The library can queue
multiple Unit Attentions for processing.


## Command Usage

This command should be issued whenever the initiator receives a CHECK CONDITION from the library. It
should continue to be issued until all check conditions have been cleared.


## Request Sense CDB Format

The REQUEST SENSE CDB format is shown in the following table.

**Table 1: REQUEST SENSE CDB format**

```
              Bit       7          6           5         4          3           2          1            0
  Byte
          0                                              Op Code (03h)

          1                                          Reserved                                        DESC
          2                                                  Reserved
          3                                                  Reserved
          4                                            Allocation Length
          5                                                  Control
```

- **DESC**: This field must be set to 0b, as the library returns only fixed format sense data per Request Sense Response below.
- **Allocation Length**: This field specifies the number of sense bytes requested by the initiator.


## Request Sense Response

**Table 2: Request Sense Response**

```
           Bit          7               6           5             4              3           2          1          0
  Byte
      0               Valid                                    Response Code (70h)
      1                                                       Reserved
                                                              SDAT_
      2           FILEMARK           EOM            ILI                                     Sense Key
                                                              OVFL
      3
       :                                                     Information
      6
      7                                     Additional Sense Length (0Ah or 10h)
      8
       :                              Command Specific Information (0000 0000h)
      11
      12                                        Additional Sense Code (ASC)
      13                                 Additional Sense Code Qualifier (ASCQ)
      14                                        Field Replaceable Unit Code
      15              SKSV            C/D                 Reserved            BPV                 Bit Pointer
      16
                                                            Field Pointer
      17
      18
       :                                                     Reserved
      23
```

- **Valid**: The Valid field is set to 0 if the Information field is not valid. It is set to 1 if the Information field contains valid additional data as described below.
- **Response Code**: The Response Code field is set to 70h to indicate that the library returns current errors.
- **FILEMARK**: The FILEMARK field is set to 0 as it is not applicable to Medium Changer response data.
- **EOM**: The End of Medium (EOM) field is set to 0 as it is not applicable to Medium Changer response data.
- **ILI**: The Incorrect Length Indicator (ILI) field is set to 0 as it is not applicable to Medium Changer response data.
- **SDAT_OVFL**: The Sense Data Overflow (SDAT_OVFL) is set to 0 to indicate that the library does not truncate sense data.
- **Sense Key**: Table 3 on the next page describes the Sense Key values.
- **Information**: This field returns additional information for certain ASC/ASCQs where a specific device must be identified and the sense data is associated with a Unit Attention condition instead of a specific command. These are described as follows:
  - If the ASC/ASCQ is related to Import/Export stations, then byte 6 indicates which I/E station it pertains to (1 to 4).
  - If the ASC/ASCQ is related to a specific Data Transfer Element, then bytes 5 and 6 contain the element address of that element.
  - If the ASC/ASCQ is related to Towers, then byte 6 indicates which Tower it pertains to (1 to n).
- **Additional Sense Length**: This field specifies the number of additional sense bytes that follow this field. If the media changer device control path is hosted by a DA blade controller device, 0Ah (10 bytes) of additional sense data are returned. If the media changer device control path is hosted by a tape drive, 10h (16 bytes) of additional sense data are returned.
- **Command Specific Information**: This field is not supported and returns 0000 0000h.
- **Additional Sense Code (ASC)**: This field denotes a specific error condition. Additional information is provided in the Additional Sense Code Qualifier (ASCQ) field. Table 4 on page 149 lists all the codes.
- **Additional Sense Code Qualifier (ASCQ)**: This field provides additional information for the ASC. Refer to Table 4 on page 149 for more information.
- **Field Replaceable Unit Code**: This field is not used and returns zero.
- **Sense Key Specific Valid (SKSV)**: This field returns a value of 1 if bytes 15-17 contain valid data for a Sense Key of Illegal Request (05h). Otherwise this field returns 0.
- **Command/Data (C/D)**: A value of 1 indicates that the illegal parameter was detected in the CDB. It returns 0 if the illegal parameter was detected in the data parameters. This field only applies if SKSV is 1.
- **Bit Pointer Valid (BPV)**: A value of 0 indicates that the Bit Pointer field is not valid. A value of 1 indicates that the Bit Pointer field is valid. This field only applies if SKSV is 1.
- **Bit Pointer**: This field indicates which bit of the byte designated by the field pointer is in error. For a multi-bit field, it points to the most significant bit of the field. This field only applies if SKSV is 1.
- **Field Pointer**: This field indicates which byte of the CDB or Parameter List (starting with byte zero) was in error. For a multi-byte field, the Field Pointer points to the most significant byte. This field only applies if SKSV is 1.

**Table 3: Sense Key**

| Sense Key | Description |
|-----------|-------------|
| 0h | No Sense. No specific sense key information to report. |
| 2h | Not Ready. The library is not ready to perform motion commands. |
| 4h | Hardware Error. A hardware error was detected and operator intervention may be required. |
| 5h | Illegal Request. The CDB or supplied parameter data contains an unsupported or illegal parameter. |
| 6h | Unit Attention. The library operating status changed. Additional processing may be required. |
| Bh | Aborted Command. The library aborted the command. |


## Additional Sense Codes and Qualifiers

The following table lists the Additional Sense Codes (ASC) and Additional Sense Code Qualifiers
(ASCQ) associated with the reported Sense Keys. A sense key of 00h (no sense) has no ASC/ASCQ
associated with it. A few ASC/ASCQs can be associated with more than one sense key. The sense keys
that can give a particular ASC/ASCQ are indicated with an "x" in the appropriate column.

ASC/ASCQs that can indicate an abnormal element status as part of element descriptor information are
shown in bold.

**Table 4: Additional Sense Codes and Qualifiers**

```
                            Sense Keys
                                                                Description
   ASC         ASCQ
                              2         4           5   6   B

                              x                                 The library is not ready due to an
    04h          00h
                                                                unknown cause

                                                            x   LU Communication - SCSI
    04h          00h
                                                                Command Communication Failure

    04h          01h          x                                 The library is becoming ready

                              x                                 The library is not ready and a
    04h          03h
                                                                manual intervention is required

    04h          12h          x                                 Logical unit not ready, offline

                              x                                 The library is not ready due to aisle
    04h          83h
                                                                power being disabled

                              x                                 The library is not ready because it
    04h          8Dh
                                                                is offline

    08h          00h                                        x   LU Communication Failure


    08h          01h                                        x
                                                                LU Communication – Timeout

    08h          80h                                        x   LU Communication – SCSI Failure

                                                                LU Communication – SCSI
    08h          82h                                        x   Command Execution or Queuing
                                                                Failure

                                                            x   LU Communication – SCSI
    08h          83h
                                                                Command Failed

                                                            x   LU Communication – SCSI Time-
    08h          84h
                                                                Out

                                                            x   LU Communication – SCSI
    08h          85h
                                                                Autosense Failed

                                                            x   LU Communication – SCSI
    08h          86h
                                                                Aborted

                                                            x   LU Communication – SCSI Abort
    08h          87h
                                                                Failed

                                                            x   LU Communication – SCSI Status
    08h          88h
                                                                Failed

                                                            x   LU Communication – FC Data
    08h          B0h
                                                                Underrun

                                                            x   LU Communication – FC DMA
    08h          B1h
                                                                Error

    08h          B2h                                        x   LU Communication – FC Reset

                                                            x   LU Communication – FC Data
    08h          B3h
                                                                Overrun

                                                            x   LU Communication – FC Queue
    08h          B4h
                                                                Full

                                                            x   LU Communication – Port
    08h          B5h
                                                                Unavailable

                                                            x   LU Communication - Port Logged
    08h          B6h
                                                                Out

                                                            x   LU Communication - Port
    08h          B7h
                                                                Configuration Changed

                                        x           x           A mechanical positioning error
    15h          01h
                                                                occurred

   1Ah           00h                                x           Parameter list length error

   1Bh           00h                                        x   Synchronous data transfer error

    20h          00h                                x           Illegal opcode in CDB

    21h          01h                                x           Invalid element address in CDB

    24h          00h                                x           Invalid field in CDB

    25h          00h                                x           Illegal LUN

    26h          00h                                x           Invalid field in Parameter List

                                                    x           Invalid release of persistent
    26h          04h
                                                                reservation

                                                        x       Not Ready to Ready change,
    28h          00h
                                                                element status may have changed

                                                        x       Insert/Eject station opened and
    28h          01h
                                                                closed

    29h          00h                                    x       Power-on or reset occurred

    29h          01h                                    x       Power on occurred

    29h          03h                                    x       Device reset occurred

    29h          04h                                    x       Internal reset occurred

    29h          07h                                        X   Nexxus loss occurred

                                                        x       Mode parameters have been
   2Ah           01h
                                                                changed

   2Ah           03h                                    x       Reservations preempted

   2Ah           04h                                    x       Reservations released

   2Ah           05h                                    x       Registrations preempted

                                                        x       Medium removal prevention
   2Ah           15h
                                                                preempted

   2Ch           00h                                x           Command sequence error

   30h           00h                                x           Incompatible medium installed

    39h          00h                                x           Saving parameters not supported

   3Bh           0Dh                    x           x           The destination element is full

   3Bh           0Eh                    x           x           The source element is empty

   3Bh           11h                                x           Medium magazine not accessible

   3Bh           12h                                x           Media magazine not installed

   3Bh           18h                                x           Element disabled

   3Bh          1Ah                                 x           Data transfer device removed

                                                        x       Element static information
   3Bh           20h
                                                                changed

                                                    x           Media type does not match
   3Bh          A0h
                                                                destination media type

                              x                                 Logical Unit has not self-configured
   3Eh           00h
                                                                yet

   3Fh           01h                                    x       New firmware loaded

   3Fh           03h                                    x       Inquiry data changed

   3Fh           0Fh                    x                       Echo buffer overwritten

   40h           80h                    x                       Component failure

    43h          00h                                        x   Message error

                                        x                   x   Firmware detected an internal logic
    44h          00h
                                                                failure

    45h          00h                                        x   Select or reselect failure

    47h          00h                                        x   SCSI parity error

                                                            x   Initiator detected error message
    48h          00h
                                                                received

    49h          00h                                        x   Invalid message error

   4Ah           00h                                        x   Command phase error

   4Bh           00h                                        x   Data phase error

   4Eh           00h                                        x   Overlapped commands attempted

                                        x                       A drive did not load or unload a
    53h          00h
                                                                tape

   53h           01h                    x           x           A drive did not unload a cartridge

   53h           02h                                x           Medium removal prevented

   53h           07h                                x           Duplicate Volume Identifier

                                                    x           Insert/Eject area element open for
   53h           81h
                                                                operator access

    53h          82h                    x                       Cannot lock the I/E station

    53h          83h                    x                       Cannot unlock the I/E station

   83h           00h                    x                       Label too short or too long

   83h           02h                                x           Barcode label questionable

                                                    x           Cell status and barcode label
   83h           03h
                                                                questionable

   83h           04h                                x           Data transfer element not installed

                                                                Data transfer element offline or
   83h           05h                                x           varied off and not accessible for
                                                                library operations

                                                    x           Element temporarily inaccessible
   83h           06h
                                                                for library operations
```
