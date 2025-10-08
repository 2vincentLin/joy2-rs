# joy2-rs

A Rust application to connect Nintendo Joy-Con 2 controllers to Windows PC and map inputs to keyboard and mouse actions.

## ⚠️ Disclaimer

**This project is NOT affiliated with, endorsed by, or supported by Nintendo Co., Ltd.**

This software is developed for personal use and educational purposes. It does NOT:
- Break any DRM (Digital Rights Management)
- Modify or interfere with Nintendo hardware or software
- Violate any intellectual property rights

Use this software at your own risk. The developers are not responsible for any damage to your hardware or any consequences of using this software.

## What Does This Do?

`joy2-rs` allows you to:
- Connect Nintendo Joy-Con 2 controllers to your Windows PC via Bluetooth
- Map Joy-Con buttons, analog sticks, and gyroscope to keyboard and mouse inputs
- Customize button mappings through configuration files
- Use multiple profiles for different applications
- Cache controller MAC addresses for faster reconnection

**Platform Support:** Windows only (uses Windows SendInput API for keyboard/mouse simulation)

## Why This Project Exists

I needed a controller while pedaling on my smart trainer for exercise. Joy-Con 2 seemed perfect:
- Has wrist straps for secure grip
- Small and ergonomic - can hold one in each hand
- Wireless and portable

I initially believed (from online AI summaries) that Joy-Con 2 could easily connect to Windows PC. Turns out, that was information about Joy-Con 1. The Joy-Con 2 uses a completely different Bluetooth protocol that Windows doesn't natively support.

So I decided to make my own connection tool.

## How I Made This

**Full disclosure:** I am NOT a reverse engineer, nor a Bluetooth Low Energy (BLE) expert. 

