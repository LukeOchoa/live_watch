use live_watch::LiveWatch;

fn main() {
    let option = eframe::NativeOptions::default();
    eframe::run_native(
        "live_watch",
        option,
        Box::new(|_cc| Box::new(LiveWatch::default())),
    )
    .unwrap();
}
