use default_packs::create_default_packs;
use poise::serenity_prelude::{self as serenity, CreateEmbed};
use sqlx::{Pool, Sqlite, SqlitePool};
use s3::{creds::Credentials, region::Region, Bucket};

use crate::cards::card::Card;

mod cards;
mod paginate_cards;
mod commands;
mod create_user;
mod default_packs;

pub struct Data(Pool<Sqlite>, redis::Client, Bucket);
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub fn create_card_embed(e: &mut CreateEmbed, card: Card) -> &mut CreateEmbed {
    let mut image_url = std::env::var("S3_URL").unwrap();
    image_url.push_str(&format!("/tcrd/{}.{}", card.id, card.extension));
    e.title(card.id.clone())
    .field("", &format!(
        "**ID:** {}
        **Rarity:** {}
        **Type:** {}
        **HP:** {}
        **Damage:** {}
        **Defense:** {}",
        card.id, card.rarity, card.kind, card.hp, card.damage, card.defense
    ), false)
    .field("**Description**", card.description, false)
    .image(image_url)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Loads dotenv file
    let _ = dotenv::dotenv();

    // Object storage
    let username = std::env::var("S3_USERNAME").expect("Expected an s3 username in the environment");
    let password = std::env::var("S3_PASSWORD").expect("Expected an s3 password in the environment");
    let creds = Credentials::new(Some(&username), Some(&password), None, None, None).unwrap();
    let bucket = Bucket::new(
        "tcrd",
        Region::Custom {
            region: "my-store".to_owned(),
            endpoint: std::env::var("S3_URL").expect("Expected an s3 url in the environment"),
        },
        creds,
    )
    .unwrap()
    .with_path_style();

    // DB
    let database_url = "sqlite://tcrd.db";
    let conn = SqlitePool::connect(database_url).await?;
    sqlx::migrate!().run(&conn).await?;
    create_default_packs(&conn).await?;

    // Redis
    let redis_client = redis::Client::open(std::env::var("REDIS_URL").expect("Expected a redis url in the environment"))?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::manage::manage(), commands::cards::cards(), commands::balances::hourly(), commands::balances::balance(), commands::balances::daily(), commands::packs::pack(), commands::fight::fight(), commands::profile::profile(), commands::leaderboard::leaderboards()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                if let Ok(var) = std::env::var("GUILD_ID") {
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId(var.parse().expect("GUILD_ID should be an integer"))).await?;
                }
                else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                Ok(Data(conn, redis_client, bucket))
            })
        });

    framework.run().await?;
    Ok(())
}