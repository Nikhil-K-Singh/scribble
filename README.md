# Scribble - Multi-Page Drawing and Notebook App

A sophisticated drawing and text annotation application built with Rust and egui, featuring multi-page notebooks, intelligent search capabilities, text selection, and comprehensive export functionality.

## Features

### **Multi-Page Notebook System**
- **Notebook Creation**: Create notebooks with multiple pages (1-100 pages)
- **Page Navigation**: Previous/Next buttons for easy page switching
- **Dynamic Pages**: Add new pages to existing notebooks
- **Page Status**: Visual indicator showing current page (e.g., "Page 2 of 5")
- **Dual Mode Support**: Single page mode or multi-page notebook mode

### **Drawing Tools**
- **Interactive Drawing**: Click and drag to draw freehand strokes
- **Customizable Strokes**: Adjust stroke width (1-10 pixels) and colors
- **Real-time Preview**: See your current stroke while drawing
- **Per-Page Content**: Each page maintains its own drawings independently

### **Text Annotation**
- **Text Placement**: Click anywhere to place text elements
- **Multiline Support**: Create text blocks with multiple lines
- **Font Size Control**: Adjust text size (10-50 pixels)
- **Black Text Only**: Consistent text appearance (colors reserved for drawings)
- **Page-Specific Text**: Text elements are unique to each page

### **Advanced Search System**
- **Smart Text Search**: Find text elements with real-time highlighting
- **Regex Support**: Advanced pattern matching capabilities
- **Visual Arrows**: Dark orange arrows point to search matches
- **Smart Arrow Positioning**: Collision detection prevents arrows from overlapping text
- **Match Counter**: Shows total number of individual matches found
- **Intelligent Positioning**: Arrows adapt placement (bottom → top → left → right)
- **Page-Aware Search**: Search operates on current page content

### **Text Selection & Manipulation**
- **Selection Tool**: Drag to select multiple text elements
- **Visual Feedback**: Blue highlighting shows selected text
- **Drag & Drop**: Move selected text elements around the canvas
- **Copy to Clipboard**: Copy selected text using the copy button
- **Smart Selection Logic**: Click on selected text to drag, click elsewhere to select

### **File Management & Export**
- **Save/Load Projects**: Complete .scribble file format support
- **Notebook Persistence**: Save entire notebooks with all pages
- **Backwards Compatibility**: Load old single-page .scribble files
- **Drag & Drop**: Drag .scribble files onto the app to open them
- **Visual Drop Feedback**: Blue overlay and instructions during file drag operations

### **Export Capabilities**
- **Smart PNG Export**: Exports current page as PNG with auto-sizing
- **Smart SVG Export**: Vector format export with proper scaling
- **Content-Aware Bounds**: Exports automatically size to fit all content
- **No Clipping**: Full content export with intelligent padding
- **High Quality**: Professional output suitable for presentations

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

## Controls & Usage

### Multi-Page Operations
- **Create Notebook**: Button to create new multi-page notebook
- **Page Navigation**: Use Previous/Next arrow buttons
- **Add Page**: Plus button to add new pages
- **Page Counter**: Shows current page position

### Drawing Mode (Draw Tool)
- **Mouse**: Click and drag to draw freehand strokes
- **Stroke Width**: Use slider to adjust thickness (1-10 pixels)
- **Color Picker**: Choose drawing colors
- **Per-Page Drawing**: Each page maintains separate drawings

### Text Mode (Text Tool)  
- **Click**: Place text at cursor position
- **Type**: Enter text (multiline supported)
- **Ctrl+Enter**: Confirm and place text
- **Escape**: Cancel text input
- **Font Size**: Adjust with slider (10-50 pixels)

### Selection Mode (Select Tool)
- **Drag in Empty Space**: Create selection rectangle
- **Drag on Selected Text**: Move selected text elements
- **Click Empty Space**: Clear selection
- **Copy Button**: Copy selected text to clipboard

