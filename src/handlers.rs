use teloxide::{
    prelude::*,
    types::{ InlineKeyboardMarkup },
    utils::command::BotCommands,
};
use chrono::{Duration, Local, Datelike};

use crate::types::*;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
pub enum Command {
    #[command(description = "Show this help menu")]
    Help,

    // --- CLUB MANAGEMENT (EXCO ONLY) ---
    #[command(description = "Clear all attendance for the next week")]
    NewWeek,

    #[command(description = "Add a session: /add <order>, <day>, <activity>, <location>, <time>")]
    Add(String),

    #[command(description = "Delete a session by its order number: /delete <order>")]
    Delete(u8),

    #[command(description = "Override a session: /edit <order>, <day>, <activity>, <location>, <time>")]
    Edit(String),

    #[command(description = "Sync current state to permanent storage")]
    Save,

    // --- PERSONAL LOGGING (FOR MEMBERS) ---
    #[command(description = "Log a personal workout: /log <activity>, <distance>, <duration>")]
    Log(String),

    #[command(description = "View your personal training history")]
    History,
}

pub async fn handle_commands(
    bot: Bot,
    state: AppState,
    msg: Message,
    cmd: Command,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            let help_text = "<b>🔱 Aquathallyon Bot Help</b>\n\n\
                <b>👥 Member Commands</b>\n\
                /history - View your training logs\n\
                /log - Record a personal workout\n\n\
                <b>🛠️ Management (EXCO)</b>\n\
                /new_week - Reset all lists for next week\n\
                /add - Create a new training session\n\
                /edit - Modify an existing session\n\
                /delete - Remove a session\n\
                /save - Sync current data to Turso\n\n\
                <i>Tip: Use commas to separate arguments for /add and /edit. \n The format is /(command) (order), (day), (activity), (location), (time)</i>";
            bot.send_message(msg.chat.id, help_text)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
        }
        Command::NewWeek => {
            let (report, kb) = {
                let mut weekly_attendance = state.sync_state.write();

                let now = Local::now().date_naive();
                let days_to_next_monday = (7 - now.weekday().num_days_from_monday()) % 7;
                let next_monday = now + Duration::days(days_to_next_monday as i64);
                let next_sunday = next_monday + Duration::days(6);

                weekly_attendance.start_date = next_monday.format("%d/%m").to_string();
                weekly_attendance.end_date = next_sunday.format("%d/%m").to_string();

                // TODO: save state to DB before wiping state

                for session in &mut weekly_attendance.sessions {
                    session.attendees.clear();
                }

                (generate_attendance_report(&weekly_attendance), main_menu_keyboard(&weekly_attendance.sessions))
            };

            bot.send_message(msg.chat.id, format!("{}", report))
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(kb)
                .await?;
        }
        Command::Save => {
            let week_snapshot = state.sync_state.read().clone();

            // 2. Clear current attendance in DB
            state.db.execute("DELETE FROM attendance", ()).await.unwrap();

            // 3. Batch insert the new state
            for session in week_snapshot.sessions {
                for attendee in session.attendees {
                    if !attendee.cancelled {
                        let alias = week_snapshot.user_registry
                            .get(&attendee.user_id)
                            .map(|u| u.alias.as_str())
                            .unwrap_or("Unknown");

                        state.db.execute(
                            "INSERT INTO attendance (session_id, user_id, user_alias) VALUES (?, ?, ?)",
                            libsql::params![session.id, attendee.user_id, alias],
                        ).await.unwrap();
                    }
                }
            }

            bot.send_message(msg.chat.id, "✅ Attendance successfully synced to Turso!").await?;
        }
        Command::Edit(raw_args) => {
            let parts: Vec<&str> = raw_args.split(',').map(|s| s.trim()).collect();

            if parts.len() < 5 {
                bot.send_message(msg.chat.id, "❌ Format: /edit order, day, activity, location, time").await?;
                return Ok(());
            }

            let order: usize = parts[0].parse().unwrap_or(0);
            let day = parts[1].to_string();
            let activity = parts[2].to_string();
            let location = parts[3].to_string();
            let time = parts[4].to_string();

            let (report, kb, success) = {
                let mut week = state.sync_state.write();
                let index = order.saturating_sub(1);

                if index < week.sessions.len() {
                    let session = &mut week.sessions[index];
                    session.day = day;
                    session.activity = activity;
                    session.location = location;
                    session.time = time;

                    (generate_attendance_report(&week), main_menu_keyboard(&week.sessions), true)
                } else {
                    (String::new(), InlineKeyboardMarkup::default(), false)
                }
            };

            if success {
                bot.send_message(msg.chat.id, format!("📝 <b>Session #{} updated.</b>\n\n{}", order, report))
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .reply_markup(kb)
                    .await?;
            } else {
                bot.send_message(msg.chat.id, format!("⚠️ Session #{} not found.", order)).await?;
            }
        }
        Command::Add(raw_args) => {
            let parts: Vec<&str> = raw_args.split(',').map(|s| s.trim()).collect();

            if parts.len() < 5 {
                bot.send_message(msg.chat.id, "❌ Format: /add order, day, activity, location, time").await?;
                return Ok(());
            }

            let order: usize = parts[0].parse().unwrap_or(1);
            let day = parts[1].to_string();
            let activity = parts[2].to_string();
            let location = parts[3].to_string();
            let time = parts[4].to_string();

            let (report, kb) = {
                let mut week = state.sync_state.write();
                        let next_id = week.sessions.iter().map(|s| s.id).max().unwrap_or(0) + 1;
                        let new_session = TrainingSession {
                            id: next_id,
                            activity,
                            location,
                            day,
                            attendees: vec![],
                            time,
                        };

                        // Insert at specific position or push
                        if order > 0 && order <= week.sessions.len() {
                            week.sessions.insert(order - 1, new_session);
                        } else {
                            week.sessions.push(new_session);
                        }

                        (generate_attendance_report(&week), main_menu_keyboard(&week.sessions))
            };

            bot.send_message(msg.chat.id, format!("{}", report))
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(kb)
                .await?;
        }
        Command::Delete(order) => {
            let (report, kb, success) = {
                let mut weekly_attendance = state.sync_state.write();

                // 1. Convert 1-indexed order to 0-indexed index
                // Use saturating_sub to handle potential 0 input safely
                let index = (order as usize).saturating_sub(1);

                // 2. Check if the index is within the bounds of the vector
                if index < weekly_attendance.sessions.len() {
                    // Remove returns the removed element, but we just need it gone
                    weekly_attendance.sessions.remove(index);

                    (
                        generate_attendance_report(&weekly_attendance),
                        main_menu_keyboard(&weekly_attendance.sessions),
                        true
                    )
                } else {
                    // Out of bounds - nothing to delete
                    (
                        generate_attendance_report(&weekly_attendance),
                        main_menu_keyboard(&weekly_attendance.sessions),
                        false
                    )
                }
            };

            if success {
                bot.send_message(msg.chat.id, format!("🗑️ <b>Session at order #{} deleted.</b>\n\n{}", order, report))
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .reply_markup(kb)
                    .await?;
            } else {
                bot.send_message(msg.chat.id, format!("⚠️ Order #{} not found. Check the list and try again.", order))
                    .await?;
            }
        }
        Command::History => {
            let (report, kb) = {
                let week = state.sync_state.read();
                (generate_attendance_report(&week), main_menu_keyboard(&week.sessions))
            };

            bot.send_message(msg.chat.id, format!("{}", report))
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(kb)
                .await?;
        }
        Command::Log(_raw_args) => {
            let (report, kb) = {
                let week = state.sync_state.read();
                (generate_log_report(&week), main_menu_keyboard(&week.sessions))
            };

            bot.send_message(msg.chat.id, format!("{}", report))
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(kb)
                .await?;
        }
    }

    Ok(())
}

