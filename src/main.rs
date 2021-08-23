mod netease;

use dotenv::dotenv;
use serde_json::Value;
use std::env;
use std::error::Error;
use teloxide::{prelude::*, utils::command::BotCommand};

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "支持这些命令")]
enum Command {
    #[command(description = "显示这条信息")]
    Help,
    #[command(description = "搜索音乐")]
    Music(String),
    #[command(description = "音乐推荐")]
    MusicRecommend,
}

async fn answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Help => cx.answer(Command::descriptions()).await?,

        Command::Music(name) => {
            if name.is_empty() {
                cx.answer("歌曲名称不能为空").await?;
                return Ok(());
            }
            cx.answer(format!("歌曲名称是：{}，正在搜索…", name))
                .await?;
            let resp = netease::api::search(&name).await.text().await.unwrap();
            let value: Value = serde_json::from_str(&resp).expect("failed to deserialize json");
            let id = &value["result"]["songs"][0]["id"];
            let resp = netease::api::song_url(&id.to_string())
                .await
                .text()
                .await
                .unwrap();
            let value: Value = serde_json::from_str(&resp).expect("failed to deserialize json");
            cx.answer(format!("url: {}", &value["data"][0]["url"]))
                .await?
        }

        _ => cx.answer(Command::descriptions()).await?,
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    teloxide::enable_logging!();
    log::info!("Starting simple_commands_bot...");

    let bot = Bot::from_env().auto_send();
    teloxide::commands_repl(bot, env::var("BOT_NAME").unwrap(), answer).await;
}
