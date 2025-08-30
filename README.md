# Scribble - Advanced Rust Drawing App

A sophisticated drawing and text annotation application built with Rust and egui, featuring intelligent search capabilities, text selection, and smart visual feedback systems.

## Features

### **Drawing Tools**
- **Interactive Drawing**: Click and drag to draw freehand strokes
- **Customizable Strokes**: Adjust stroke width (1-10 pixels) and colors
- **Real-time Preview**: See your current stroke while drawing

### **Text Annotation**
- **Text Placement**: Click anywhere to place text elements
- **Multiline Support**: Create text blocks with multiple lines
- **Font Size Control**: Adjust text size (10-50 pixels)
- **Black Text Only**: Consistent text appearance (colors reserved for drawings)

### 🔍 **Advanced Search System**
- **Smart Text Search**: Find text elements with real-time highlighting
- **Regex Support**: Advanced pattern matching capabilities
- **Visual Arrows**: Dark orange arrows point to search matches
- **Smart Arrow Positioning**: Collision detection prevents arrows from overlapping text
- **Match Counter**: Shows total number of individual matches found
- **Intelligent Positioning**: Arrows adapt placement (bottom → top → left → right)

### 🎯 **Text Selection & Manipulation**
- **Selection Tool**: Drag to select multiple text elements
- **Visual Feedback**: Blue highlighting shows selected text
- **Drag & Drop**: Move selected text elements around the canvas
- **Copy to Clipboard**: Copy selected text using the copy button
- **Smart Selection Logic**: Click on selected text to drag, click elsewhere to select

### **Visual Enhancements**
- **Faded Grey Canvas**: Easy-on-the-eyes background
- **Collision Detection**: Text becomes semi-transparent when arrows would overlap
- **Cross-platform UI**: Consistent experience across all platforms

## Getting Started

### Prerequisites

- **Rust** (latest stable version)
- **Cargo** (comes with Rust)

### Building and Running

```bash
# Clone the repository
git clone https://github.com/Nikhil-K-Singh/scribble.git
cd scribble

# Build and run
cargo run
```

## 🎮 Controls & Usage

### Drawing Mode (✏️ Draw)
- **Mouse**: Click and drag to draw freehand strokes
- **Stroke Width**: Use slider to adjust thickness (1-10 pixels)
- **Color Picker**: Choose drawing colors

### Text Mode (📝 Text)  
- **Click**: Place text at cursor position
- **Type**: Enter text (multiline supported)
- **Ctrl+Enter**: Confirm and place text
- **Escape**: Cancel text input
- **Font Size**: Adjust with slider (10-50 pixels)

### Selection Mode (🔍 Select)
- **Drag in Empty Space**: Create selection rectangle
- **Drag on Selected Text**: Move selected text elements
- **Click Empty Space**: Clear selection
- **Copy Button**: Copy selected text to clipboard

### Search Features
- **🔍 Search Button**: Toggle search mode
- **Search Box**: Type to find text (case-insensitive)
- **Regex Checkbox**: Enable regular expression patterns
- **Clear Search**: Remove search highlighting

### General Controls
- **Clear Button**: Reset entire canvas (drawings and text)
- **Tool Selection**: Switch between Draw, Text, and Select modes

## 🛠️ Technology Stack

- **Rust**: Systems programming language for performance and safety
- **egui**: Immediate mode GUI library for responsive interfaces
- **eframe**: Native application framework
- **regex**: Advanced pattern matching for search functionality
- **arboard**: Cross-platform clipboard support

## 📁 Project Structure

```
scribble/
├── src/
│   └── main.rs          # Complete application code (~1000+ lines)
├── Cargo.toml           # Project dependencies
├── README.md            # This documentation
└── LICENSE             # MIT License
```

## 📦 Dependencies

```toml
[dependencies]
eframe = "0.28"          # Native egui framework
egui = "0.28"            # Immediate mode GUI
regex = "1.0"            # Regular expression support
arboard = "3.4"          # Clipboard functionality
```

## 🎯 Key Technical Features

### Smart Arrow System
- **Collision Detection**: Prevents arrows from overlapping existing text
- **Multi-directional**: Tries bottom → top → left → right positioning
- **Visual Consistency**: Dark orange arrows for search results
- **Line-aware**: Handles multiline text with precise positioning

### Intelligent Text Selection
- **State Management**: Distinguishes between selecting and dragging
- **Visual Feedback**: Real-time selection rectangle and highlighting
- **Multi-element**: Select and manipulate multiple text elements simultaneously

### Advanced Search Engine
- **Pattern Matching**: Both literal and regex search modes
- **Position Tracking**: Tracks exact character positions for precise arrow placement
- **Live Updates**: Real-time search as you type
- **Error Handling**: Graceful regex error reporting

## Architecture Highlights

- **Immediate Mode GUI**: Responsive, state-driven interface
- **Cross-platform**: Runs natively on Windows, macOS, and Linux
- **Memory Efficient**: Rust's ownership system ensures optimal performance
- **Type Safety**: Compile-time guarantees prevent runtime errors

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

*Built with ❤️ using Rust and egui*
