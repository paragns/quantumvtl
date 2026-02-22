# Test Unit Ready - 00h

## What the Library Does With This Command

The library returns status based on its current mode and state. These are defined in the following table.

**Table 1: Test Unit Ready statuses**

| Mode | State | Status |
|------|-------|--------|
| Online | Ready | Good |
| Online | Not Ready | Check Condition |
| Offline | Ready | Check Condition |
| Offline | Not Ready | Check Condition |

Any time a Not Ready condition or a Unit Attention is pending, a check condition status will be
encountered. The various types of Not Ready and Unit Attention conditions are listed in the Request
Sense command section in Request Sense - 03h on page 145.


## Command Usage

The TEST UNIT READY command allows the initiator to verify that the library is ready to accept
commands or perform motion tasks. It is a suitable command for general polling to monitor the library,
and receive information via Unit Attentions on any changes within the library.


## Test Unit Ready CDB Format

The TEST UNIT READY CDB format is shown in the following table.

**Table 2: TEST UNIT READY CDB format**

```
               Bit           7       6              5    4             3   2       1          0
  Byte
           0                                            Op Code (00h)
           1
           :                                                 Reserved
           4
           5                                                 Control
```
