use eframe::egui;
use regex::Regex;
use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::fs;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Scribble - Drawing App",
        options,
        Box::new(|_cc| Ok(Box::new(ScribbleApp::default()))),
    )
}

#[derive(Clone)]
struct Stroke {
    points: Vec<egui::Pos2>,
    color: egui::Color32,
    width: f32,
}

#[derive(Clone)]
struct TextElement {
    position: egui::Pos2,
    text: String,
    font_size: f32,
}

// Serializable versions for saving/loading
#[derive(Serialize, Deserialize)]
struct SerializableStroke {
    points: Vec<(f32, f32)>,
    color: (u8, u8, u8),
    width: f32,
}

#[derive(Serialize, Deserialize)]
struct SerializableTextElement {
    position: (f32, f32),
    text: String,
    font_size: f32,
}

#[derive(Serialize, Deserialize)]
struct ScribbleProject {
    strokes: Vec<SerializableStroke>,
    text_elements: Vec<SerializableTextElement>,
    canvas_size: (f32, f32),
}

#[derive(PartialEq)]
enum Tool {
    Draw,
    Text,
    Select,
}

struct ScribbleApp {
    strokes: Vec<Stroke>,
    text_elements: Vec<TextElement>,
    current_stroke: Vec<egui::Pos2>,
    is_drawing: bool,
    stroke_color: egui::Color32,
    stroke_width: f32,
    current_tool: Tool,
    text_input: String,
    text_font_size: f32,
    active_text_position: Option<egui::Pos2>,
    text_input_id: egui::Id,
    search_query: String,
    search_results: Vec<usize>,
    show_search: bool,
    regex_mode: bool,
    search_error: Option<String>,
    text_collisions: Vec<usize>, // Track which text elements have arrow collisions
    // Text selection fields
    is_selecting_text: bool,
    selection_start: Option<egui::Pos2>,
    selection_end: Option<egui::Pos2>,
    selected_text_elements: Vec<usize>,
    clipboard: Option<Clipboard>,
}

impl Default for ScribbleApp {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            text_elements: Vec::new(),
            current_stroke: Vec::new(),
            is_drawing: false,
            stroke_color: egui::Color32::BLACK,
            stroke_width: 2.0,
            current_tool: Tool::Draw,
            text_input: String::new(),
            text_font_size: 20.0,
            active_text_position: None,
            text_input_id: egui::Id::new("floating_text_input"),
            search_query: String::new(),
            search_results: Vec::new(),
            show_search: false,
            regex_mode: false,
            search_error: None,
            text_collisions: Vec::new(),
            is_selecting_text: false,
            selection_start: None,
            selection_end: None,
            selected_text_elements: Vec::new(),
            clipboard: Clipboard::new().ok(),
        }
    }
}

impl ScribbleApp {
    fn perform_search(&mut self) {
        self.search_results.clear();
        self.search_error = None;
        
        if self.search_query.is_empty() {
            return;
        }
        
        if self.regex_mode {
            match Regex::new(&self.search_query) {
                Ok(regex) => {
                    for (index, text_element) in self.text_elements.iter().enumerate() {
                        if regex.is_match(&text_element.text) {
                            self.search_results.push(index);
                        }
                    }
                }
                Err(e) => {
                    self.search_error = Some(format!("Regex error: {}", e));
                }
            }
        } else {
            let query_lower = self.search_query.to_lowercase();
            for (index, text_element) in self.text_elements.iter().enumerate() {
                if text_element.text.to_lowercase().contains(&query_lower) {
                    self.search_results.push(index);
                }
            }
        }
    }
    
    fn get_total_match_count(&self) -> usize {
        let mut total_matches = 0;
        
        if self.search_query.is_empty() {
            return 0;
        }
        
        for &index in &self.search_results {
            if let Some(text_element) = self.text_elements.get(index) {
                let matches = self.get_match_positions(&text_element.text);
                total_matches += matches.len();
            }
        }
        
        total_matches
    }
    
    fn get_match_positions(&self, text: &str) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        
        if self.search_query.is_empty() {
            return positions;
        }
        
        if self.regex_mode {
            if let Ok(regex) = Regex::new(&self.search_query) {
                for match_result in regex.find_iter(text) {
                    positions.push((match_result.start(), match_result.end()));
                }
            }
        } else {
            let query_lower = self.search_query.to_lowercase();
            let text_lower = text.to_lowercase();
            let mut start = 0;
            
            while let Some(pos) = text_lower[start..].find(&query_lower) {
                let actual_pos = start + pos;
                positions.push((actual_pos, actual_pos + self.search_query.len()));
                start = actual_pos + 1;
            }
        }
        
