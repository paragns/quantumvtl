# Prevent Allow Medium Removal - 1Eh

## What the Library Does With This Command

The library will prohibit movement of media to an Import/Export element when media removal has been
prevented. MOVE MEDIUM commands requesting such a move will be rejected with a Check Condition
indicating Medium Removal Prevented. This command does not control locking or unlocking of a
mailbox. The library automatically locks mailboxes during robotic access, and unlocks them afterwards.

While media removal is prevented, importing of media can still occur. The medium removal setting does
not persist across power cycles of the library.

Any initiator issuing this command to allow medium removal (Prevent set to 00b) will allow medium
removal for all initiators. This is done to maintain compatibility with certain bridged environment behavior
(e.g., Fibre Channel to Parallel SCSI).

The prevention of medium removal condition terminates after:
- one of the following occurs for each I_T nexus that had previously prevented medium removal:
    - successful completion of a PREVENT ALLOW MEDIUM REMOVAL command with the PREVENT field set to 00b; or
    - an I_T nexus loss.
- a power on
- a hard reset
- a logical unit reset


## Command Usage

In conjunction with keyed access to the physical library doors, this command can be used to secure the
library against unauthorized removal of media.


## Prevent Allow Medium Removal CDB Format

The PREVENT ALLOW MEDIUM REMOVAL CDB format is shown in the following table.

**Table 1: PREVENT ALLOW MEDIUM REMOVAL CDB format**

```
               Bit         7             6          5      4         3     2         1         0
    Byte
           0                                            Op Code (1Eh)
           1                                              Reserved
           2                                               Reserved
           3                                               Reserved
           4             Preempt                         Reserved                                Prevent
           5                                                Control
```

| Field | Description |
|-------|-------------|
| Preempt | A Preempt bit set to 1 requests that the device terminate the prevention of medium removal condition if a prevention of medium removal condition has been established regardless of the I_T nexuses that requested prevention of medium removal. It is not an error if the Preempt bit is set to 1 and no prevention of medium removal condition has been established. A Preempt bit set to 0 does not request that the device terminate the prevention of medium removal condition. If the PREEMPT bit is set to one and the PREVENT field is not set to 00b, then the command is terminated with CHECK CONDITION status, with the sense key set to ILLEGAL REQUEST and the additional sense code set to INVALID FIELD IN CDB. If the prevention of medium removal condition is preempted, then a unit attention condition will be queued with the additional sense code set to MEDIUM REMOVAL PREVENTION PREEMPTED for the initiator port associated with each I_T nexus that is affected by the preemption of the prevention of medium removal condition except for the I_T nexus over which this command was received. |
| Prevent | This field controls medium removal as follows: |
| | - 00b -- Allow medium removal |
| | - 01b -- Prohibit medium removal |
| | - 10b -- Not supported |
| | - 11b -- Not supported |