### Search Features
- **Search Button**: Toggle search mode
- **Search Box**: Type to find text (case-insensitive)
- **Regex Checkbox**: Enable regular expression patterns
- **Clear Search**: Remove search highlighting

### File Operations
- **File Menu**: Save Project, Load Project, Export SVG, Export PNG
- **Drag & Drop**: Drag .scribble files onto app window to open
- **Auto-Detection**: Automatically detects single-page vs notebook format

### General Controls
- **Clear Button**: Reset current page (drawings and text)
- **Tool Selection**: Switch between Draw, Text, and Select modes

## Technology Stack

- **Rust**: Systems programming language for performance and safety
- **egui**: Immediate mode GUI library for responsive interfaces
- **eframe**: Native application framework
- **regex**: Advanced pattern matching for search functionality
- **arboard**: Cross-platform clipboard support
- **serde/serde_json**: Serialization for file save/load
- **rfd**: Native file dialogs
- **image**: PNG export functionality

## Project Structure

```
scribble/
├── src/
│   └── main.rs          # Complete application code (~1600+ lines)
├── Cargo.toml           # Project dependencies
├── README.md            # This documentation
└── LICENSE             # MIT License
```

## Dependencies

```toml
[dependencies]
eframe = "0.28"          # Native egui framework
egui = "0.28"            # Immediate mode GUI
regex = "1.0"            # Regular expression support
arboard = "3.4"          # Clipboard functionality
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"       # JSON serialization
rfd = "0.14"             # File dialogs
image = "0.25"           # PNG export
```

## File Format

### .scribble Files
The application uses a JSON-based .scribble format that supports:

**Single Page Format** (backwards compatible):
```json
{
  "strokes": [...],
  "text_elements": [...],
  "canvas_size": [800.0, 600.0]
}
```

**Notebook Format** (multi-page):
```json
{
  "pages": [
    {
      "name": "Page 1",
      "strokes": [...],
      "text_elements": [...]
    }
  ],
  "current_page_index": 0,
  "canvas_size": [800.0, 600.0]
}
```

## Key Technical Features

### Multi-Page Architecture
- **Page-based Data Model**: Separate stroke and text collections per page
- **Efficient Navigation**: Fast page switching with state preservation
- **Scalable Design**: Supports large numbers of pages efficiently

### Smart Export System
- **Content Bounds Calculation**: Analyzes all elements to determine optimal export size
- **Coordinate Translation**: Properly positions content in exported images
- **Format Support**: Both raster (PNG) and vector (SVG) export options

### Intelligent File Handling
- **Format Detection**: Automatically determines file type (single-page vs notebook)
- **Migration Support**: Seamlessly loads old single-page files
- **Drag & Drop Integration**: Native file dropping with visual feedback

### Advanced Search Engine
- **Pattern Matching**: Both literal and regex search modes
- **Position Tracking**: Tracks exact character positions for precise arrow placement
- **Live Updates**: Real-time search as you type
- **Error Handling**: Graceful regex error reporting

### Robust State Management
- **Borrowing Safety**: Rust's ownership system prevents data races
- **Efficient Updates**: Minimal redraws and state changes
- **Memory Management**: Automatic cleanup and resource management

## Architecture Highlights

- **Immediate Mode GUI**: Responsive, state-driven interface
- **Cross-platform**: Runs natively on Windows, macOS, and Linux
- **Memory Efficient**: Rust's ownership system ensures optimal performance
- **Type Safety**: Compile-time guarantees prevent runtime errors
- **Modular Design**: Clean separation of concerns for maintainability

## Use Cases

- **Digital Sketching**: Create multi-page sketchbooks and drawing collections
- **Note Taking**: Combine drawings with text annotations across multiple pages
- **Documentation**: Create illustrated documentation with search capabilities
- **Presentations**: Export pages as high-quality images for presentations
- **Education**: Interactive whiteboards with save/load functionality

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.