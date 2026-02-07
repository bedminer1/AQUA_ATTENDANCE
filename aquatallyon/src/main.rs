use teloxide::{
    prelude::*, 
    types::{ InlineKeyboardMarkup, MaybeInaccessibleMessage},
};
use std::sync::{Arc, RwLock};

mod types;
use crate::types::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting aquathallyon bot...");

    let initial_state = WeeklyAttendance {
        start_date: "2026-02-02".into(),
        end_date: "2026-02-08".into(),
        sessions: vec![
            TrainingSession { id: 1, activity: "Swim".into(), location: "USC Pool".into(), day: "Monday".into(), attendees: vec![] },
            TrainingSession { id: 2, activity: "Run".into(), location: "NUS Track".into(), day: "Tuesday".into(), attendees: vec![] },
            TrainingSession { id: 3, activity: "Swim".into(), location: "USC Pool".into(), day: "Wednesday".into(), attendees: vec![] },
            TrainingSession { id: 4, activity: "Run".into(), location: "NUS Track".into(), day: "Thursday".into(), attendees: vec![] },
            TrainingSession { id: 5, activity: "Swim".into(), location: "USC Pool".into(), day: "Friday".into(), attendees: vec![] },
            TrainingSession { id: 6, activity: "Bricks".into(), location: "Palawan Beach".into(), day: "Saturday".into(), attendees: vec![] },
        ],
    };

    let mut state = Arc::new(initial_state);
    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(send_menu));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn main_menu_keyboard(trainings: &[TrainingSession]) -> InlineKeyboardMarkup {
    let rows = trainings
        .iter()
        .map(|s| vec![s.make_button()])
        .collect::<Vec<_>>();

    InlineKeyboardMarkup::new(rows)
}

fn generate_attendance_report(state: &WeeklyAttendance) -> String {
    let header = format!("📅 <b>Week: {} to {}</b>\n\n", state.start_date, state.end_date);
    
    let body = state.sessions.iter().map(|s| {
        let attendees = if s.attendees.is_empty() {
            "<i>No one yet</i>".to_string()
        } else {
            let list = s.attendees.iter()
                .map(|u| format!("@{}", u.alias))
                .collect::<Vec<_>>()
                .join(", ");
            format!("└ {}", list)
        };
        
        format!("<b>{} {}</b> (@ {})\n{}\n", s.day, s.activity, s.location, attendees)
    }).collect::<Vec<_>>().join("\n");

    format!("{}{}", header, body)
}

async fn send_menu(
    bot: Bot,
    mut state: SharedState,
    msg: Message
) -> ResponseResult<()> {
    let week = Arc::make_mut(&mut state);
    let text = generate_attendance_report(&week);
    println!("{}", text);

    bot.send_message(msg.chat.id, text)
        .parse_mode(teloxide::types::ParseMode::Html) // Use V2 for better formatting
        .reply_markup(main_menu_keyboard(&week.sessions))
        .await?;

    Ok(())
}
