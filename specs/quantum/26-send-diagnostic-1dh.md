# Send Diagnostic - 1Dh

## What the Library Does With This Command

This command requests a tape library diagnostic. The library only supports a self-test option, which
executes a pre-defined diagnostic.


## Command Usage

This command can be used to verify the operational status of the library and its components.


## Send Diagnostic CDB Format

The SEND DIAGNOSTIC CDB format is shown in the following table.

**Table 1: SEND DIAGNOSTIC CDB format**

```
          Bit     7          6         5            4          3            2              1              0
  Byte
      0                                                    Op Code (1Dh)
      1               Self-Test Code            PF           Rsvd        SelfTest      DevOfl          UnitOfl
      2                                                       Reserved
      3
                                                        Parameter List Length
      4
      5                                                        Control
```

- **Self-Test Code**: The Self-Test Code is not supported and must be set to 0.
- **Page Format (PF)**: Diagnostic pages are not supported and this field should be set to 0.
- **SelfTest**: When set to 1 the library will perform a predefined self-test. The SEND DIAGNOSTIC command will not return until this completes, and command completion status will indicate the results of this test. When set to 0, the self-test is not performed.
- **Device Offline (DevOfl)**: This field is not supported and should be set to 0.
- **Unit Offline (UnitOfl)**: This field is not supported and should be set to 0.
- **Parameter List Length**: This field is not supported and should be set to 0.
