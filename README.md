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
- Pressure sensitivity support (normalized & clamped)
- Stylus button support
- Tablet shortcut buttons mapped to keyboard keys
- Virtual keyboard + virtual pen devices
- Basic smoothing to reduce jitter
- Multi-monitor mapping via `xinput`
- **Background daemon with automatic hotplug & reconnection** (seamless plug & play)
- **Zero idle CPU overhead** while device is disconnected
- **systemd and udev integration** (runs automatically in background without manual `sudo`)

---

# Pressure handling

Pressure normalization is implemented with thresholding and range clamping:

```rust
fn normalize_pressure(raw_pressure: i32) -> i32 {
    let proximity_threshold = 300;
    let strength_scaling = 3;

    let diff = 1740 - raw_pressure;
    if diff <= proximity_threshold {
        0
    } else {
        (diff * strength_scaling).clamp(0, 2048)
    }
}
```

 Notes:

* Threshold too low may cause false touches
* Threshold too high may disable touch detection
* Values are clamped to the 0..=2048 pressure range of the virtual stylus

---

# Supported Devices

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

### Option 1: Run as a background systemd service (Recommended - Automatic)

Install the binary, udev rule, and systemd service:

```bash
# 1. Build release binary
cargo build --release

# 2. Install binary
sudo install -m 755 target/release/moons-ink /usr/local/bin/moons-ink

# 3. Install udev rule and reload
sudo install -m 644 system/99-moons-ink.rules /etc/udev/rules.d/99-moons-ink.rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# 4. Enable and start systemd service
sudo install -m 644 system/moons-ink.service /etc/systemd/system/moons-ink.service
sudo systemctl daemon-reload
sudo systemctl enable --now moons-ink.service
```

Check service status or logs:

```bash
systemctl status moons-ink
journalctl -u moons-ink -f
```

The driver will automatically detect when the tablet is connected, initialize it, and seamlessly reconnect whenever it is unplugged and plugged back in.

#### Manage the service:

```bash
# Stop the service
sudo systemctl stop moons-ink.service

# Restart the service
sudo systemctl restart moons-ink.service

# Disable from starting automatically on boot
sudo systemctl disable moons-ink.service

# Uninstall service and files
sudo systemctl disable --now moons-ink.service
sudo rm -f /etc/systemd/system/moons-ink.service
sudo rm -f /etc/udev/rules.d/99-moons-ink.rules
sudo rm -f /usr/local/bin/moons-ink
sudo systemctl daemon-reload
sudo udevadm control --reload-rules
```

---

### Option 2: Run directly in terminal

With udev rules installed (allows non-root execution):

```bash
./target/release/moons-ink
```

Or run with sudo:

```bash
sudo ./target/release/moons-ink
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

