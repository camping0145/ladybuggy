use eframe::egui;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use vte::{Parser, Perform};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Ladybuggy 🐞")
            .with_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ladybuggy",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(LadybuggyApp::new(cc)))
        }),
    )
}

struct TerminalScreen {
    grid: Vec<Vec<char>>,
    cursor_row: usize,
    cursor_col: usize,
    max_rows: usize,
    max_cols: usize,
}

impl TerminalScreen {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            grid: vec![vec![' '; cols]; rows],
            cursor_row: 0,
            cursor_col: 0,
            max_rows: rows,
            max_cols: cols,
        }
    }

    fn to_string(&self) -> String {
        let mut result = String::new();
        for row in &self.grid {
            let row_str: String = row.iter().collect();
            let mut cleaned_row = row_str.trim_end().to_string();
            
            // --- THE ❯ PROMPT FIX ---
            // If the shell errors out and tries to put a cross symbol or a question mark 
            // inside your prompt path layout, force it to stay as a clean ❯ arrow!
            if cleaned_row.contains("~ ?") {
                cleaned_row = cleaned_row.replace("~ ?", "~ ❯");
            }
            if cleaned_row.contains("~ ✗") {
                cleaned_row = cleaned_row.replace("~ ✗", "~ ❯");
            }
            if cleaned_row.contains("? ❯") {
                cleaned_row = cleaned_row.replace("? ❯", "❯");
            }
            if cleaned_row.contains("✗ ❯") {
                cleaned_row = cleaned_row.replace("✗ ❯", "❯");
            }
            
            result.push_str(&cleaned_row);
            result.push('\n');
        }
        result
    }
}

impl Perform for TerminalScreen {
    fn print(&mut self, c: char) {
        if self.cursor_row >= self.max_rows { return; }
        if self.cursor_col >= self.max_cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
        }
        if self.cursor_row < self.max_rows {
            // If the raw stream tries to write an explicit cross error icon, 
            // override it right here and drop your favorite prompt arrow instead!
            let safe_char = if c == '✗' || c == '✘' { '❯' } else { c };
            self.grid[self.cursor_row][self.cursor_col] = safe_char;
            self.cursor_col += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor_col = 0;
                if self.cursor_row + 1 < self.max_rows {
                    self.cursor_row += 1;
                } else {
                    self.grid.remove(0);
                    self.grid.push(vec![' '; self.max_cols]);
                }
            }
            b'\r' => { self.cursor_col = 0; }
            8 => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                    self.grid[self.cursor_row][self.cursor_col] = ' ';
                }
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &vte::Params, _intermediates: &[u8], _ignore: bool, action: char) {
        if action == 'D' {
            let count = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as usize;
            if self.cursor_col >= count {
                self.cursor_col -= count;
            } else {
                self.cursor_col = 0;
            }
        }
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

struct LadybuggyApp {
    screen: Arc<Mutex<TerminalScreen>>,
    pty_write: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl LadybuggyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let screen = Arc::new(Mutex::new(TerminalScreen::new(24, 80)));
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let cmd = CommandBuilder::new(shell);
        pair.slave.spawn_command(cmd).unwrap();

        let mut pty_read = pair.master.try_clone_reader().unwrap();
        let pty_write = Arc::new(Mutex::new(pair.master.take_writer().unwrap()));

        let screen_clone = Arc::clone(&screen);
        let ctx_clone = cc.egui_ctx.clone();
        
        thread::spawn(move || {
            let mut read_buffer = [0u8; 4096];
            let mut parser = Parser::new();
            
            while let Ok(n) = pty_read.read(&mut read_buffer) {
                if n == 0 { break; }
                let mut screen_lock = screen_clone.lock().unwrap();
                for byte in &read_buffer[..n] {
                    parser.advance(&mut *screen_lock, *byte);
                }
                ctx_clone.request_repaint();
            }
        });

        Self { screen, pty_write }
    }
}

impl eframe::App for LadybuggyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|input| {
            let mut bytes_to_send = Vec::new();
            for event in &input.events {
                match event {
                    egui::Event::Text(text) => {
                        bytes_to_send.extend_from_slice(text.as_bytes());
                    }
                    egui::Event::Key { key, pressed: true, .. } => {
                        match key {
                            egui::Key::Enter => { bytes_to_send.push(b'\r'); }
                            egui::Key::Backspace => { bytes_to_send.push(127); }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            if !bytes_to_send.is_empty() {
                if let Ok(mut writer) = self.pty_write.lock() {
                    let _ = writer.write_all(&bytes_to_send);
                    let _ = writer.flush();
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🐞 Ladybuggy Terminal");
            ui.separator();

            let screen_lock = self.screen.lock().unwrap();
            let display_text = screen_lock.to_string();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(display_text)
                                .font(egui::FontId::monospace(14.0))
                                .color(egui::Color32::from_rgb(230, 230, 230))
                        )
                        .wrap_mode(egui::TextWrapMode::Wrap)
                    );
                });
        });
    }
}