        positions
    }
    
    fn draw_arrows_for_matches(&self, painter: &egui::Painter, text_pos: egui::Pos2, text: &str, font_size: f32) {
        let positions = self.get_match_positions(text);
        if positions.is_empty() {
            return;
        }
        
        let font_id = egui::FontId::proportional(font_size);
        
        // Split text into lines to handle multiline positioning
        let lines: Vec<&str> = text.lines().collect();
        let line_height = painter.layout_no_wrap(
            "Ag".to_string(), // Sample text to measure line height
            font_id.clone(),
            egui::Color32::WHITE,
        ).size().y;
        
        for (start_char, end_char) in positions {
            // Find which line the match is on and position within that line
            let mut char_count = 0;
            let mut match_line = 0;
            let mut match_start_in_line = start_char;
            let mut match_end_in_line = end_char;
            
            // Find the line containing the match
            for (line_idx, line) in lines.iter().enumerate() {
                let line_len = line.len() + 1; // +1 for newline character
                if char_count + line_len > start_char {
                    match_line = line_idx;
                    match_start_in_line = start_char - char_count;
                    match_end_in_line = end_char - char_count;
                    break;
                }
                char_count += line_len;
            }
            
            // Ensure we don't go beyond the line boundary
            if match_line < lines.len() {
                let current_line = lines[match_line];
                match_end_in_line = match_end_in_line.min(current_line.len());
                
                // Calculate positions within the specific line
                let line_y = text_pos.y + (match_line as f32 * line_height);
                let before_match = &current_line[..match_start_in_line];
                let match_text = &current_line[match_start_in_line..match_end_in_line];
                
                // Measure text to get horizontal positions
                let before_galley = painter.layout_no_wrap(
                    before_match.to_string(),
                    font_id.clone(),
                    egui::Color32::WHITE,
                );
                let match_galley = painter.layout_no_wrap(
                    match_text.to_string(),
                    font_id.clone(),
                    egui::Color32::WHITE,
                );
                
                let match_start_x = text_pos.x + before_galley.size().x;
                let match_end_x = match_start_x + match_galley.size().x;
                let match_center_x = (match_start_x + match_end_x) / 2.0;
                let text_bottom = line_y + match_galley.size().y;
                
                // Draw arrows pointing to the match on the correct line
                self.draw_pointing_arrows(painter, match_center_x, text_bottom, match_galley.size().x);
            }
        }
    }
    
    fn draw_pointing_arrows(&self, painter: &egui::Painter, center_x: f32, text_bottom: f32, match_width: f32) {
        // Always use dark orange for arrows
        let arrow_color = egui::Color32::from_rgb(200, 80, 0); // Dark orange arrows
        
        let arrow_length = 15.0;
        let arrow_gap = 5.0;
        
        // Try different arrow positions to avoid collisions
        let text_top = text_bottom - 20.0; // Approximate text height
        let text_center_y = text_top + 10.0; // Center of text
        let arrow_positions = [
            ("bottom", center_x, text_bottom + arrow_gap),  // Below (pointing up)
            ("top", center_x, text_top - arrow_gap - arrow_length), // Above (pointing down)
            ("left", center_x - match_width / 2.0 - arrow_gap - arrow_length, text_center_y), // Left (pointing right)
            ("right", center_x + match_width / 2.0 + arrow_gap, text_center_y), // Right (pointing left)
        ];
        
        let mut arrow_drawn = false;
        
        for (arrow_type, arrow_x, arrow_y) in arrow_positions {
            if !self.check_arrow_collision_at_position(arrow_x, arrow_y, arrow_length) {
                match arrow_type {
                    "bottom" => self.draw_bottom_arrow(painter, arrow_x, arrow_y, arrow_length, arrow_color),
                    "top" => self.draw_top_arrow(painter, arrow_x, arrow_y, arrow_length, arrow_color),
                    "left" => self.draw_left_arrow(painter, arrow_x, arrow_y, arrow_length, arrow_color),
                    "right" => self.draw_right_arrow(painter, arrow_x, arrow_y, arrow_length, arrow_color),
                    _ => {}
                }
                arrow_drawn = true;
                break;
            }
        }
        
        // If no position is collision-free, draw at the original bottom position
        if !arrow_drawn {
            self.draw_bottom_arrow(painter, center_x, text_bottom + arrow_gap, arrow_length, arrow_color);
        }
        
        // Draw side arrows if the match is wide enough and we used the bottom position
        if match_width > 30.0 && !arrow_drawn {
            let side_offset = match_width / 3.0;
            
            // Left side arrow
            let left_center = center_x - side_offset;
            self.draw_bottom_arrow(painter, left_center, text_bottom + arrow_gap, arrow_length * 0.7, arrow_color);
            
            // Right side arrow
            let right_center = center_x + side_offset;
            self.draw_bottom_arrow(painter, right_center, text_bottom + arrow_gap, arrow_length * 0.7, arrow_color);
        }
    }
    
    fn check_arrow_collision_at_position(&self, arrow_x: f32, arrow_y: f32, arrow_length: f32) -> bool {
        // Create a slightly larger area around the arrow for collision detection
        let collision_padding = 2.0;
        let arrow_area = egui::Rect::from_center_size(
            egui::Pos2::new(arrow_x, arrow_y),
            egui::Vec2::new(arrow_length + collision_padding * 2.0, arrow_length + collision_padding * 2.0),
        );
        
        // Only check for collisions with other text elements (not the one being searched)
        for (text_idx, text_element) in self.text_elements.iter().enumerate() {
            // Skip text elements that are search results (we want to point to them)
            if self.search_results.contains(&text_idx) {
                continue;
            }
            
            let lines: Vec<&str> = text_element.text.lines().collect();
            let font_size = text_element.font_size;
            let line_height = font_size * 1.2;
            
            for (line_idx, line) in lines.iter().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                
                let line_y = text_element.position.y + (line_idx as f32) * line_height;
                let estimated_text_width = line.len() as f32 * font_size * 0.6; // Rough estimation
                
                let text_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(text_element.position.x, line_y),
                    egui::Vec2::new(estimated_text_width, font_size), // Standard text height
                );
                
                if arrow_area.intersects(text_rect) {
                    return true;
                }
            }
        }
        false
    }
    
    fn draw_bottom_arrow(&self, painter: &egui::Painter, center_x: f32, arrow_y: f32, arrow_length: f32, color: egui::Color32) {
        let arrow_tip = egui::Pos2::new(center_x, arrow_y);
        let arrow_base = egui::Pos2::new(center_x, arrow_y + arrow_length);
        
        // Main arrow line
        painter.line_segment([arrow_base, arrow_tip], egui::Stroke::new(2.0, color));
        
        // Arrow head
        let head_size = 4.0;
        let left_wing = egui::Pos2::new(center_x - head_size, arrow_y + head_size);
        let right_wing = egui::Pos2::new(center_x + head_size, arrow_y + head_size);
        
        painter.line_segment([arrow_tip, left_wing], egui::Stroke::new(2.0, color));
        painter.line_segment([arrow_tip, right_wing], egui::Stroke::new(2.0, color));
    }
    
    fn draw_top_arrow(&self, painter: &egui::Painter, center_x: f32, arrow_y: f32, arrow_length: f32, color: egui::Color32) {
        let arrow_tip = egui::Pos2::new(center_x, arrow_y + arrow_length);
        let arrow_base = egui::Pos2::new(center_x, arrow_y);
        
        // Main arrow line
        painter.line_segment([arrow_base, arrow_tip], egui::Stroke::new(2.0, color));
        
        // Arrow head (pointing down)
        let head_size = 4.0;
        let left_wing = egui::Pos2::new(center_x - head_size, arrow_y + arrow_length - head_size);
        let right_wing = egui::Pos2::new(center_x + head_size, arrow_y + arrow_length - head_size);
        
        painter.line_segment([arrow_tip, left_wing], egui::Stroke::new(2.0, color));
        painter.line_segment([arrow_tip, right_wing], egui::Stroke::new(2.0, color));
    }
    
    fn draw_left_arrow(&self, painter: &egui::Painter, arrow_x: f32, center_y: f32, arrow_length: f32, color: egui::Color32) {
        let arrow_base = egui::Pos2::new(arrow_x, center_y);
        let arrow_tip = egui::Pos2::new(arrow_x + arrow_length, center_y);
        
        // Main arrow line
        painter.line_segment([arrow_base, arrow_tip], egui::Stroke::new(2.0, color));
        
        // Arrow head (pointing right)
        let head_size = 4.0;
        let top_wing = egui::Pos2::new(arrow_x + arrow_length - head_size, center_y - head_size);
        let bottom_wing = egui::Pos2::new(arrow_x + arrow_length - head_size, center_y + head_size);
        
        painter.line_segment([arrow_tip, top_wing], egui::Stroke::new(2.0, color));
        painter.line_segment([arrow_tip, bottom_wing], egui::Stroke::new(2.0, color));
    }
    
    fn draw_right_arrow(&self, painter: &egui::Painter, arrow_x: f32, center_y: f32, arrow_length: f32, color: egui::Color32) {
        let arrow_base = egui::Pos2::new(arrow_x + arrow_length, center_y);
        let arrow_tip = egui::Pos2::new(arrow_x, center_y);
        
        // Main arrow line
        painter.line_segment([arrow_base, arrow_tip], egui::Stroke::new(2.0, color));
        
        // Arrow head (pointing left)
        let head_size = 4.0;
        let top_wing = egui::Pos2::new(arrow_x + head_size, center_y - head_size);
        let bottom_wing = egui::Pos2::new(arrow_x + head_size, center_y + head_size);
        
        painter.line_segment([arrow_tip, top_wing], egui::Stroke::new(2.0, color));
        painter.line_segment([arrow_tip, bottom_wing], egui::Stroke::new(2.0, color));
    }
    
    fn update_text_selection(&mut self) {
        self.selected_text_elements.clear();
        
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let selection_rect = egui::Rect::from_two_pos(start, end);
            
            for (idx, text_element) in self.text_elements.iter().enumerate() {
                let lines: Vec<&str> = text_element.text.lines().collect();
                let font_size = text_element.font_size;
                let line_height = font_size * 1.2;
                
                for (line_idx, line) in lines.iter().enumerate() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    let line_y = text_element.position.y + (line_idx as f32) * line_height;
                    let estimated_text_width = line.len() as f32 * font_size * 0.6;
                    
                    let text_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(text_element.position.x, line_y),
                        egui::Vec2::new(estimated_text_width, font_size),
                    );
                    
                    if selection_rect.intersects(text_rect) && !self.selected_text_elements.contains(&idx) {
                        self.selected_text_elements.push(idx);
                        break; // Only need to add the text element once
                    }
                }
            }
        }
    }
    
    fn get_text_element_at_position(&self, pos: egui::Pos2) -> Option<usize> {
        for (idx, text_element) in self.text_elements.iter().enumerate() {
            let lines: Vec<&str> = text_element.text.lines().collect();
            let font_size = text_element.font_size;
            let line_height = font_size * 1.2;
            
            for (line_idx, line) in lines.iter().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                
                let line_y = text_element.position.y + (line_idx as f32) * line_height;
                let estimated_text_width = line.len() as f32 * font_size * 0.6;
                
                let text_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(text_element.position.x, line_y),
                    egui::Vec2::new(estimated_text_width, font_size),
                );
                
                if text_rect.contains(pos) {
                    return Some(idx);
                }
            }
        }
        None
    }
    
    fn copy_selected_text_to_clipboard(&mut self) -> bool {
        if self.selected_text_elements.is_empty() {
            return false;
        }
        
        let mut combined_text = String::new();
        for &text_idx in &self.selected_text_elements {
            if let Some(text_element) = self.text_elements.get(text_idx) {
                if !combined_text.is_empty() {
                    combined_text.push('\n');
                }
                combined_text.push_str(&text_element.text);
            }
        }
        
        if let Some(ref mut clipboard) = self.clipboard {
            if let Ok(()) = clipboard.set_text(combined_text) {
                return true;
            }
        }
        false
    }
    
    fn drag_selected_text(&mut self, current_pos: egui::Pos2) {
        // Calculate the offset from the initial drag position
        if let Some(start_pos) = self.selection_start {
            let offset = current_pos - start_pos;
            
            // Apply offset to all selected text elements
            for &text_idx in &self.selected_text_elements {
                if let Some(text_element) = self.text_elements.get_mut(text_idx) {
                    text_element.position = text_element.position + offset;
                }
            }
            
            // Update the drag start position for next frame
            self.selection_start = Some(current_pos);
        }
    }
    
    // === FILE OPERATIONS ===
    
    fn save_project(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Scribble Project", &["scribble"])
            .set_file_name("my_drawing.scribble")
            .save_file()
        {
            let project = ScribbleProject {
                strokes: self.strokes.iter().map(|s| SerializableStroke {
                    points: s.points.iter().map(|p| (p.x, p.y)).collect(),
                    color: (s.color.r(), s.color.g(), s.color.b()),
                    width: s.width,
                }).collect(),
                text_elements: self.text_elements.iter().map(|t| SerializableTextElement {
                    position: (t.position.x, t.position.y),
                    text: t.text.clone(),
                    font_size: t.font_size,
                }).collect(),
                canvas_size: (800.0, 600.0), // Default canvas size
            };
            
            let json = serde_json::to_string_pretty(&project)?;
            fs::write(path, json)?;
        }
        Ok(())
    }
    
    fn load_project(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Scribble Project", &["scribble"])
            .pick_file()
        {
            let json = fs::read_to_string(path)?;
            let project: ScribbleProject = serde_json::from_str(&json)?;
            
            // Clear current content
            self.strokes.clear();
            self.text_elements.clear();
            self.current_stroke.clear();
            self.is_drawing = false;
            self.selected_text_elements.clear();
            self.is_selecting_text = false;
            self.selection_start = None;
            self.selection_end = None;
            self.search_results.clear();
            self.search_query.clear();
            
            // Load strokes
            self.strokes = project.strokes.into_iter().map(|s| Stroke {
                points: s.points.into_iter().map(|(x, y)| egui::Pos2::new(x, y)).collect(),
                color: egui::Color32::from_rgb(s.color.0, s.color.1, s.color.2),
                width: s.width,
            }).collect();
            
            // Load text elements
            self.text_elements = project.text_elements.into_iter().map(|t| TextElement {
                position: egui::Pos2::new(t.position.0, t.position.1),
                text: t.text,
                font_size: t.font_size,
            }).collect();
        }
        Ok(())
    }
    
    fn export_svg(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SVG Image", &["svg"])
            .set_file_name("my_drawing.svg")
            .save_file()
        {
            let mut svg = String::new();
            
            // SVG header with light grey background
            svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600">"#);
            svg.push('\n');
            
            // Background
            svg.push_str(r#"<rect width="800" height="600" fill="rgb(245,245,245)"/>"#);
            svg.push('\n');
            
            // Export strokes as paths
            for stroke in &self.strokes {
                if stroke.points.len() > 1 {
                    svg.push_str(&format!(
                        r#"<path d="M{},{}"#,
                        stroke.points[0].x, stroke.points[0].y
                    ));
                    
                    for point in &stroke.points[1..] {
                        svg.push_str(&format!(" L{},{}", point.x, point.y));
                    }
                    
                    svg.push_str(&format!(
                        r#"" stroke="rgb({},{},{})" stroke-width="{}" fill="none" stroke-linecap="round" stroke-linejoin="round"/>"#,
                        stroke.color.r(), stroke.color.g(), stroke.color.b(),
                        stroke.width
                    ));
                    svg.push('\n');
                }
            }
            
            // Export text elements
            for text_element in &self.text_elements {
                // Handle multiline text
                let lines: Vec<&str> = text_element.text.lines().collect();
                for (line_idx, line) in lines.iter().enumerate() {
                    if !line.trim().is_empty() {
                        let line_y = text_element.position.y + text_element.font_size + (line_idx as f32 * text_element.font_size * 1.2);
                        svg.push_str(&format!(
                            r#"<text x="{}" y="{}" font-size="{}" font-family="monospace" fill="black">{}</text>"#,
                            text_element.position.x,
                            line_y,
                            text_element.font_size,
                            Self::html_escape(line)
                        ));
                        svg.push('\n');
                    }
                }
            }
            
            svg.push_str("</svg>");
            fs::write(path, svg)?;
        }
        Ok(())
    }
    
    fn export_png(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Note: PNG export would require rendering to an image buffer
        // This is a placeholder - full implementation would need image crate
        if let Some(_path) = rfd::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .set_file_name("my_drawing.png")
            .save_file()
        {
            // TODO: Implement PNG export
            // Would require:
            // 1. Create image buffer
            // 2. Render all strokes and text to buffer  
            // 3. Save as PNG using image crate
            eprintln!("PNG export not yet implemented - use SVG export instead");
        }
        Ok(())
    }
    
    fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
    
    fn detect_arrow_collisions(&mut self, painter: &egui::Painter) {
        self.text_collisions.clear();
        
        if self.search_results.is_empty() {
            return;
        }
        
        // For each text element with search results, check if arrows would collide with other text
        for &search_index in &self.search_results {
            if search_index >= self.text_elements.len() {
                continue;
            }
            
            let search_element = &self.text_elements[search_index];
            let positions = self.get_match_positions(&search_element.text);
            
            for (start_char, end_char) in positions {
                let font_id = egui::FontId::proportional(search_element.font_size);
                
                // Calculate arrow area (simplified version of the arrow drawing logic)
                let lines: Vec<&str> = search_element.text.lines().collect();
                let line_height = painter.layout_no_wrap(
                    "Ag".to_string(),
                    font_id.clone(),
                    egui::Color32::WHITE,
                ).size().y;
                
                let mut char_count = 0;
                let mut match_line = 0;
                let mut match_start_in_line = start_char;
                
                for (line_idx, line) in lines.iter().enumerate() {
                    let line_len = line.len() + 1;
                    if char_count + line_len > start_char {
                        match_line = line_idx;
                        match_start_in_line = start_char - char_count;
                        break;
                    }
                    char_count += line_len;
                }
                
                if match_line < lines.len() {
                    let current_line = lines[match_line];
                    let match_end_in_line = (end_char - char_count).min(current_line.len());
                    let line_y = search_element.position.y + (match_line as f32 * line_height);
                    
                    let before_match = &current_line[..match_start_in_line];
                    let match_text = &current_line[match_start_in_line..match_end_in_line];
                    
                    let before_galley = painter.layout_no_wrap(
                        before_match.to_string(),
                        font_id.clone(),
                        egui::Color32::WHITE,
                    );
                    let match_galley = painter.layout_no_wrap(
                        match_text.to_string(),
                        font_id.clone(),
                        egui::Color32::WHITE,
                    );
                    
                    let match_start_x = search_element.position.x + before_galley.size().x;
                    let match_end_x = match_start_x + match_galley.size().x;
                    let text_bottom = line_y + match_galley.size().y;
                    
                    // Define arrow area (arrows appear below text)
                    let arrow_area = egui::Rect::from_min_max(
                        egui::Pos2::new(match_start_x - 10.0, text_bottom),
                        egui::Pos2::new(match_end_x + 10.0, text_bottom + 25.0), // Arrow height
                    );
                    
                    // Check collision with other text elements
                    for (other_index, other_element) in self.text_elements.iter().enumerate() {
                        if other_index == search_index {
                            continue;
                        }
                        
                        // Estimate text area for collision detection
                        let other_lines: Vec<&str> = other_element.text.lines().collect();
                        let other_line_height = painter.layout_no_wrap(
                            "Ag".to_string(),
                            egui::FontId::proportional(other_element.font_size),
                            egui::Color32::WHITE,
                        ).size().y;
                        
                        // Calculate approximate text bounds
                        let max_line_width = other_lines.iter()
                            .map(|line| {
                                painter.layout_no_wrap(
                                    line.to_string(),
                                    egui::FontId::proportional(other_element.font_size),
                                    egui::Color32::WHITE,
                                ).size().x
                            })
                            .fold(0.0, f32::max);
                        
                        let text_area = egui::Rect::from_min_size(
                            other_element.position,
                            egui::Vec2::new(
                                max_line_width,
                                other_line_height * other_lines.len() as f32,
                            ),
                        );
                        
                        if arrow_area.intersects(text_area) {
                            self.text_collisions.push(other_index);
                        }
                    }
                }
            }
        }
        
        // Remove duplicates
        self.text_collisions.sort();
        self.text_collisions.dedup();
    }
}

