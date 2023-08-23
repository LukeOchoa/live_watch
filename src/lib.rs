use core::panic;
use std::path::PathBuf;
use walkdir::WalkDir;

pub type MagicError = Box<dyn std::error::Error>;
pub mod err_tools {
    #[derive(Debug)]
    pub struct ErrorX {
        details: String,
    }

    impl ErrorX {
        pub fn new(msg: &str) -> ErrorX {
            ErrorX {
                details: msg.to_string(),
            }
        }
        pub fn new_box(msg: &str) -> Box<ErrorX> {
            Box::new(ErrorX {
                details: msg.to_string(),
            })
        }
    }

    impl std::fmt::Display for ErrorX {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.details)
        }
    }

    impl std::error::Error for ErrorX {
        fn description(&self) -> &str {
            &self.details
        }
    }
}

use std::env;
pub fn get_args() -> Result<Vec<String>, MagicError> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("_______________________\n\n\n\n\n\n_______________");
        return Err(err_tools::ErrorX::new_box(&format!(
            "You need to give 2 arguments if you want to pass a file!"
        )));
    }

    Ok(args)
}

pub fn find_file_path(file_name: &String) -> Option<PathBuf> {
    for maybe_entry in WalkDir::new(file_name) {
        if let Ok(entry) = maybe_entry {
            return Some(entry.into_path());
        }
    }

    None
}

pub fn validate_file_existence(file_name: String) -> Result<(), String> {
    for maybe_entry in WalkDir::new(file_name.clone()) {
        if let Ok(_) = maybe_entry {
            return Ok(());
        }
    }

    Err(format!(
        "No File present by the name of <{}> in the current directory...!",
        file_name
    ))
}

pub fn get_file(file_name: &String) -> Result<String, MagicError> {
    match find_file_path(file_name) {
        Some(path) => {
            let file = std::fs::read(path)?;
            let file = String::from_utf8(file).unwrap();
            Ok(file)
        }
        None => Err(err_tools::ErrorX::new_box(&format!(
            "File <{}> could not be located!",
            file_name
        ))),
    }
}

#[derive(PartialEq)]
enum LineSeparation {
    Off,
    AlphaNumeric,
    All,
}

pub struct LiveWatch {
    filename: String,
    file: Arc<Mutex<String>>,
    font_size: egui::FontId,
    watcher: notify::INotifyWatcher,
    word_wrap: bool,
    line_separation: LineSeparation,
    selectable_text_mode: bool,
}

impl LiveWatch {
    fn risky_get_fstring(&self) -> String {
        self.file.try_lock().unwrap().clone()
    }
}

impl Default for LiveWatch {
    fn default() -> Self {
        let filename = get_args().unwrap().remove(1);

        match validate_file_existence(filename.clone()) {
            Ok(_) => {
                println!("This is a filename: <{}>", filename);
            }
            Err(_) => {
                println!("This is Not a filename");
                panic!("Broken");
            }
        }

        let (watcher, file) = file_watcher(filename.clone());
        let font_size = egui::FontId::proportional(30.0);
        let word_wrap = true;
        let line_separation = LineSeparation::Off;
        let selectable_text_mode = false;

        Self {
            filename,
            file,
            watcher,
            font_size,
            word_wrap,
            line_separation,
            selectable_text_mode,
        }
    }
}

use notify::{RecursiveMode, Watcher};
use std::sync::{Arc, Mutex};

fn file_watcher(filename: String) -> (notify::INotifyWatcher, Arc<Mutex<String>>) {
    let file = Arc::new(Mutex::new(String::new()));
    let file_arc = Arc::clone(&file);

    *file.try_lock().unwrap() = get_file(&filename).unwrap();
    let watcher = notify::recommended_watcher(move |res| match res {
        Ok(_event) => {
            let mut x = file_arc.try_lock().unwrap();
            let d = get_file(&filename).unwrap();
            *x = d;
        }
        Err(e) => {
            panic!("{}", e);
        }
    })
    .unwrap();

    (watcher, file)
}

fn has_alphanumeric(buffer: &Vec<char>) -> bool {
    let mut booly = false;
    for chr in buffer {
        // if chr.is_ascii_alphanumeric() {
        // booly = true;
        // break;
        // }
        if *chr != ' ' || *chr != '\n' {
            booly = true;
            break;
        }
    }

    booly
}

