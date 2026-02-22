# Position to Element - 2Bh

## What the Library Does With This Command

The library will move the picker in front of the specified element at the current media Get position.


## Command Usage

This command can be used to pre-position the robotics to an element to enhance performance, or it can
be used as a general-purpose way to relocate the robotics without involving media movement. This might
be useful for diagnostic or demonstration purposes.


## Position to Element CDB Format

The POSITION TO ELEMENT CDB format is shown in the following table.

**Table 1: POSITION TO ELEMENT CDB format**

```
              Bit           7       6           5           4             3       2          1           0
  Byte
          0                                                Op Code (2Bh)
          1                                                     Reserved
          2                                   Medium Transport Element Address
          3
          4                                         Destination Element Address
          5
          6
                                                                Reserved
          7
          8                                             Reserved                                       Invert
          9                                                     Control
```

| Field | Description |
|-------|-------------|
| Medium Transport Element Address | This field contains the address of the Medium Transport element to position. A value of 0001h is the address of the Medium Transport element, but a value of 0000h is also supported (which selects the default Medium Transport element). |
| Destination Element Address | This field contains the element address of the target to position to. It can be a storage, data transfer, or import/export element. |
| Invert | This invert request is not supported by the robot. This field must be set to 0. |
