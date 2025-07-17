
# 🚗 TwinCAN

> **A powerful DBC file comparison tool with a beautiful GUI**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Slint](https://img.shields.io/badge/Slint-GUI-blue?style=for-the-badge)](https://slint.dev/)
[![CAN](https://img.shields.io/badge/CAN-Bus-green?style=for-the-badge)](https://en.wikipedia.org/wiki/CAN_bus)

TwinCAN is a modern, user-friendly application designed to compare DBC (Database CAN) files side-by-side. Perfect for automotive engineers, embedded systems developers, and anyone working with CAN bus networks.

![TwinCAN Interface](https://via.placeholder.com/800x500/667eea/white?text=TwinCAN+Interface)

## ✨ Features

### 🔍 **Comprehensive Comparison**
- **Message-level comparison**: IDs, sizes, cycle times, transmitters
- **Signal-level comparison**: Start bits, signal sizes, factors, offsets, min/max values, units
- **Difference highlighting**: Clear visual distinction between Message and Signal differences

### 🎨 **Beautiful Interface**
- **Modern GUI**: Built with Slint for a responsive, native experience
- **Intuitive Design**: Easy file selection with drag-and-drop feel
- **Color-coded Results**: Messages and signals are visually differentiated
- **Professional Styling**: Gradient buttons and smooth animations

### 📊 **Export Capabilities**
- **CSV Export**: Save comparison results for further analysis
- **Structured Output**: Organized by type, message, signal, and field differences
- **Ready for Excel**: Import directly into spreadsheet applications

### 🚀 **Performance**
- **Fast Parsing**: Efficient DBC file processing with regex-based parsing
- **Memory Efficient**: Optimized for large DBC files
- **Cross-platform**: Runs on Windows, macOS, and Linux

## 🛠️ Installation

### Prerequisites
- Rust 1.70+ (automatically handled in Replit)
- Modern operating system (Windows 10+, macOS 10.14+, Linux with GUI support)

### Quick Start on Replit
1. **Clone or Fork** this repository
2. **Run the application**:
   ```bash
   cargo run
   ```
3. **Start comparing** your DBC files!

### Local Installation
```bash
# Clone the repository
git clone <your-repo-url>
cd TwinCAN

# Build and run
cargo build --release
cargo run
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
1. Click **"📊 Export to Excel"** to save results
2. Choose location for your CSV file
3. Open in Excel or any spreadsheet application

## 📋 Comparison Details

### Message Properties
- ✅ Message ID and type (Standard/Extended CAN)
- ✅ Message size (bytes)
- ✅ Cycle time (ms)
- ✅ Transmitter node
- ✅ Signal existence

### Signal Properties
- ✅ Start bit position
- ✅ Signal size (bits)
- ✅ Scale factor
- ✅ Offset value
- ✅ Min/Max ranges
- ✅ Engineering units

## 🏗️ Architecture

```
TwinCAN/
├── src/
│   ├── main.rs          # Main application logic & GUI callbacks
│   └── rs_dbc.rs        # DBC parsing engine
├── ui/
│   └── main.slint       # User interface definition
├── assets/
│   └── logo.png         # Application icon
└── Cargo.toml           # Dependencies & metadata
```

### Key Components
- **🎨 Slint GUI**: Modern cross-platform UI framework
- **🔧 DBC Parser**: Custom regex-based parser for DBC files
- **📊 Comparison Engine**: Efficient diff algorithm
- **💾 CSV Exporter**: Structured data export functionality

## 🧩 Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `slint` | 1.12.1 | Cross-platform GUI framework |
| `regex` | 1.11.1 | DBC file parsing |
| `rfd` | 0.15.3 | Native file dialogs |

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **🐛 Report Bugs**: Open an issue with detailed information
2. **💡 Suggest Features**: Share your ideas for improvements
3. **🔧 Submit PRs**: Fork, develop, and submit pull requests
4. **📖 Improve Docs**: Help make our documentation better

### Development Setup
```bash
# Fork and clone the repository
git clone <your-fork-url>
cd TwinCAN

# Create a feature branch
git checkout -b feature/amazing-feature

# Make your changes and test
cargo test
cargo run

# Commit and push
git commit -m "Add amazing feature"
git push origin feature/amazing-feature
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Slint Team** for the amazing GUI framework
- **Rust Community** for the powerful ecosystem
- **CAN Bus Community** for standardizing DBC format
- **Open Source Contributors** who make projects like this possible

## 📞 Support

- 🐛 **Issues**: [GitHub Issues](../../issues)
- 💬 **Discussions**: [GitHub Discussions](../../discussions)
- 📧 **Contact**: Open an issue for questions

---

<div align="center">

**Made with ❤️ and Rust**

[⭐ Star this repo](../../stargazers) • [🍴 Fork it](../../fork) • [📝 Report bug](../../issues)

</div>
