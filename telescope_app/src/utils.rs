pub fn color_for_status(status: u32) -> egui::Color32 {
    match status {
        100..=199 => egui::Color32::from_rgb(0, 155, 0), // green
        200..=299 => egui::Color32::from_rgb(0, 255, 0), // green
        300..=399 => egui::Color32::from_rgb(255, 255, 0), // yellow
        400..=499 => egui::Color32::from_rgb(255, 0, 0), // red
        500..=599 => egui::Color32::from_rgb(255, 0, 0), // red
        _ => egui::Color32::from_rgb(0, 0, 255), // blue, why is this here?
    }
}