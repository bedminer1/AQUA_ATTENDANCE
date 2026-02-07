use teloxide::{
    prelude::*, 
    types::{ InlineKeyboardMarkup, MaybeInaccessibleMessage, Me},
};

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

    let state = SharedState::new(initial_state);
    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(send_menu));
        // .branch(Update::filter_callback_query().endpoint(receive_btn_press));

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
    state: SharedState,
    msg: Message
) -> ResponseResult<()> {
    let (text, keyboard) = {
        let weekly_attendance = state.read();
        let t = generate_attendance_report(&weekly_attendance);
        let k = main_menu_keyboard(&weekly_attendance.sessions);
        (t, k) // Return the data we need, dropping the guard here
    };
    
    bot.send_message(msg.chat.id, text)
        .parse_mode(teloxide::types::ParseMode::Html) // Use V2 for better formatting
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

// async fn receive_btn_press(
//     bot: Bot,
//     mut state: SharedState,
//     q: CallbackQuery
// ) -> ResponseResult<()> {
//     let user_name = q.from.username.as_deref().unwrap_or("unknown user");
//     let user_id = q.from.id.0;

//     let mut response_text = "Issue with callback".to_string();

//     let text = match q.data.as_deref() {
//         Some(data) if data.starts_with("checkin_") => {
//             let id = data.replace("checkin_", "");
//             let mut week = state
//         }
//         _ => "Issue with callback".to_string(),
//     };

//     if let Some(MaybeInaccessibleMessage::Regular(msg)) = q.message {
//         bot.edit_message_text(msg.chat.id, msg.id, text)
//             .reply_markup(main_menu_keyboard(&state.sessions))
//             .await?;
//     }

//     bot.answer_callback_query(q.id).await?;
//     Ok(())
// }