impl eframe::App for ScribbleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Top controls
            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    self.strokes.clear();
                    self.text_elements.clear();
                    self.current_stroke.clear();
                    self.is_drawing = false;
                    self.text_input.clear();
                    self.active_text_position = None;
                    self.search_results.clear();
                    self.search_query.clear();
                    // Clear selection state
                    self.selected_text_elements.clear();
                    self.is_selecting_text = false;
                    self.selection_start = None;
                    self.selection_end = None;
                }
                
                ui.separator();
                
                // File operations
                ui.menu_button("📁 File", |ui| {
                    if ui.button("💾 Save Project").clicked() {
                        if let Err(e) = self.save_project() {
                            eprintln!("Save error: {}", e);
                        }
                        ui.close_menu();
                    }
                    
                    if ui.button("📂 Load Project").clicked() {
                        if let Err(e) = self.load_project() {
                            eprintln!("Load error: {}", e);
                        }
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("📤 Export SVG").clicked() {
                        if let Err(e) = self.export_svg() {
                            eprintln!("SVG export error: {}", e);
                        }
                        ui.close_menu();
                    }
                    
                    if ui.button("📸 Export PNG").clicked() {
                        if let Err(e) = self.export_png() {
                            eprintln!("PNG export error: {}", e);
                        }
                        ui.close_menu();
                    }
                });
                
                ui.separator();
                
                // Search toggle button
                if ui.button(if self.show_search { "🔍 Hide Search" } else { "🔍 Search" }).clicked() {
                    self.show_search = !self.show_search;
                    if !self.show_search {
                        self.search_results.clear();
                        self.search_query.clear();
                        self.search_error = None;
                    }
                }
                
                ui.separator();
                
                // Tool selection
                ui.label("Tool:");
                ui.selectable_value(&mut self.current_tool, Tool::Draw, "✏️ Draw");
                ui.selectable_value(&mut self.current_tool, Tool::Text, "📝 Text");
                ui.selectable_value(&mut self.current_tool, Tool::Select, "🔍 Select");
                
                ui.separator();
                
                if self.current_tool == Tool::Draw {
                    ui.label("Stroke width:");
                    ui.add(egui::Slider::new(&mut self.stroke_width, 1.0..=10.0));
                } else if self.current_tool == Tool::Text {
                    ui.label("Font size:");
                    ui.add(egui::Slider::new(&mut self.text_font_size, 10.0..=50.0));
                } else if self.current_tool == Tool::Select {
                    ui.label("Selection tool active");
                    if !self.selected_text_elements.is_empty() {
                        ui.label(format!("Selected: {} text element(s)", self.selected_text_elements.len()));
                        
                        // Copy button
                        if ui.button("📋 Copy").clicked() {
                            if self.copy_selected_text_to_clipboard() {
                                // Could add a status message here if needed
                            }
                        }
                    }
                }
                
                ui.separator();
                
                ui.label("Color:");
                let mut color = [
                    self.stroke_color.r() as f32 / 255.0,
                    self.stroke_color.g() as f32 / 255.0, 
                    self.stroke_color.b() as f32 / 255.0,
                ];
                if ui.color_edit_button_rgb(&mut color).changed() {
                    self.stroke_color = egui::Color32::from_rgb(
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                    );
                }
                
                ui.separator();
                
                ui.label(format!("Strokes: {} | Text: {}", self.strokes.len(), self.text_elements.len()));
            });
            
            // Search bar (only shown when search is enabled)
            if self.show_search {
                ui.horizontal(|ui| {
                    ui.label("🔍 Search:");
                    
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Type to search text elements...")
                            .desired_width(200.0)
                    );
                    
                    ui.checkbox(&mut self.regex_mode, "Regex");
                    
                    if search_response.changed() {
                        self.perform_search();
                    }
                    
                    if ui.button("Clear Search").clicked() {
                        self.search_query.clear();
                        self.search_results.clear();
                        self.search_error = None;
                    }
                    
                    // Show search results count
                    if !self.search_query.is_empty() {
                        if let Some(error) = &self.search_error {
                            ui.colored_label(egui::Color32::RED, error);
                        } else {
                            let total_matches = self.get_total_match_count();
                            ui.colored_label(
                                egui::Color32::GREEN,
                                format!("Found {} matches", total_matches)
                            );
                        }
                    }
                });
                ui.separator();
            }

            ui.separator();            // Drawing area
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::click_and_drag(),
            );
            
            // Draw faded grey background
            let canvas_rect = response.rect;
            painter.rect_filled(
                canvas_rect,
                egui::Rounding::ZERO,
                egui::Color32::from_rgb(245, 245, 245), // Light grey background
            );
            
            // Detect arrow collisions before drawing
            self.detect_arrow_collisions(&painter);
            
            // Handle mouse input based on selected tool
            if self.current_tool == Tool::Draw {
                // Drawing logic
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    if response.drag_started() {
                        self.is_drawing = true;
                        self.current_stroke.clear();
                        self.current_stroke.push(pointer_pos);
                    } else if self.is_drawing && response.dragged() {
                        self.current_stroke.push(pointer_pos);
                    }
                }
                
                if response.drag_stopped() {
                    if self.is_drawing && self.current_stroke.len() > 1 {
                        self.strokes.push(Stroke {
                            points: self.current_stroke.clone(),
                            color: self.stroke_color,
                            width: self.stroke_width,
                        });
                    }
                    self.current_stroke.clear();
                    self.is_drawing = false;
                }
            } else if self.current_tool == Tool::Text {
                // Text placement logic
                if response.clicked() {
                    if let Some(pointer_pos) = response.interact_pointer_pos() {
                        self.active_text_position = Some(pointer_pos);
                        self.text_input.clear();
                        // Request focus for the text input that will appear
                        ui.memory_mut(|mem| mem.request_focus(self.text_input_id));
                    }
                }
            } else if self.current_tool == Tool::Select {
                // Text selection and dragging logic
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    if response.drag_started() {
                        // Check if we clicked on a selected text element to start dragging
                        let clicked_element = self.get_text_element_at_position(pointer_pos);
                        if let Some(element_idx) = clicked_element {
                            if self.selected_text_elements.contains(&element_idx) {
                                // Start dragging selected elements, don't start selection
                                self.selection_start = Some(pointer_pos);
                                self.is_selecting_text = false;
                            } else {
                                // Clicked on unselected text, start new selection
                                self.is_selecting_text = true;
                                self.selection_start = Some(pointer_pos);
                                self.selection_end = Some(pointer_pos);
                                self.selected_text_elements.clear();
                            }
                        } else {
                            // Clicked in empty space, start new selection
                            self.is_selecting_text = true;
                            self.selection_start = Some(pointer_pos);
                            self.selection_end = Some(pointer_pos);
                            self.selected_text_elements.clear();
                        }
                    } else if response.dragged() {
                        if self.is_selecting_text {
                            // Update selection area
                            self.selection_end = Some(pointer_pos);
                            self.update_text_selection();
                        } else if !self.selected_text_elements.is_empty() {
                            // Handle dragging of selected text
                            self.drag_selected_text(pointer_pos);
                        }
                    }
                    
                    // Clear selection on single click in empty space
                    if response.clicked() && self.get_text_element_at_position(pointer_pos).is_none() {
                        self.selected_text_elements.clear();
                    }
                }
                
                if response.drag_stopped() {
                    if self.is_selecting_text {
                        self.is_selecting_text = false;
                        self.update_text_selection();
                    }
                }
            }
            
            // Show floating text input if active
            if let Some(text_pos) = self.active_text_position {
                let text_area = egui::Area::new(egui::Id::new("floating_text_area"))
                    .fixed_pos(text_pos)
                    .order(egui::Order::Foreground);
                
                text_area.show(ctx, |ui| {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Type your text (multiline supported):");
                            
                            let text_edit_response = ui.add(
                                egui::TextEdit::multiline(&mut self.text_input)
                                    .id(self.text_input_id)
                                    .desired_width(250.0)
                                    .desired_rows(5)
                                    .font(egui::TextStyle::Body)
                            );
                            
                            // Auto-focus the text input when it first appears
                            if text_edit_response.gained_focus() {
                                ui.memory_mut(|mem| mem.request_focus(self.text_input_id));
                            }
                            
                            ui.horizontal(|ui| {
                                if ui.button("✅ Add").clicked() {
                                    if !self.text_input.trim().is_empty() {
                                        self.text_elements.push(TextElement {
                                            position: text_pos,
                                            text: self.text_input.clone(),
                                            font_size: self.text_font_size,
                                        });
                                        self.text_input.clear();
                                        self.active_text_position = None;
                                    }
                                }
                                
                                if ui.button("❌ Cancel").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                    self.active_text_position = None;
                                    self.text_input.clear();
                                }
                            });
                            
                            ui.label("Ctrl+Enter to add, Esc to cancel");
                            
                            // Handle Ctrl+Enter to add text
                            if ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl) {
                                if !self.text_input.trim().is_empty() {
                                    self.text_elements.push(TextElement {
                                        position: text_pos,
                                        text: self.text_input.clone(),
                                        font_size: self.text_font_size,
                                    });
                                    self.text_input.clear();
                                    self.active_text_position = None;
                                }
                            }
                        });
                    });
                });
            }
            
            // Draw completed strokes
            for stroke in &self.strokes {
                if stroke.points.len() > 1 {
                    let points: Vec<egui::Pos2> = stroke.points.iter().copied().collect();
                    painter.add(egui::Shape::line(
                        points,
                        egui::Stroke::new(stroke.width, stroke.color),
                    ));
                }
            }
            
            // Draw selection rectangle if actively selecting
            if self.is_selecting_text {
                if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                    let selection_rect = egui::Rect::from_two_pos(start, end);
                    painter.rect_stroke(
                        selection_rect,
                        egui::Rounding::ZERO,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 255)),
                    );
                    painter.rect_filled(
                        selection_rect,
                        egui::Rounding::ZERO,
                        egui::Color32::from_rgba_premultiplied(100, 150, 255, 30),
                    );
                }
            }
            
            // Draw text elements
            for (index, text_element) in self.text_elements.iter().enumerate() {
                let is_search_result = self.search_results.contains(&index);
                let has_collision = self.text_collisions.contains(&index);
                let is_selected = self.selected_text_elements.contains(&index);
                
                // Draw selection background if selected
                if is_selected {
                    let lines: Vec<&str> = text_element.text.lines().collect();
                    let font_size = text_element.font_size;
                    let line_height = font_size * 1.2;
                    
                    for (line_idx, line) in lines.iter().enumerate() {
                        if line.trim().is_empty() {
                            continue;
                        }
                        
                        let line_y = text_element.position.y + (line_idx as f32) * line_height;
                        let estimated_text_width = line.len() as f32 * font_size * 0.6;
                        
                        let selection_rect = egui::Rect::from_min_size(
                            egui::Pos2::new(text_element.position.x - 2.0, line_y - 2.0),
                            egui::Vec2::new(estimated_text_width + 4.0, font_size + 4.0),
                        );
                        
                        painter.rect_filled(
                            selection_rect,
                            egui::Rounding::same(3.0),
                            egui::Color32::from_rgba_premultiplied(100, 150, 255, 80), // Light blue selection
                        );
                    }
                }
                
                // Text is always black, but may be semi-transparent if there's a collision
                let text_color = if has_collision {
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, 128) // Semi-transparent black
                } else {
                    egui::Color32::BLACK // Always black for text
                };
                
                // Draw the text in its original form
                painter.text(
                    text_element.position,
                    egui::Align2::LEFT_TOP,
                    &text_element.text,
                    egui::FontId::proportional(text_element.font_size),
                    text_color,
                );
                
                // Draw arrows pointing to matches
                if is_search_result && !self.search_query.is_empty() {
                    self.draw_arrows_for_matches(
                        &painter,
                        text_element.position,
                        &text_element.text,
                        text_element.font_size,
                    );
                }
            }
            
            // Draw current stroke being drawn
            if self.current_stroke.len() > 1 {
                let points: Vec<egui::Pos2> = self.current_stroke.iter().copied().collect();
                painter.add(egui::Shape::line(
                    points,
                    egui::Stroke::new(self.stroke_width, egui::Color32::LIGHT_BLUE),
                ));
            }
            
            // Draw instructions if no content
            if self.strokes.is_empty() && self.text_elements.is_empty() && !self.is_drawing && self.active_text_position.is_none() {
                let text_pos = response.rect.center();
                let instruction_text = match self.current_tool {
                    Tool::Draw => "Click and drag to draw!",
                    Tool::Text => "Click to place text!",
                    Tool::Select => "Drag to select text, then drag selected text to move!\nUse the Copy button to copy selected text.",
                };
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    instruction_text,
                    egui::FontId::proportional(20.0),
                    egui::Color32::GRAY,
                );
            }
        });
    }
}
