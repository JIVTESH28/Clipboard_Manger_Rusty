use std::collections::VecDeque;
use std::time::Duration;

use eframe::egui;
use clipboard::{ClipboardProvider, ClipboardContext};


#[derive(Clone, Debug)]
struct ClipEntry {
    content: String,
    timestamp: std::time::SystemTime,
}

impl ClipEntry {
    fn new(content: String) -> Self {
        Self {
            content,
            timestamp: std::time::SystemTime::now(),
        }
    }
}


struct ClipboardStack {
    entries: VecDeque<ClipEntry>,
    max_size: usize,
}

impl ClipboardStack {
    fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_size,
        }
    }

    fn push(&mut self, entry: ClipEntry) {

        self.entries.retain(|e| e.content != entry.content);
        
        
        self.entries.push_front(entry);
        
        
        if self.entries.len() > self.max_size {
            self.entries.pop_back();
        }
    }

    fn get(&self, index: usize) -> Option<&ClipEntry> {
        self.entries.get(index)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn iter(&self) -> impl Iterator<Item = &ClipEntry> {
        self.entries.iter()
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}


struct ClipboardApp {
    clipboard_stack: ClipboardStack,
    clipboard_ctx: ClipboardContext,
    last_clipboard_content: String,
    monitor_interval: Duration,
    auto_monitor: bool,
    search_filter: String,
}

impl ClipboardApp {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let clipboard_stack = ClipboardStack::new(50);
        let mut clipboard_ctx: ClipboardContext = ClipboardProvider::new()?;
        
        
        let initial_content = clipboard_ctx.get_contents().unwrap_or_default();
        
        Ok(Self {
            clipboard_stack,
            clipboard_ctx,
            last_clipboard_content: initial_content,
            monitor_interval: Duration::from_millis(500),
            auto_monitor: true,
            search_filter: String::new(),
        })
    }

    fn monitor_clipboard(&mut self) {
        if !self.auto_monitor {
            return;
        }
        
        if let Ok(content) = self.clipboard_ctx.get_contents() {
            if !content.is_empty() && content != self.last_clipboard_content {
                let entry = ClipEntry::new(content.clone());
                self.clipboard_stack.push(entry);
                self.last_clipboard_content = content;
            }
        }
    }

    fn copy_to_clipboard(&mut self, content: &str) {
        if let Err(e) = self.clipboard_ctx.set_contents(content.to_string()) {
            eprintln!("Failed to set clipboard: {}", e);
        } else {
            self.last_clipboard_content = content.to_string();
        }
    }

    fn filtered_entries(&self) -> Vec<(usize, &ClipEntry)> {
        if self.search_filter.is_empty() {
            self.clipboard_stack.iter().enumerate().collect()
        } else {
            self.clipboard_stack
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    entry.content.to_lowercase().contains(&self.search_filter.to_lowercase())
                })
                .collect()
        }
    }
}

impl eframe::App for ClipboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        self.monitor_clipboard();
        
        
        ctx.request_repaint_after(self.monitor_interval);
        

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📋 Clipboard Manager");
            ui.separator();
            
            
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.auto_monitor, "Auto Monitor");
                ui.separator();
                
                if ui.button("🔄 Refresh").clicked() {
                    self.monitor_clipboard();
                }
                
                if ui.button("🗑️ Clear All").clicked() {
                    self.clipboard_stack.clear();
                }
                
                ui.separator();
                ui.label(format!("📊 {} items", self.clipboard_stack.len()));
            });
            
            ui.separator();
            
            
            ui.horizontal(|ui| {
                ui.label("🔍 Search:");
                ui.text_edit_singleline(&mut self.search_filter);
                if ui.button("✖").clicked() {
                    self.search_filter.clear();
                }
            });
            
            ui.separator();
            
            
            ui.collapsing("ℹ️ Instructions", |ui| {
                ui.label("• Copy text normally (Ctrl+C) - it will appear here automatically");
                ui.label("• Click 'Copy' button to copy item back to clipboard");
                ui.label("• Use search to filter through your clipboard history");
                ui.label("• Toggle 'Auto Monitor' to pause/resume clipboard monitoring");
            });
            
            ui.separator();
            
            
            let filtered_entries = self.filtered_entries();
            
            
            let mut entries_to_copy: Vec<String> = Vec::new();
            
            
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if filtered_entries.is_empty() {
                        if self.search_filter.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("📝 No clipboard history yet.\nCopy something to get started!");
                            });
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("🔍 No matches found for your search.");
                            });
                        }
                    } else {
                        for (original_index, entry) in &filtered_entries {
                            ui.group(|ui| {
                                
                                ui.horizontal(|ui| {
                                    ui.label(format!("#{}", original_index + 1));
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("📋 Copy").clicked() {
                                            entries_to_copy.push(entry.content.clone());
                                        }
                                        
                                        
                                        if let Ok(duration) = entry.timestamp.elapsed() {
                                            let seconds = duration.as_secs();
                                            let time_str = if seconds < 60 {
                                                format!("{}s ago", seconds)
                                            } else if seconds < 3600 {
                                                format!("{}m ago", seconds / 60)
                                            } else {
                                                format!("{}h ago", seconds / 3600)
                                            };
                                            ui.label(format!("🕒 {}", time_str));
                                        }
                                    });
                                });
                                
                                ui.separator();
                                
                                
                                let mut preview = if entry.content.len() > 200 {
                                    format!("{}...", &entry.content[..200])
                                } else {
                                    entry.content.clone()
                                };
                                
                                
                                ui.add(
                                    egui::TextEdit::multiline(&mut preview)
                                        .desired_rows(3)
                                        .desired_width(f32::INFINITY)
                                        .code_editor()
                                );
                                
                                
                                ui.horizontal(|ui| {
                                    ui.label(format!("📏 {} chars", entry.content.len()));
                                    ui.label(format!("📄 {} lines", entry.content.lines().count()));
                                });
                            });
                            
                            ui.add_space(5.0);
                        }
                    }
                });
            
            for content in entries_to_copy {
                self.copy_to_clipboard(&content);
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = ClipboardApp::new()?;
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 700.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Clipboard Manager",
        options,
        Box::new(|_cc| Box::new(app)),
    )?;
    
    Ok(())
}