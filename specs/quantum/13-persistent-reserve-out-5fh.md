# Persistent Reserve Out - 5Fh

## What the Library Does With This Command

The library will perform service actions relative to persistent reservations as requested. This includes
creating and clearing reservations.

An I_T nexus performing Persistent Reserve Out service actions is identified by a reservation key.

This command is only supported if the library control path is provided by a tape drive.

> **Note:** IO blades connected to drives configured with a control path may report certain library ready
> conditions differently than drives without a control path configured.


## Command Usage

This command is used in conjunction with PERSISTENT RESERVE IN to manage persistent
reservations. It can be used to request exclusive access to the device. The PERSISTENT RESERVE IN
and PERSISTENT RESERVE OUT commands should not be used with the RESERVE ELEMENT and
RELEASE ELEMENT commands.


## Persistent Reserve Out CDB Format

The PERSISTENT RESERVE OUT CDB format is shown in the following table.

**Table 1: PERSISTENT RESERVE OUT CDB format**

```
               Bit        7          6              5           4             3         2          1   0
  Byte
         0                                                    Op Code (5Fh)
         1                      Reserved                                          Service Action
         2                               Scope                                              Type
         3
          :
                                                                    Reserved
         6
         7
                                                        Parameter List Length (18h)
         8
         9                                                          Control
```

| Field | Description |
|-------|-------------|
| Service Action | This field specifies what reservation action to take as follows: |
| | - 00h Register -- Register a reservation key without making a reservation. |
| | - 01h Reserve -- Create a persistent reservation of the specified scope and type. |
| | - 02h Release -- Releases the selected reservation for the requesting initiator. |
| | - 03h Clear -- Clears all reservations keys and all persistent reservations. |
| | - 04h Preempt -- Preempt reservations from another initiator. |
| | - 05h Preempt and Abort -- Preempt reservations from another initiator and abort all tasks for all initiators with the specified reservation key. |
| | - 06h Register and Ignore Existing Key -- Register a new reservation key and discard existing reservation key. |
| | - 07h -- Register and Move Registers a reservation key for another I_T nexus and moves the persistent reservation to that I-T nexus. |
| | - 08h -- Replace lost persistent reservation information. |
| Scope | Only logical unit scope is supported, and this field must be a 0h. |
| Type | This field specifies the type of reservation as follows: |
| | - 3h Exclusive Access -- The initiator holding the persistent reservation has exclusive read and write access. Requests from any other initiators to transfer data to or from the logical unit will result in a Reservation Conflict. |
| | - 6h Exclusive Access, Registrants Only -- Any currently registered initiator has exclusive data transfer access. Requests from unregistered initiators to transfer data to or from the logical unit will result in a Reservation Conflict. |
| | - 8h Exclusive Access, All Registrants (only supported if the command is issued to a media changer device hosted by an HP tape drive). |
| Parameter List Length | This field returns 18h (24) to indicate the length of the PERSISTENT RESERVE OUT parameter list, which is shown in the following table. |

**Table 2: PERSISTENT RESERVE OUT Parameter List**

```
             Bit     7         6          5          4          3            2      1       0
  Byte
         0
         :                                               Reservation Key
         7
         8
         :                                     Service Action Reservation Key
       15
       16
         :                                          Scope-Specific Address
       19
                                                            SPEC-1_         ALL_
       20                       Reserved                                           Rsvd   APTPL
                                                              PT           TG-PT
       21                                                   Reserved
       22
         :                                                  Obsolete
       23
       24
         :
                     Additional Parameter Data (see Table 3 on page 97 and Table 4 on page 98)
         n
```

