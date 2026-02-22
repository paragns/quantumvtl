# Persistent Reserve In - 5Eh

## What the Library Does With This Command

The library returns information about persistent reservation and reservation keys that are currently active.
This command is only supported if the library control path is provided by a tape drive.

> **Note:** IO blades connected to drives configured with a control path may report certain library ready
> conditions differently than drives without a control path configured.


## Command Usage

This command is used in conjunction with PERSISTENT RESERVE OUT to manage persistent
reservations. It can be used to retrieve a list of the current reservations and the registered reservation
keys. The PERSISTENT RESERVE IN and PERSISTENT RESERVE OUT commands should not be
used with the RESERVE ELEMENT and RELEASE ELEMENT commands.


## Persistent Reserve In CDB Format

The PERSISTENT RESERVE IN CDB format is shown in the following table.

**Table 1: PERSISTENT RESERVE IN CDB format**

```
               Bit        7          6              5      4             3         2          1   0
  Byte
          0                                              Op Code (5Eh)
          1                     Reserved                                     Service Action
          2                                                    Reserved
          3                                                    Reserved
          4                                                    Reserved
          5                                                    Reserved
          6                                                    Reserved
          7
                                                        Allocation Length
          8
          9                                                    Control
```

| Field | Description |
|-------|-------------|
| Service Action | This field specifies the type of request being made as follows: |
| | - 0h -- Read all registered reservation keys. |
| | - 1h -- Read all current persistent reservations. |
| | - 2h -- Report capabilities of the supported persistent reservation features (only supported if the command is issued to a media changer device hosted by a tape drive). |
| | - 3h -- Report full status of registration and reservation status for each registered I_T nexus (only supported if the command is issued to a media changer device hosted by a tape drive). |
| Allocation Length | This field specifies the byte length allowed for returning the requested data. The number of bytes returned is the lesser of the available data to return or the allocation length. |


## Persistent Reserve In Response

Response data is returned depending on the requested service action.

### Read Keys Response

The response for a Read Keys service action is shown the following table.

**Table 2: Read Keys Response**

```
               Bit        7           6             5         4         3           2           1          0
  Byte
          0
          :                                                   Generation
          3
          4
          :                                             Additional Length (n-7)
          7
                                             Reservation Key List
          8
          :                                              First Reservation Key
          15
                                                          :
          n-7
           :                                             Last Reservation Key
          n
```

| Field | Description |
|-------|-------------|
| Generation | This field is a 32-bit counter that is incremented every time a PERSISTENT RESERVE OUT command requests a Register, a Register and Ignore Existing Key, a Clear, a Preempt, or a Preempt and Abort service action. This counter is not maintained across power cycles. |
| Additional Length | This field indicates the length in bytes of the Reservation Key List. |
| Reservation Key List | This is a list of all the 8-byte reservation keys that have been registered through the PERSISTENT RESERVE OUT command. |

### Read Reservations Response

The response for a Read Reservations service action is shown in the following table.

**Table 3: Read Reservations Response**

```
                Bit       7          6              5        4          3          2           1           0
  Byte
          0
           :                                                 Generation
          3
          4
           :                                        Additional Length (0 or 0010h)
          7
                                            Reservation Descriptor
          8
           :                                              Reservation Key
          15
          16
          :                                             Scope-Specific Address
          19
          20                                                        Reserved
          21                           Scope                                              Type
          22
                                                                    Obsolete
          23
```

| Field | Description |
|-------|-------------|
| Generation | This field is a 32-bit counter that is incremented every time a PERSISTENT RESERVE OUT command requests a Register, a Register and Ignore Existing Key, a Clear, a Preempt, or a Preempt and Abort service action. This counter is not maintained across power cycles. |
| Additional Length | This field indicates the length in bytes of the Reservation Descriptor, which may return 0 or 0010h (16). Since element reservations are not supported, a single reservation descriptor is returned for logical unit. |
| Reservation Key | This is the 8-byte reservation key that was registered through the PERSISTENT RESERVE OUT command. |
| Scope-Specific Address | Element reservations are not supported and this field returns 0000h. |
| Scope | This field returns a 0h, indicating logical unit scope. Element scope is not supported. |
| Type | This field returns the type of reservation as follows: |
| | - 3h Exclusive Access -- The initiator holding the persistent reservation has exclusive read and write access. Requests from any other initiators to transfer data to or from the logical unit will result in a Reservation Conflict. |
| | - 6h Exclusive Access, Registrants Only -- Any currently registered initiator has exclusive data transfer access. Requests from unregistered initiators to transfer data to or from the logical unit will result in a Reservation Conflict. |
| | - 8h Exclusive Access, All Registrants (only supported if the command is issued to a media changer device hosted by an HP tape drive). |

