use reqwest::Url;
use serde::Deserialize;
use std::{collections::HashMap, error::Error, str::FromStr};
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Deserialize)]
struct DogInfo {
    message: String,
    status: String,
}

#[derive(Deserialize)]
struct GoingeckoCoinValue {
    usd: f32,
}

async fn get_random_dog() -> Result<DogInfo, reqwest::Error> {
    reqwest::get("https://dog.ceo/api/breeds/image/random")
        .await?
        .json::<DogInfo>()
        .await
}

async fn get_random_dog_from_breed(breed: &str) -> Result<DogInfo, reqwest::Error> {
    reqwest::get(format!("https://dog.ceo/api/breed/{}/images/random", breed))
        .await?
        .json::<DogInfo>()
        .await
}

async fn get_euro_usd() -> Result<Option<f32>, reqwest::Error> {
    let res = reqwest::get(
        "https://api.coingecko.com/api/v3/simple/price?ids=tether-eurt&vs_currencies=usd",
    )
    .await?
    .json::<HashMap<String, GoingeckoCoinValue>>()
    .await?;

    let euro = res.get("tether-eurt");

    if let Some(euro) = euro {
        Ok(Some(euro.usd))
    } else {
        Ok(None)
    }
}

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Random dog")]
    Doggo,

    #[command(description = "Random dog from the specified breed")]
    Breed(String),

    #[command(description = "Get the value of EURO in USD")]
    Euro,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting the bot...");

    let bot = Bot::from_env().auto_send();

    teloxide::commands_repl(bot, answer, Command::ty()).await;
}

async fn answer(
    bot: AutoSend<Bot>,
    message: Message,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Doggo => {
            info!("Fetching a random dog...");

            let dog = get_random_dog().await;

            if let Ok(dog) = dog {
                if dog.status == "success" {
                    let url = Url::from_str(&dog.message).unwrap();
                    let res = bot.send_photo(message.chat.id, InputFile::url(url)).await;
                    if let Err(e) = res {
                        error!("Error while sending message {:?} ", e);
                    } else {
                        info!("Dog sent with success");
                    }
                } else {
                    error!("Could not find a dog");
                }
            } else {
                error!("Could not find a dog");
            }
        }
        Command::Euro => {
            let euro = get_euro_usd().await;

            if let Ok(Some(euro)) = euro {
                let res = bot.send_message(message.chat.id, format!("${}", euro)).await;
                if let Err(e) = res {
                    error!("Error while sending message {:?} ", e);
                } else {
                    info!("Dog sent with success");
                }
            } else if let Err(e) = euro {
                error!("Could not fetch the value of Euro -> {}", e);
            }
        }
        Command::Breed(breed) => {
            info!("Fetching a random dog...");

            let dog = get_random_dog_from_breed(&breed).await;

            if let Ok(dog) = dog {
                if dog.status == "success" {
                    let url = Url::from_str(&dog.message).unwrap();
                    let res = bot.send_photo(message.chat.id, InputFile::url(url)).await;
                    if let Err(e) = res {
                        error!("Error while sending message {:?} ", e);
                    } else {
                        info!("Dog sent with success");
                    }
                } else {
                    error!("Could not find a dog");
                }
            } else {
                error!("Could not find a dog");
            }
        }
    };

    Ok(())
}