pub async fn receive_btn_press(
    bot: Bot,
    state: AppState,
    q: CallbackQuery,
) -> ResponseResult<()> {
    let _user_name = q.from.username.clone().unwrap_or_else(|| "unknown".into());
    let display_name = q.from.full_name();
    let user_id = q.from.id.0;

    let (report_text, keyboard) = {
        let mut week = state.sync_state.write();

        let session_id = q.data.as_deref()
            .and_then(|data| data.strip_prefix("checkin_"))
            .and_then(|id_str| id_str.parse::<u8>().ok());

        // Update registry with latest display name
        week.user_registry.insert(user_id, UserProfile { alias: display_name });

        if let Some(session) = session_id.and_then(|id| week.get_session_mut(id)) {
            if let Some(attendee) = session.attendees.iter_mut().find(|a| a.user_id == user_id) {
                attendee.cancelled = !attendee.cancelled;
            } else {
                session.attendees.push(Attendee { user_id, cancelled: false });
            }
        }

        let text = generate_attendance_report(&week);
        let kb = main_menu_keyboard(&week.sessions);
        (text, kb)
    }; // RwLockWriteGuard drop

    if let Some(teloxide::types::MaybeInaccessibleMessage::Regular(msg)) = q.message {
        bot.edit_message_text(msg.chat.id, msg.id, report_text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}

fn main_menu_keyboard(trainings: &[TrainingSession]) -> InlineKeyboardMarkup {
    let rows = trainings
        .iter()
        .map(|s| vec![s.make_button()])
        .collect::<Vec<_>>();

    InlineKeyboardMarkup::new(rows)
}

fn generate_attendance_report(state: &WeeklyAttendance) -> String {
    let header = format!("📅 <b>Training Attendance {} to {}</b>\n\n", state.start_date, state.end_date);

    let body = state.sessions.iter().map(|s| {
        let attendees = if s.attendees.is_empty() {
            "<i>No one yet</i>".to_string()
        } else {
            let list = s.attendees.iter()
                .map(|a| {
                    let name = state.user_registry
                        .get(&a.user_id)
                        .map(|u| u.alias.as_str())
                        .unwrap_or("Unknown");

                    if a.cancelled {
                        format!("<s>{}</s>", name)
                    } else {
                        format!("{}", name)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}", list)
        };

        let count = s.attendees.iter().filter(|a| !a.cancelled).count();
        format!("<b>{} {}</b> @ {} ({} 👥)\n{}\n", s.day, s.activity, s.location, count, attendees)
    }).collect::<Vec<_>>().join("\n");

    format!("{}{}", header, body)
}

fn generate_log_report(state: &WeeklyAttendance) -> String {
    let header = format!("📅 <b>Training Log {} to {}</b>\n\n", state.start_date, state.end_date);

    let body = state.sessions.iter().map(|s| {
        let count = s.attendees.iter().filter(|a| !a.cancelled).count();
        format!("<b>{} {}</b> @ {} ({} 👥)\n", s.day, s.activity, s.location, count)
    }).collect::<Vec<_>>().join("\n");

    format!("{}{}", header, body)
}
