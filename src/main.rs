use teloxide::{
    prelude::*,
};

mod types;
mod handlers;
use crate::types::*;
use crate::handlers::*;
use chrono::{Duration, Local, Datelike};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting aquathallyon bot...");

    let now = Local::now().date_naive();
    let days_to_next_monday = (7 - now.weekday().num_days_from_monday()) % 7;
    let next_monday = now + Duration::days(days_to_next_monday as i64);
    let next_sunday = next_monday + Duration::days(6);
    let initial_state = WeeklyAttendance {
        start_date: next_monday.format("%d/%m").to_string(),
        end_date: next_sunday.format("%d/%m").to_string(),
        sessions: vec![
            TrainingSession { id: 1, activity: "Swim".into(), location: "USC Pool".into(), day: "Monday".into(), attendees: vec![], time: "5:00 PM".into() },
            TrainingSession { id: 2, activity: "Run".into(), location: "NUS Track".into(), day: "Tuesday".into(), attendees: vec![], time: "6:00 PM".into() },
            TrainingSession { id: 3, activity: "Swim".into(), location: "USC Pool".into(), day: "Wednesday".into(), attendees: vec![], time: "5:00 PM".into() },
            TrainingSession { id: 4, activity: "Run".into(), location: "NUS Track".into(), day: "Thursday".into(), attendees: vec![], time: "6:00 PM".into() },
            TrainingSession { id: 5, activity: "Swim".into(), location: "USC Pool".into(), day: "Friday".into(), attendees: vec![], time: "5:00 PM".into() },
            TrainingSession { id: 6, activity: "Bricks".into(), location: "Palawan Beach".into(), day: "Saturday".into(), attendees: vec![], time: "8:30 AM".into() },
        ],
        user_registry: std::collections::HashMap::new(),
    };

    let app_state = AppState::new(initial_state).await;
    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().filter_command::<Command>().endpoint(handle_commands))
        .branch(Update::filter_callback_query().endpoint(receive_btn_press));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
