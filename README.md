
# 🚗 TwinCAN

> **A powerful DBC file comparison tool with a beautiful GUI**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Slint](https://img.shields.io/badge/Slint-GUI-blue?style=for-the-badge)](https://slint.dev/)
[![CAN](https://img.shields.io/badge/CAN-Bus-green?style=for-the-badge)](https://en.wikipedia.org/wiki/CAN_bus)

TwinCAN is a modern, user-friendly application designed to compare DBC (Database CAN) files side-by-side. Perfect for automotive engineers, embedded systems developers, and anyone working with CAN bus networks.

## ✨ Features

### 🔍 **Comprehensive Comparison**
- **Message-level comparison**: ID, ID-Format, DLC, Cycle Time, Transmitter
- **Signal-level comparison**: Length, Byte Order, Value Type, Initial Value, Start bits, Multiplexer, factor, offset, min/max values, unit, Signal Descriptions

### 📊 **Export Capabilities**
- **CSV Export**: Save comparison results for further analysis
- **Structured Output**: Organized by type, message, signal, and field differences
- **Ready for Excel**: Import directly into spreadsheet applications

### 🚀 **Performance**
- **Fast Parsing**: Efficient DBC file processing with regex-based parsing
- **Memory Efficient**: Optimized for large DBC files
- **Cross-platform**: Runs on Windows, macOS, and Linux

## 🛠️ Installation

### Quick Start
1. **Clone** this repository
2. **Run the application**:
   ```bash
   cargo run
   ```
3. **Start comparing** your DBC files!

### Local Installation
```bash
# Clone the repository
git clone https://gitlab.com/JP-Makers/twincan.git
cd TwinCAN

# Build and run
cargo build --release
cargo run --release
```

## 🚀 Usage

### Step 1: Select DBC Files
1. Click **"📂 Browse..."** for DBC File 1
2. Select your first DBC file
3. Click **"📂 Browse..."** for DBC File 2
4. Select your second DBC file

### Step 2: Compare
1. Click **"⚡ Compare Files"** button
2. View results in the comparison table below
3. Results are organized by:
   - **Type**: Message or Signal
   - **Message**: CAN message name
   - **Signal**: Signal name (if applicable)
   - **Field**: What property differs
   - **DBC1/DBC2**: Values from each file

### Step 3: Export (Optional)
1. Click **"📊 Export to CSV"** to save results
2. Choose location for your CSV file
3. Open in Excel or any spreadsheet application

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **🐛 Report Bugs**: Open an issue with detailed information
2. **💡 Suggest Features**: Share your ideas for improvements
3. **🔧 Submit PRs**: Fork, develop, and submit pull requests
4. **📖 Improve Docs**: Help make our documentation better

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

- 🐛 **Issues**: [Gitlab Issues](../../issues)
- 📧 **Contact**: Open an issue for questions

---

<div align="center">

**Made with ❤️ and Rust**

[⭐ Star this repo](../../stargazers) • [🍴 Fork it](../../fork) • [📝 Report bug](../../issues)

</div>