This project was built by:
1. Reading open-source repositories and documentation:
   - [Joy2Win](https://github.com/Logan-Gaillard/Joy2Win) - Joy-Con 2 protocol research
   - [switch2_controller_research](https://github.com/ndeadly/switch2_controller_research/blob/master/bluetooth_interface.md) - Bluetooth interface documentation
   - [Joycon2test](https://github.com/yujimny/Joycon2test) - Testing and validation

2. Porting the logic to Rust with the help of AI coding assistants

3. Standing on the shoulders of the community's collective knowledge

All I really did was read, understand, and implement. This is a testament to community knowledge-sharing and the advancement of AI-assisted development.

## Features

- ✅ **Full Joy-Con 2 Support**: Connect both Left and Right Joy-Con 2 controllers
- ✅ **Button Mapping**: Map any button to keyboard keys or mouse clicks
- ✅ **Analog Stick Support**: 
  - Map to directional keys (WASD)
  - Map to mouse movement with adjustable sensitivity
  - Continuous movement when stick is held (not just on position change)
- ✅ **Gyroscope**: Use gyro for mouse control (toggle on/off per controller), the orientation is button facing up.
- ✅ **Multi-Profile Support**: Switch between different button layouts on-the-fly
- ✅ **Sensitivity Cycling**: Adjust mouse/gyro sensitivity during gameplay
- ✅ **Profile Overrides**: Different button mappings when gyro mouse is active
- ✅ **Multi-Key Combos**: Support for combinations like `shift+w`
- ✅ **MAC Address Caching**: Faster reconnection to previously paired controllers
- ✅ **Battery Level Monitoring**: See battery status on connection

## Requirements

- **Windows 10/11** (64-bit)
- **Bluetooth 4.0+** adapter
- **Nintendo Joy-Con 2** controllers (the ones with the Nintendo Switch 2)
- **Rust toolchain** (for building from source)

## Installation

### Option 1: Download Pre-built Binary (Coming Soon)

Pre-built binaries will be available in the [Releases](../../releases) page.

### Option 2: Build from Source

1. **Install Rust**: https://rustup.rs/

2. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/joy2-rs.git
   cd joy2-rs
   ```

3. **Build the project**:
   ```bash
   cargo build --release
   ```

4. **The executable will be at**: `target/release/joy2-rs.exe`

## Usage

### Quick Start

1. **Put your Joy-Con 2 in pairing mode**:
   - Press and hold the small sync button on the side rail until the LEDs start flashing

2. **Run the application**:
   - Double-click `joy2-rs.exe`, or
   - Run from command line: `cargo run --release`
   - you should see a console pop up and display some essential info

3. **The application will**:
   - Scan for Joy-Con 2 controllers
   - Connect automatically when found
   - Start sending keyboard/mouse input based on your configuration

1. **To stop**: Press `Ctrl+C` in the terminal, or just click close button.

### Configuration

Edit `configs/default.toml` to customize your button mappings.

#### Basic Structure

```toml
[settings]
left_stick_deadzone = 0.15      # Analog stick deadzone (0.0 - 1.0)
right_stick_deadzone = 0.15
vibration_enabled = true
default_profile = "base"         # Starting profile
sensitivity_factor = [1.0, 2.0, 3.0]  # Available sensitivity levels

[[profiles]]
name = "base"
description = "Default profile"

[profiles.buttons]
A = [{ type = "keyhold", key = "space" }]
B = [{ type = "keyhold", key = "b" }]
ZR = [{ type = "keyhold", key = "w" }]
Plus = [{ type = "cyclesensitivity" }]     # Cycle through sensitivity levels
SLR = [{ type = "cycleprofiles" }]         # Switch between profiles
SRR = [{ type = "togglegyromouser" }]      # Toggle gyro mouse (right controller)

[profiles.sticks.left]
mode = "directional"             # Use left stick for WASD
sensitivity = 1.0
[profiles.sticks.left.directions]
up = "w"
down = "s"
left = "a"
right = "d"

[profiles.sticks.right]
mode = "mouse"                   # Use right stick for mouse movement
sensitivity = 1.0

[profiles.gyro.right]
enabled = false                  # Gyro disabled by default (toggle with SRR)
sensitivity = 1.0
invert_x = false
invert_y = false

# Button overrides when gyro mouse is active
[profiles.gyro_mouse_overrides_right]
R = [{ type = "mouseclick", button = "left" }]   # R button = left click in gyro mode
ZR = [{ type = "mouseclick", button = "right" }] # ZR button = right click in gyro mode
```

#### Button Action Types

- `keyhold`: Press and hold a keyboard key
  ```toml
  A = [{ type = "keyhold", key = "space" }]
  ```

- `mouseclick`: Click a mouse button
  ```toml
  R = [{ type = "mouseclick", button = "left" }]  # left, right, or middle
  ```

- `cyclesensitivity`: Cycle through sensitivity levels, this is similar to mouse DPI
  ```toml
  Plus = [{ type = "cyclesensitivity" }]
  ```

- `cycleprofiles`: Switch to the next profile
  ```toml
  SLR = [{ type = "cycleprofiles" }]
  ```

- `togglegyromouser` / `togglegyromousel`: Toggle gyro mouse mode
  ```toml
  SRR = [{ type = "togglegyromouser" }]
  ```

- `none`: Disable a button
  ```toml
  Home = [{ type = "none" }]
  ```

#### Multi-Key Combinations

Use `+` to combine keys (press in order, release in reverse):
```toml
ZL = [{ type = "keyhold", key = "shift+w" }]  # Hold Shift+W
```

#### Available Buttons

**Face Buttons**: `A`, `B`, `X`, `Y`  
**Shoulder Buttons**: `L`, `R`, `ZL`, `ZR`  
**D-Pad**: `DpadUp`, `DpadDown`, `DpadLeft`, `DpadRight`  
**System**: `Plus`, `Minus`, `Home`, `Capture`  
**Stick Clicks**: `LeftStickClick`, `RightStickClick`  
**Side Buttons**: `SLL`, `SRL` (Left controller), `SLR`, `SRR` (Right controller)

## Examples

The `examples/` directory contains several test programs:

- `25_full.rs` - Full manager with ETS2 configuration (for testing)
- `01_scan_joycon2.rs` - Simple scanner to detect Joy-Con 2 controllers
- `11_gyro_mock_mouse.rs` - Test gyroscope input

Run examples with:
```bash
cargo run --example 25_full
```

## Troubleshooting

### Controllers Won't Connect

1. Try to close the app and reopen it, and connect one Joy-Con 2 at a time.
2. Make sure controllers are in pairing mode (sync button held, LEDs flashing)
3. Remove any previous Bluetooth pairings of the Joy-Cons in Windows settings
4. Make sure no other application is using the Joy-Cons
5. Try restarting the Bluetooth service in Windows

### Input Lag or Stuttering

1. Reduce sensitivity in the config file
2. Close other Bluetooth devices that might cause interference
3. Make sure your Bluetooth adapter supports BLE (Bluetooth Low Energy)

### Buttons Not Working

1. Check your `default.toml` configuration
2. Look for warnings about empty keys in the console
3. Make sure you're using valid key names (see Windows Virtual Key Codes)

## Project Structure

```
joy2-rs/
├── src/
│   ├── main.rs              # Main application entry point
│   ├── lib.rs               # Library exports
│   ├── manager.rs           # Controller manager
│   ├── backend/             # Keyboard/mouse input backends
│   ├── joycon2/             # Joy-Con 2 connection & protocol
│   └── mapping/             # Button/stick mapping logic
├── configs/
│   ├── default.toml         # Default configuration
│   └── ETS2.toml           # Example: Euro Truck Simulator 2
└── examples/               # Test programs
```

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

Since this project relies heavily on community research, if you discover new Joy-Con 2 protocol details, please share them!

## Credits

This project wouldn't exist without:

- [Logan-Gaillard/Joy2Win](https://github.com/Logan-Gaillard/Joy2Win) - Joy-Con 2 protocol implementation
- [ndeadly/switch2_controller_research](https://github.com/ndeadly/switch2_controller_research) - Bluetooth interface documentation
- [yujimny/Joycon2test](https://github.com/yujimny/Joycon2test) - Testing and validation
- The Rust community and ecosystem
- AI coding assistants for helping port and debug

## License

MIT License - see [LICENSE](LICENSE) file for details.

## ## 🚧 Future Development & Known Limitations

### Current State

**⚠️ This is an early-stage project!** While functional for personal use, there are several areas that need improvement:

#### Code Quality
- **Not production-ready**: The codebase needs significant refactoring and cleanup
- **Expect bugs**: This is a hobby project, tested primarily on the developer's setup
- **TOML parsing**: Some edge cases in configuration parsing may not be handled correctly
- **Limited error handling**: Some error cases may cause panics or unexpected behavior

#### Platform & Controller Support
- **Windows only**: Currently only supports Windows (SendInput API)
- **Joy-Con 2 only**: Designed specifically for Joy-Con 2 controllers
- **No other controller support yet**: Extending to other controllers (Joy-Con 1, Pro Controller, DualSense, etc.) would require:
  - More abstract protocol layer
  - Controller-specific protocol implementations
  - Better separation of concerns

The **backend trait system** is the most extensible part - you can implement `KeyboardBackend` and `MouseBackend` traits to add support for:
- Linux (uinput, evdev)
- macOS (CGEvent)
- Virtual gamepads (ViGEm)

#### User Interface
- **Console only**: Currently runs in a terminal with log output
- **No GUI planned** (for now): A console showing logs is sufficient for personal use
- **GUI may be added if requested**: If there's community interest, a graphical interface could be developed for:
  - Visual config editor
  - Button mapping visualizer
  - Real-time input monitoring
  - Controller battery status

### Potential Improvements

If this project gets traction, here are areas that could be improved:

1. **Code Architecture**
   - Refactor protocol handling into traits
   - Better separation between transport (BLE) and protocol logic
   - More comprehensive error types
   - Better async/sync boundaries

2. **Configuration**
   - Config validation improvements
   - Better TOML error messages
   - Config hot-reloading
   - Profile switching hotkeys

3. **Features**
   - Calibration data reading (for more accurate stick/gyro)
   - Rumble/vibration support
   - Battery level monitoring
   - Multiple controller pair support
   - Macro recording

4. **Testing**
   - Unit tests for all modules
   - Integration tests
   - Mock controller for testing

5. **Documentation**
   - API documentation
   - Protocol documentation
   - Contributing guide
   - Video tutorials

### Contributing

Given the current state, contributions are welcome but keep in mind:
- The codebase needs significant cleanup
- Breaking changes are likely
- No formal contribution guidelines yet
- Test thoroughly before submitting PRs

If you want to add support for other controllers or platforms, please open an issue first to discuss the approach!

## 📊 Project Status

| Feature | Status | Notes |
|---------|--------|-------|
| Joy-Con 2 Connection | ✅ Working | BLE discovery and pairing |
| Button Mapping | ✅ Working | All buttons supported |
| Stick Mapping | ✅ Working | Directional and mouse modes |
| Gyro Mouse | ✅ Working | Yaw/pitch control |
| Multi-key Combos | ✅ Working | e.g., "shift+w" |
| Profile Switching | ✅ Working | Runtime profile changes |
| MAC Address Cache | ✅ Working | Faster reconnection |
| Windows Support | ✅ Working | SendInput backend |
| Configuration | ✅ Working | TOML-based with validation |
| Linux Support | ❌ Not yet | Would need uinput backend |
| macOS Support | ❌ Not yet | Would need CGEvent backend |
| Other Controllers | ❌ Not yet | Needs abstraction layer |
| GUI | ❌ Not yet | Console only |
| Calibration | ❌ Not yet | Uses default values |
| Rumble | ❌ Not yet | Protocol supports it |

**Legend**: ✅ Implemented | ⚠️ Partial | ❌ Not implemented





---

**Note**: Nintendo, Joy-Con, and Nintendo Switch are trademarks of Nintendo Co., Ltd. This project is not affiliated with Nintendo.
