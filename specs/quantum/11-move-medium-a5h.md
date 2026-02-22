# Move Medium - A5h

## What the Library Does With This Command

The library will attempt to physically move a cartridge from the requested source element to the
requested destination. The library will make reasonable attempts to retry this operation within the scope
of its capabilities, but if unsuccessful, will try to return the cartridge to its source element. If the source
element was a drive, the library will attempt to leave the cartridge in either a storage or I/E element if the
configuration supports it (not partitioned in the case of the I/E), otherwise it may remain in the picker. If
placement to an alternate slot succeeds, the Move Medium request will report good status, and a Unit
Attention 6/2800 will be queued for all initiators to inform of an element status change so that host
applications can refresh element status and correct tape cartridge inventory status.

If the library includes towers, any required movement of the towers will be provided automatically by the
library.

When the source and destination addresses are the same, the library will still do a full Get and Put, even if
it is a drive (data transfer element).

The library will check that the source element is occupied and that the destination element is empty. It will
also check for media compatibility between the source and destination elements. Failures in either of
these will result in a Check Condition.


## Command Usage

Storage, data transfer, and import/export elements can be used as valid source or destination elements.
The medium transport element (picker) cannot be a destination element. Depending on library model and
configuration, the medium transport element can be a source element to recover a tape cartridge
stranded in the medium transport element.

This is the primary command for the library, and should be used to accomplish any media movement
within the system. If the library indicates a failure due to element status problems (source empty,
destination full, media incompatible, etc.), element status should be re-initialized and re-synchronized.
This would apply to both hardware errors and illegal requests.


## Move Medium CDB Format

The MOVE MEDIUM CDB format is shown in the following table.

**Table 1: MOVE MEDIUM CDB format**

```
              Bit       7           6           5     4          3         2          1          0
  Byte
         0                                           Op Code (A5h)
         1                                                     Reserved
         2
                                              Medium Transport Element Address
         3
         4
                                                     Source Element Address
         5
         6
                                                    Destination Element Address
         7
         8
                                                                Reserved
         9
         10                                            Reserved                                        Invert
                                                                           Control
         11                 FO
```

| Field | Description |
|-------|-------------|
| Medium Transport Element Address | This field contains the address of the Medium Transport element to use for the move. A value of 0001h is the address for all Medium Transport elements, but a value of 0000h is also supported to select the default Medium Transport element. |
| Source Element Address | This field specifies the element address from where the cartridge is retrieved. |
| Destination Element Address | This field specifies the element address for where the cartridge is to be placed. |
| Invert | This field must be set to 0 since the library does not support double-sided media. |
| Failover (FO) | Set to 0 to indicate no failover sequence identified. |