| Field | Description |
|-------|-------------|
| Reservation Key | This is an 8-byte reservation key that identifies the initiator. The value must match the registered reservation key for the I_T nexus except for: |
| | - The Register and Ignore Existing Key service action, where this field is ignored. |
| | - The Register service action for an unregistered I_T nexus, where this field is 0. |
| | If the Reservation Key does not match with the one registered in the device server for the I_T nexus, the device server returns Reservation Conflict. |
| Service Action Reservation Key | This field only applies to the following service actions as follows: |
| | - Register -- This is the new reservation key to register. |
| | - Register and Ignore Existing Key -- This is the new reservation key to register. |
| | - Preempt -- This is the reservation key of the persistent reservation to preempt. |
| | - Preempt and Abort -- This is the reservation key of the persistent reservation to preempt. |
| | For the Register and Register and Ignore Existing Key service actions: |
| | - 0 -- Unregisters the registered reservation key specified in the Reservation Key field. |
| | - n -- The new reservation key to replace the existing one as specified in the Reservation Key field for the I_T nexus. |
| | For the Preempt and Preempt and Abort service actions, this field contains: |
| | - n -- The reservation key of registrations to be removed or, if this field also identifies a persistent reservation holder, the persistent reservation to be pre-empted. |
| | For the Register and Move service action, this field contains: |
| | - n -- The reservation key to be registered on the specified I_T nexus. |
| Scope-Specific Address | Element reservations are not supported and this field must be 0000h. |
| SPEC_I_PT | This bit is valid for the Register and Register and Ignore Existing Key service actions. Set to 0 to ignore the additional parameter data and apply the registration to the I_T nexus that sent the command. Set to 1 to have the additional parameter data include a list of transport IDs and apply the registration to the I_T nexus for every initiator port specified in the transport list |
| ALL_TG_PT | This bit is not supported and is ignored. |
| Activate Persist Through Power Loss (APTPL) | This bit is only valid for the Register, Register and Ignore Existing Key and Register and Move service actions. If set to 1, the logical unit preserves any persistent reservation and all registrations if power is lost and later returned. If the EEPROM (non-volatile memory) is unable to store data anymore, a CHECK CONDITION will be returned to reject the request. If a library media changer device control path is configured via a DA blade, persistent reservations are not supported across power cycles, so this field must be set to 0. |

**Table 3: Additional Parameter Data**

```
               Bit        7          6              5            4          3        2          1           0
  Byte
          24
          :                                Transport Parameter Data length (n - 27)
          27
                 Transport ID List (see Table 5 on page 99 and Table 6 on page 100)
          28
                                                               First Transport ID
          :
                                                           :
          :
                                                           Last Transport ID
          n
```

| Field | Description |
|-------|-------------|
| Transport Parameter Data Length | Specifies the number of bytes of Transport IDs to follow. |
| Transport IDs | See Section Transport IDs below. |

**Table 4: Parameter Data for the Register and Move Service Action**

```
               Bit      7          6          5             4          3         2           1            0
  Byte
          0
          :                                                Reservation Key
          7
          8
          :                                         Service Action Reservation Key
          15
          16                                                    Reserved
          17                                   Reserved                                   Unreg        APTPL
          18
          19                                         Relative Target Port Identifier

          20
          :
                                                  Additional Descriptor Length (18h)
          23
          24
          :
                                                                Transport ID
          n
```

| Field | Description |
|-------|-------------|
| Reservation Key | This is an 8-byte reservation key that identifies the initiator. The value must match the registered reservation key for the I_T nexus except for: |
| | - The Register and Ignore Existing Key service action, where this field is ignored. |
| | - The Register service action for an unregistered I_T nexus, where this field is 0. |
| | If the Reservation Key does not match with the one registered in the device server for the I_T nexus, the device server returns Reservation Conflict. |
| Service Action Registration Key | Specifies the reservation key to be registered on the specified I_T nexus. |
| Unreg | Set to 1 to indicate that the I-T nexus on which the command was received be unregistered. |
| APTPL | Set to 1 to indicate that the logical unit preserve any persistent reservation and all registrations if power is lost and later returned. If the EEPROM (non-volatile memory) is unable to store data anymore, a CHECK CONDITION will be returned to reject the request |
| Relative Target Port Identifier | Relative target port identification descriptor (Table 11 on page 36 |
| Transport ID | See tables below. |

## Transport IDs

**Table 5: Fibre Channel Transport IDs**

```
                Bit        7           6              5              4              3       2            1           0
  Byte
          0            TP_ID Format (00b)                 Reserved                      Protocol Identifier (0h)
          1
          :
                                                                         Reserved
          7
          8
          :
                                                      World Wide Port Name (WWPN)
          15
          16
          :
                                                                         Reserved
          23
```

| Field | Description |
|-------|-------------|
| Transport ID Format (TPID Format) | The TransportID format (TPID FORMAT) field specifies the format of the TransportID. This value is set to 00b. |
| World Wide Port Name (WWPN) | Unique identifier for Fibre Channel port. |

**Table 6: SAS Transport IDs**

```
               Bit        7          6              5            4          3         2           1         0
  Byte
                         TPID Format
          0                                         Reserved                     Protocol Identifier (6h)
                            (00b)
          1
          :
                                                                     Reserved
          3
          4
          :
                                                               SAS Address
          11
          12
          :
                                                                     Reserved
          23
```

| Field | Description |
|-------|-------------|
| Transport ID Format (TPID Format) | The TransportID format (TPID FORMAT) field specifies the format of the TransportID. This value is set to 00b. |
| SAS Address | Port address on SAS device. |
