# moons-ink (10moons Linux User-Space Driver)

A modern user-space Linux driver for 10moons graphics tablets, built in Rust using evdev and uinput.

This project is a **heavily modified continuation** of an older MX002 driver (circa 2021), but has since evolved into a **device-specific implementation for 10moons tablets**, with custom pressure handling, smoothing, and input normalization.

It is no longer a generic MX002 driver.

---

#  Status

**Beta / usable**

The driver is functional and usable for drawing, but still under active calibration.

Known quirks:
- Pressure curve is heuristic-based
- Minor smoothing artifacts on fast diagonal strokes
- HID decoding is partially reverse-engineered
- Some hardware assumptions may not generalize across all rebrands

---

#  Features

- Full stylus movement via uinput
- Pressure sensitivity support (normalized)
- Stylus button support
- Tablet shortcut buttons mapped to keyboard keys
- Virtual keyboard + virtual pen devices
- Basic smoothing to reduce jitter
- Multi-monitor mapping via `xinput`

---

# Pressure handling

Pressure normalization is currently implemented heuristically:

```rust
fn normalize_pressure(raw_pressure: i32) -> i32 {
    let proximity_threshold = 300;
    let strength_scaling = 3;

    match 1740 - raw_pressure {
        x if x <= proximity_threshold => 0,
        x => x * strength_scaling,
    }
}
````

 Notes:

* Threshold too low may cause false touches
* Threshold too high may disable touch detection
* Scaling is experimental and may exceed device range

---

#Supported Devices

Originally derived from MX002-class tablets, but now primarily tested on:

* 10moons G10 / 1060Plus series

Other rebrands MAY work but are not guaranteed.

To check USB compatibility:

```bash
lsusb | grep 08f2:6811
```

Expected output:

```
Gotop Information Inc. [T501] Driver Inside Tablet
```

---

#  Build

Requires Rust:

```bash
git clone git@github.com:asdaasp-hearn-mat/moons-ink-driver.git
cd moons-ink-driver
cargo build --release
```

Binary output:

```
target/release/moons-ink
```

---

#  Usage

Run as root:

```bash
sudo ./moons-ink
```

---

## Optional: udev rule

```bash
echo 'SUBSYSTEM=="usb", ATTRS{idVendor}=="08f2", ATTRS{idProduct}=="6811", TAG+="uaccess"' \
| sudo tee /etc/udev/rules.d/99-moons-ink.rules

sudo udevadm control --reload-rules
sudo udevadm trigger
```

---
#  Architecture

* USB HID input via `rusb`
* Event parsing (custom reverse engineering)
* evdev virtual devices
* uinput emission layer
* smoothing + normalization layer

---

#  TODO

* Better HID report decoding (reduce assumptions)
* Pressure curve calibration profiles
* Proper hover/proximity detection refinement
* Multi-device support (other 10moons variants)
* Config file instead of hardcoded mappings
* Improve diagonal smoothing algorithm
* Remove remaining legacy MX002 assumptions

---

# Origins

This project is a **heavily modified evolution** of earlier work inspired by:
* [https://github.com/marvinbelfort/mx002_linux_driver](https://github.com/marvinbelfort/mx002_linux_driver)
* [https://github.com/DIGImend/10moons-tools](https://github.com/DIGImend/10moons-tools)
* [https://github.com/alex-s-v/10moons-driver](https://github.com/alex-s-v/10moons-driver)


---

#  Disclaimer

This is a reverse-engineered user-space driver.

It is not officially supported by any hardware vendor and may require tuning depending on tablet revision.

```

---

