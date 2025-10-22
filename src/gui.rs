pub fn labelled(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>, widget: impl egui::Widget) {
    ui.horizontal(|ui| {
        let label = ui.label(label);
        ui.add(widget).labelled_by(label.id);
    });
}

pub fn text_entry(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>, output: &mut String) {
    ui.horizontal(|ui| {
        let label = ui.label(label);
        ui.add_sized(ui.available_size(), egui::TextEdit::singleline(output))
            .labelled_by(label.id);
    });
}
