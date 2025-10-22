#![feature(iter_array_chunks)]
#[macro_use]
mod cards;
mod eval;
mod gui;
mod preflop;
mod rank;
mod state;

use cards::*;
use egui::{Color32, RichText, Widget};
//use rand::prelude::*;
use gui::*;
use rank::*;
use state::*;

fn main() {
    let opts = eframe::NativeOptions {
        vsync: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Glow,
        centered: true,
        dithering: true,
        ..Default::default()
    };
    const MAX_PLAYERS: usize = 9;

    let mut pocket_cards_input = String::new();
    let mut board_cards_input = String::new();
    let mut players_at_table = 5usize;
    let mut players_in = 5usize;
    let mut state: Option<DeckState> = None;
    let mut pot_input = String::new();
    let mut pot = 0usize;
    let mut blind = 5usize;
    let mut blind_input = format!("{blind}");

    eframe::run_simple_native("Poker Solver", opts, move |ctx, _frame| {
        ctx.set_pixels_per_point(2.0);
        egui::CentralPanel::default().show(ctx, |ui| {
            text_entry(ui, "Cards in hand:", &mut pocket_cards_input);
            text_entry(ui, "Cards on board:", &mut board_cards_input);
            // Parse deck state
            if let (Some(pocket_cards), Some(board_cards)) = (
                parse_cards(&pocket_cards_input),
                parse_cards(&board_cards_input),
            ) && pocket_cards.len() == 2
                && [0, 3, 4, 5].contains(&board_cards.len())
            {
                state = Some(DeckState {
                    board: match board_cards.as_slice() {
                        [] => Board::PreFlop,
                        [a, b, c] => Board::Flop([*a, *b, *c]),
                        [a, b, c, d] => Board::Turn([*a, *b, *c, *d]),
                        [a, b, c, d, e] => Board::River([*a, *b, *c, *d, *e]),
                        _ => unreachable!(),
                    },
                    hand: [pocket_cards[0], pocket_cards[1]],
                });
            } else {
                ui.colored_label(Color32::RED, "Invalid card inputs");
            }
            ui.horizontal(|ui| {
                ui.label(format!("Players in the game:"));
                ui.label(
                    RichText::new(format!("{players_in}"))
                        .color(Color32::GREEN)
                        .underline(),
                );
                if ui
                    .add_enabled(players_in > 2, egui::Button::new("-").corner_radius(0))
                    .clicked()
                {
                    players_in = if players_in <= 2 { 2 } else { players_in - 1 }
                }
                if ui
                    .add_enabled(
                        players_in < players_at_table,
                        egui::Button::new("+").corner_radius(0),
                    )
                    .clicked()
                {
                    players_in = if players_in >= players_at_table {
                        players_at_table
                    } else {
                        players_in + 1
                    }
                };
                if ui
                    .add_enabled(
                        players_in != players_at_table,
                        egui::Button::new("max").corner_radius(0),
                    )
                    .clicked()
                {
                    players_in = players_at_table;
                }
            });
            labelled(
                ui,
                "Players at table:",
                egui::Slider::new(&mut players_at_table, 2..=MAX_PLAYERS).show_value(true),
            );
            players_in = players_in.clamp(2, players_at_table);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Chips in pot:");
                ui.label(
                    RichText::new(format!("{pot}"))
                        .color(Color32::GREEN)
                        .underline(),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Modify pot: ");
                let enabled = pot_input.trim().parse::<usize>().is_ok();
                let value = pot_input.trim().parse::<usize>().unwrap_or(0);
                if ui
                    .add_enabled(enabled, egui::Button::new("set").corner_radius(0))
                    .clicked()
                {
                    pot = value;
                }
                if ui
                    .add_enabled(enabled, egui::Button::new("-").corner_radius(0))
                    .clicked()
                {
                    pot = pot.saturating_sub(value);
                }
                if ui
                    .add_enabled(enabled, egui::Button::new("+").corner_radius(0))
                    .clicked()
                {
                    pot += value;
                }
                if ui
                    .add(egui::Button::new("blind").corner_radius(0))
                    .clicked()
                {
                    pot_input = format!("{blind}");
                }
                if !ui.text_edit_singleline(&mut pot_input).has_focus() && pot_input == "" {
                    pot_input = "0".to_string();
                }
            });
            text_entry(ui, "Blind:", &mut blind_input);
            if let Ok(value) = blind_input.parse::<usize>() {
                blind = value;
            }
            ui.separator();
        });
    })
    .unwrap();
}