fn do_seperation(lw: &LiveWatch, buffer: Vec<char>, ui: &mut egui::Ui) {
    let text = buffer.into_iter().collect::<String>();
    let rich_text = make_rich(text, lw.font_size.clone());
    display_text(lw.word_wrap, rich_text, ui);
    ui.separator();
}

fn make_separate(lw: &LiveWatch, ui: &mut egui::Ui) {
    let mut buffer = Vec::new();
    for chr in lw.risky_get_fstring().chars() {
        if chr == '\n' {
            buffer.push(chr);

            match lw.line_separation {
                LineSeparation::AlphaNumeric => {
                    if has_alphanumeric(&buffer) {
                        do_seperation(lw, buffer, ui);
                    }
                }
                LineSeparation::All => do_seperation(lw, buffer, ui),
                _ => {}
            }

            // Reset buffer
            buffer = Vec::new();

            // ui seperator
        } else {
            buffer.push(chr);
        }
    }
}

fn display_text(word_wrap: bool, text: impl Into<egui::WidgetText>, ui: &mut egui::Ui) {
    ui.add(egui::Label::new(text).wrap(word_wrap));
}

fn make_rich(string: String, font_size: egui::FontId) -> egui::RichText {
    egui::RichText::new(string).font(font_size)
}

fn separation_button(ui: &mut egui::Ui, lw: &mut LiveWatch, option: LineSeparation, text: &str) {
    if ui
        .add(egui::RadioButton::new(lw.line_separation == option, text))
        .clicked()
    {
        if lw.selectable_text_mode {
            lw.selectable_text_mode = false;
        }
        if lw.line_separation == option {
            lw.line_separation = LineSeparation::Off
        } else {
            lw.line_separation = option
        }
    }
}

// Notes:
// To Use this app, you need to provide a full filename as a cmd arg
impl eframe::App for LiveWatch {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // FILE-NAME Setting
            let rich_text = egui::RichText::new(format!("File Name: <{}>", self.filename))
                .font(self.font_size.clone());
            ui.label(rich_text);

            // OPTIONS
            egui::Grid::new(0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Font Options
                    egui::introspection::font_id_ui(ui, &mut self.font_size);
                });
                ui.end_row();

                ui.horizontal(|ui| {
                    // Word Wrap Option ===========
                    if ui.radio(self.word_wrap, "Word Wrap").clicked() {
                        self.word_wrap = !self.word_wrap;
                    }

                    // Line Seperater Option ===========
                    separation_button(ui, self, LineSeparation::AlphaNumeric, "Separate Lines");
                    separation_button(ui, self, LineSeparation::All, "Separate All Lines");

                    // Temp: Selectedable Text mode (LineSeparation will be turned off. I think it has to...? currently at this code patch of egui
                    if ui
                        .radio(self.selectable_text_mode, "Highlight/Copyable Mode")
                        .clicked()
                    {
                        self.selectable_text_mode = !self.selectable_text_mode;

                        if self.selectable_text_mode {
                            self.line_separation = LineSeparation::Off;
                        }
                    }
                });
            });
            ui.separator();
            //---------

            // TEXT BODY
            egui::ScrollArea::both().show(ui, |ui| {
                match self.line_separation {
                    LineSeparation::Off => {
                        if !self.selectable_text_mode {
                            let rich_text = egui::RichText::new(self.risky_get_fstring())
                                .font(self.font_size.clone());
                            ui.add(egui::Label::new(rich_text).wrap(self.word_wrap));
                        } else {
                            ui.add(egui::TextEdit::multiline(&mut self.risky_get_fstring()));
                        }
                    }
                    _ => {
                        make_separate(self, ui);
                    }
                }

                let result = self.watcher.watch(
                    std::path::Path::new(&self.filename),
                    RecursiveMode::Recursive,
                );

                ui.allocate_space(ui.available_size());

                match result {
                    Ok(_) => {
                        // println!("There is ok?")
                    }
                    Err(e) => {
                        println!("There is nothing: <{}>", e)
                    }
                }
            });
            ctx.request_repaint();
        });
    }
}
