use teloxide::{
    prelude::*, 
    types::{InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage},
};

#[derive(Default)]
struct TrainingSession {
    id: u8,
    activity: String,
    location: String,
    timing: String,
}

fn get_weekly_schedule() -> Vec<TrainingSession> {
    vec![
        TrainingSession { id: 1, activity: "Swim".into(), location: "USC Pool".into(), timing: "Monday".into() },
        TrainingSession { id: 2, activity: "Run".into(), location: "NUS Track".into(), timing: "Tuesday".into() },
        TrainingSession { id: 3, activity: "Swim".into(), location: "USC Pool".into(), timing: "Wednesday".into() },
        TrainingSession { id: 4, activity: "Run".into(), location: "NUS Track".into(), timing: "Thursday".into() },
        TrainingSession { id: 5, activity: "Swim".into(), location: "USC Pool".into(), timing: "Friday".into() },
        TrainingSession { id: 6, activity: "Bricks".into(), location: "Palawan Beach".into(), timing: "Saturday".into() },
    ]
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(send_menu))
        .branch(Update::filter_callback_query().endpoint(receive_callback));

    Dispatcher::builder(bot, handler).enable_ctrlc_handler().build().dispatch().await;
}

fn main_menu_keyboard(trainings: Vec<TrainingSession>) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = trainings
        .iter()
        .map(|s| {
            vec![InlineKeyboardButton::callback(
                format!("{}: {} @ {}", s.timing, s.activity, s.location),
                format!("checkin_{}", s.id),
            )]
        })
        .collect();

    InlineKeyboardMarkup::new(rows)
}

async fn send_menu(bot: Bot, msg: Message) -> ResponseResult<()> {

    bot.send_message(msg.chat.id, "Welcome to AquaTallyon! Choose an option:")
        .reply_markup(main_menu_keyboard(get_weekly_schedule()))
        .await?;

    Ok(())
}

async fn receive_callback(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
    let user_name = q.from.username.as_deref().unwrap_or("Friend");
    let trainings = get_weekly_schedule();

    let text = match q.data.as_deref() {
        Some(data) if data.starts_with("checkin_") => {
            let id_str = data.replace("checkin_", "");
            let session_name = trainings.iter()
                .find(|s| s.id.to_string() == id_str)
                .map(|s| format!("{} {}", s.timing, s.activity))
                .unwrap_or_else(|| "Training".to_string());
                
            format!("✅ @{} marked as attending: {}!", user_name, session_name)
        }
        _ => "Unknown action".to_string(),
    };

    if let Some(MaybeInaccessibleMessage::Regular(msg)) = q.message {
        bot.edit_message_text(msg.chat.id, msg.id, text)
            .reply_markup(main_menu_keyboard(get_weekly_schedule()))
            .await?;
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}