### Report Capabilities Response

This response contains a bit map that indicates the persistent reservation types that are supported by the
device server.

**Table 4: Report Capabilities Response**

```
          Bit       7            6           5          4            3            2            1             0
  Byte
      0
      1                                                  Length (0008h)
      2          RLR_C               Reserved          CRH        SIP_C        ATP_C         Rsvd        PTPL_C
      3           TMV                                        Reserved                                    PTPL_A
                               EX_         WX_
                 WR_
      4                        AC_         EX_        Rsvd        EX_AC         Rsvd       WR_EX           Rsvd
                EX_AR
                               RO          RO
                                                                                                        EX_AC_
      5                                              Reserved
                                                                                                          AR
      6
      7                                                      Reserved
```

| Field | Description |
|-------|-------------|
| Length | This field reports 8 bytes of data. |
| Replace Lost Reservation Capable (RLR_C) | A replace lost reservation capable (RLR_C) bit set to one indicates that the device supports the REPLACE LOST RESERVATION service action in the PERSISTENT RESERVE OUT command. This bit is set to 0, to indicate that the device does not support the REPLACE LOST RESERVATION service action in the PERSISTENT RESERVE OUT command. If the RLR_C bit is set to zero then the device shall not terminate any commands with CHECK CONDITION status. |
| CRH | A compatible reservation handling (CRH) bit set to one indicates that the device server supports the exceptions to the SPC-defined RESERVE commands and RELEASE commands. A CRH bit set to zero indicates that RESERVE commands and RELEASE commands are processed as defined in the SPC. |
| SIP_C | A specify initiator ports capable (SIP_C) bit set to one indicates that the device server supports the SPEC_I_PT bit in the PERSISTENT RESERVE OUT command parameter data. An SIP_C bit set to zero indicates that the device server does not support the SPEC_I_PT bit in the PERSISTENT RESERVE OUT command parameter data |
| ATP_C | An all target ports capable (ATP_C) bit set to one indicates that the device server supports the ALL_TG_PT bit in the PERSISTENT RESERVE OUT command parameter data. An ATP_C bit set to zero indicates that the device server does not support the ALL_TG_PT bit in the PERSISTENT RESERVE OUT command parameter data. |
| PTPL_C | A persist through power loss capable (PTPL_C) bit set to one indicates that the device server supports the persist through power loss capability for persistent reservations and the APTPL bit in the PERSISTENT RESERVE OUT command parameter data. An PTPL_C bit set to zero indicates that the device server does not support the persist through power loss capability. |
| TMV | A type mask valid (TMV) bit set to one indicates that the PERSISTENT RESERVATION TYPE MASK field contains a bit map indicating which persistent reservation types are supported by the device server. A TMV bit set to zero indicates that the PERSISTENT RESERVATION TYPE MASK field shall be ignored. |
| PTPL_A | A Persist Through Power Loss Activated (PTPL_A) bit set to one indicates that the persist through power loss capability is activated. A PTPL_A bit set to zero indicates that the persist through power loss capability is not activated |
| WR_EX_AR | A Write Exclusive -- All Registrants (WR_EX_AR) bit set to one indicates that the device server supports the Write Exclusive -- All Registrants persistent reservation type. An WR_EX_AR bit set to zero indicates that the device server does not support the Write Exclusive -- All Registrants persistent reservation type. |
| EX_AC_RO | An Exclusive Access -- Registrants Only (EX_AC_RO) bit set to one indicates that the device server supports the Exclusive Access -- Registrants Only persistent reservation type. An EX_AC_RO bit set to zero indicates that the device server does not support the Exclusive Access -- Registrants Only persistent reservation type. |
| WX_EX_RO | A Write Exclusive -- Registrants Only (WR_EX_RO) bit set to one indicates that the device server supports the Write Exclusive -- Registrants Only persistent reservation type. An WR_EX_RO bit set to zero indicates that the device server does not support the Write Exclusive -- Registrants Only persistent reservation type. |
| EX_AC | An Exclusive Access (EX_AC) bit set to one indicates that the device server supports the Exclusive Access persistent reservation type. An EX_AC bit set to zero indicates that the device server does not support the Exclusive Access persistent reservation type. |
| WR_EX | A Write Exclusive (WR_EX) bit set to one indicates that the device server supports the Write Exclusive persistent reservation type. An WR_EX bit set to zero indicates that the device server does not support the Write Exclusive persistent reservation type. |
| EX_AC_AR | An Exclusive Access -- All Registrants (EX_AC_AR) bit set to one indicates that the device server supports the Exclusive Access -- All Registrants persistent reservation type. An EX_AC_AR bit set to zero indicates that the device server does not support the Exclusive Access -- All Registrants persistent reservation type. |

