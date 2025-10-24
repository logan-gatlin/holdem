#![allow(unused_imports)]
#![feature(iter_array_chunks, array_try_from_fn)]
#[macro_use]
mod cards;
mod eval;
mod gui;
mod parse;
mod preflop;
mod rank;
mod state;

use cards::*;
use egui::{Color32, RichText, Widget};
use eval::*;
use gui::*;
use preflop::*;
use state::*;

use crate::parse::Parse;

fn main() {
    let opts = eframe::NativeOptions {
        vsync: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        dithering: true,
        viewport: egui::ViewportBuilder::default().with_inner_size((800.0, 1000.0)),
        ..Default::default()
    };
    const MAX_PLAYERS: usize = 9;
    const INITIAL_BET_BB: f64 = 2.5;

    let mut pocket_cards_input = String::new();
    let mut board_cards_input = String::new();
    let mut players_in = 5usize;
    let mut players_at_table = 5usize;
    let mut position = Position::default();
    let mut state: Option<DeckState> = None;
    let mut last_state: Option<DeckState> = None;
    let mut pot_input = String::new();
    let mut pot = 0usize;
    let mut blind = 5usize;
    let mut blind_input = format!("{blind}");
    let mut stack_input = String::new();
    let mut stack = 0usize;
    let mut call_price_input = String::new();
    let mut call_price = 0;
    let mut strength_calc_thread: Option<std::thread::JoinHandle<f64>> = None;
    let mut hand_strength: Option<f64> = None;

    eframe::run_simple_native("Poker Solver", opts, move |ctx, _frame| {
        ctx.set_pixels_per_point(1.0);
        egui::CentralPanel::default().show(ctx, |ui| {
            text_entry(ui, "Cards in hand:", &mut pocket_cards_input);
            text_entry(ui, "Cards on board:", &mut board_cards_input);
            // Parse deck state
            if let (Some(pocket_cards), Some(board_cards)) = (
                Vec::parse(&mut pocket_cards_input.chars().filter(|c| !c.is_whitespace())),
                Vec::parse(&mut board_cards_input.chars().filter(|c| !c.is_whitespace())),
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
                    .add_enabled(players_in > 2, egui::Button::new("-"))
                    .clicked()
                {
                    players_in = if players_in <= 2 { 2 } else { players_in - 1 }
                }
                if ui
                    .add_enabled(players_in < players_at_table, egui::Button::new("+"))
                    .clicked()
                {
                    players_in = if players_in >= players_at_table {
                        players_at_table
                    } else {
                        players_in + 1
                    }
                };
                if ui
                    .add_enabled(players_in != players_at_table, egui::Button::new("max"))
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
            ui.horizontal(|ui| {
                for pos in Position::with_n_players(players_at_table) {
                    if ui
                        .selectable_label(position == *pos, format!("{pos}"))
                        .clicked()
                    {
                        position = *pos;
                    }
                }
            });
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
                if ui.add_enabled(enabled, egui::Button::new("=")).clicked() {
                    pot = value;
                }
                if ui.add_enabled(enabled, egui::Button::new("-")).clicked() {
                    pot = pot.saturating_sub(value);
                }
                if ui.add_enabled(enabled, egui::Button::new("+")).clicked() {
                    pot += value;
                }
                if ui.add(egui::Button::new("blind")).clicked() {
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
            text_entry(ui, "Call price:", &mut call_price_input);
            if let Ok(value) = call_price_input.parse::<usize>() {
                call_price = value;
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Chips in stack:");
                ui.label(
                    RichText::new(format!("{stack}"))
                        .color(Color32::GREEN)
                        .underline(),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Modify stack: ");
                let enabled = stack_input.trim().parse::<usize>().is_ok();
                let value = stack_input.trim().parse::<usize>().unwrap_or(0);
                if ui.add_enabled(enabled, egui::Button::new("=")).clicked() {
                    stack = value;
                }
                if ui
                    .add_enabled(
                        enabled && value <= stack && value != 0,
                        egui::Button::new("bet"),
                    )
                    .clicked()
                {
                    pot += value;
                    stack = stack.saturating_sub(value);
                }
                if ui
                    .add_enabled(stack != 0, egui::Button::new("all in"))
                    .clicked()
                {
                    pot += stack;
                    stack = 0;
                }
                if ui
                    .add_enabled(pot != 0, egui::Button::new("rake"))
                    .clicked()
                {
                    stack += pot;
                    pot = 0;
                }
                if !ui.text_edit_singleline(&mut stack_input).has_focus() && stack_input == "" {
                    stack_input = "0".to_string();
                }
            });
            ui.separator();
            if strength_calc_thread
                .as_ref()
                .is_some_and(|t| t.is_finished())
            {
                hand_strength = Some(strength_calc_thread.take().unwrap().join().unwrap());
                strength_calc_thread = None;
            }
            if ui
                .add_enabled(
                    strength_calc_thread.is_none() && state.is_some() && last_state != state,
                    egui::Button::new(if last_state.is_none() {
                        "calculate hand"
                    } else {
                        "recalculate hand"
                    }),
                )
                .clicked()
            {
                last_state = state;
                hand_strength = None;
                strength_calc_thread =
                    Some(std::thread::spawn(move || last_state.unwrap().strength()));
            }
            if let (Some(mut hand_strength), Some(state)) = (hand_strength, last_state) {
                ui.label(
                    RichText::new(format!(
                        "{} decision:",
                        match state.board {
                            Board::PreFlop => "Pre-flop",
                            Board::Flop(_) => "Flop",
                            Board::Turn(_) => "Turn",
                            Board::River(_) => "River",
                        }
                    ))
                    .color(Color32::ORANGE)
                    .underline(),
                );
                hand_strength = hand_strength.powi(players_in as i32 - 1);
                let pot_odds = call_price as f64 / (pot + call_price) as f64;
                let expected_value =
                    (pot as f64 * hand_strength) - (call_price as f64 * (1.0 - hand_strength));
                let opponent_strength = (1.0 - hand_strength) / (players_in - 1) as f64;
                if state.board == Board::PreFlop {
                    if position != Position::BigBlind {
                        ui.horizontal(|ui| {
                            ui.label("Opening move:");
                            let range = position.gto_preflop();
                            let recommendation = match range[state.hand] {
                                Recommendation::Fold => "Fold".to_string(),
                                Recommendation::Call => "Call BB".to_string(),
                                Recommendation::Raise => {
                                    let raise_amount = blind as f64 * INITIAL_BET_BB;
                                    if (raise_amount + call_price as f64) > stack as f64 {
                                        "All in".to_string()
                                    } else {
                                        format!("Raise {:.0}", blind as f64 * INITIAL_BET_BB)
                                    }
                                }
                                Recommendation::AllIn => "All in".to_string(),
                            };
                            ui.label(
                                RichText::new(recommendation)
                                    .color(Color32::GREEN)
                                    .underline(),
                            );
                        });
                    }
                }
                if expected_value < 0.0 {
                    ui.label(RichText::new("Fold to this bet amount").color(Color32::RED));
                } else if expected_value > pot as f64 / 2.0 {
                    let raise_amount = pot as f64;
                    ui.label(
                        RichText::new(if (raise_amount + call_price as f64) > stack as f64 {
                            "All in".to_string()
                        } else {
                            format!("Raise {raise_amount:.0}")
                        })
                        .color(Color32::GREEN)
                        .underline(),
                    );
                } else if expected_value > pot as f64 / 4.0 {
                    let raise_amount = pot as f64 / 2.0;
                    ui.label(
                        RichText::new(if (raise_amount + call_price as f64) > stack as f64 {
                            "All in".to_string()
                        } else {
                            format!("Raise {raise_amount:.0}")
                        })
                        .color(Color32::GREEN)
                        .underline(),
                    );
                } else {
                    ui.label(RichText::new("Call this bet amount").color(Color32::GREEN));
                }
                ui.separator();
                ui.label(
                    RichText::new("Stats for this hand:")
                        .color(Color32::ORANGE)
                        .underline(),
                );
                ui.horizontal(|ui| {
                    ui.label("Hand strength:");
                    ui.label(
                        RichText::new(format!("{:.1}%", hand_strength * 100.0))
                            .color(Color32::GREEN)
                            .underline(),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Pot odds:");
                    ui.label(
                        RichText::new(format!("{:.1}%", pot_odds * 100.0))
                            .color(Color32::GREEN)
                            .underline(),
                    );
                });
                labelled(
                    ui,
                    "Expected value after call:",
                    egui::Label::new(
                        RichText::new(format!("{expected_value:+.0}"))
                            .color(Color32::GREEN)
                            .underline(),
                    ),
                );
                labelled(
                    ui,
                    "Opponent strength:",
                    egui::Label::new(
                        RichText::new(format!("{:.1}", opponent_strength * 100.0))
                            .color(Color32::GREEN)
                            .underline(),
                    ),
                );
            } else if strength_calc_thread.is_some() {
                ui.spinner();
            }
        });
    })
    .unwrap();
}
