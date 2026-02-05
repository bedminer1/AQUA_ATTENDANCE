use teloxide::{
    prelude::*, 
    utils::command::BotCommands,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage},
};

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

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "more information on bot commands.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}

async fn send_menu(bot: Bot, msg: Message) -> ResponseResult<()> {
    let buttons = [
        InlineKeyboardButton::callback("Help", "help"),
        InlineKeyboardButton::callback("Greet Me", "greet"),
    ];

    bot.send_message(msg.chat.id, "Welcome to AquaTallyon! Choose an option:")
        .reply_markup(InlineKeyboardMarkup::new([buttons]))
        .await?;

    Ok(())
}

async fn receive_callback(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
    // 1. Get the username of the person who pressed the button
    let user_name = q.from.username.as_deref().unwrap_or("Friend");
    
    // 2. Determine the new text based on which button was pressed ("data")
    let text = match q.data.as_deref() {
        Some("help") => "This bot tracks attendance for NUS Aquathlon. Click Greet to test!".to_string(),
        Some("greet") => format!("Hello, @{}! Ready for the set?", user_name),
        _ => "Unknown action".to_string(),
    };

    if let Some(MaybeInaccessibleMessage::Regular(msg)) = q.message {
        bot.edit_message_text(msg.chat.id, msg.id, text)
            .await?;
    }

    // 4. Answer the callback query so the "loading" spinner stops on Telegram
    bot.answer_callback_query(q.id).await?;
    Ok(())
}

fn make_back_button() -> InlineKeyboardMarkup {
    let button = InlineKeyboardButton::callback("« Back to Menu", "main_menu");
    InlineKeyboardMarkup::new([[button]])
}