### Report Full Status

The response for a Report Full Status service action is shown in the following table.

**Table 5: Report Full Status**

```
               Bit        7            6            5           4          3           2        1           0
  Byte
          0
          :                                                     Generation
          3
          4
          :                                              Additional Length (n-7)
          7
                              Full Status Descriptors (see Table 6 on the next page)
          8
          :                                             First Full Status Descriptor
          15
                                                            :
         n-7
          :                                              Last Full Status Descriptor
          n
```

| Field | Description |
|-------|-------------|
| Generation | This field is a 32-bit counter that is incremented every time a PERSISTENT RESERVE OUT command requests a Register, a Register and Ignore Existing Key, a Clear, a Preempt, or a Preempt and Abort service action. This counter is not maintained across power cycles. |
| Additional Length (n-7) | This field indicates the length in bytes of the Reservation Descriptor, |
| Full Status Descriptor | The response for a Full Status Descriptor is shown in the table below. |

**Table 6: Full Status Descriptor**

```
               Bit       7         6           5           4          3        2             1           0
  Byte
          0
          :                                               Reservation Key
          7
          8
          :                                                     Reserved
          11
                                                                                          ALL_
                                                                                                      R_
          12                                   Reserved                                  TG_PT
                                                                                                     Holder
                                                                                           (0)
          13                           Scope                                          Type
          14
          :
                                                                Reserved
          17
          18
                                                    Relative Target Port Identifier
          19
          20
          :
                                                    Additional Descriptor Length
          23
          24
          :                                                    Transport ID
          n
```

| Field | Description |
|-------|-------------|
| Reservation Key | This is the 8-byte reservation key that was registered through the PERSISTENT RESERVE OUT command. |
| ALL_TG_PT (0) | This bit is not supported and is ignored. |
| R_Holder | Set to 1 to indicate that the I-T nexus is a Persistent Reservation Holder. |
| Scope | This field returns a 0h, indicating logical unit scope. Element scope is not supported. |
| Type | This field returns the type of reservation as follows: |
| | - 3h Exclusive Access -- The initiator holding the persistent reservation has exclusive read and write access. Requests from any other initiators to transfer data to or from the logical unit will result in a Reservation Conflict. |
| | - 6h Exclusive Access, Registrants Only -- Any currently registered initiator has exclusive data transfer access. Requests from unregistered initiators to transfer data to or from the logical unit will result in a Reservation Conflict. |
| | - 8h Exclusive Access, All Registrants (only supported if the command is issued to a media changer device hosted by an HP tape drive). |
| Relative Target Port Identifier | Relative target port identification descriptor (Table 11 on page 36 |
| Transport _ID | See details in PERSISTENT RESERVE OUT command (Persistent Reserve In - 5Eh on page 84 and Persistent Reserve In - 5Eh on page 84). |
