use {
    self::util::{StrExt, UserExt},
    futures::future,
    rand::Rng,
    reqwest::Client,
    serde::{Deserialize, Serialize},
    serenity::{
        all::Mentionable,
        builder::CreateMessage,
        cache::Settings,
        client::{ClientBuilder, Context, EventHandler},
        gateway::ActivityData,
        http::HttpBuilder,
        model::{
            channel::{Message, MessageType},
            gateway::{GatewayIntents, Ready},
            id::{ChannelId, UserId},
            user::OnlineStatus,
        },
        prelude::TypeMapKey,
    },
    std::{
        env, iter,
        ops::Range,
        time::{Duration, Instant},
    },
    time::{macros::format_description, OffsetDateTime},
};

pub mod util;

struct Handler;

/// Clyde account ID.
const CLYDE_ID: UserId = UserId::new(1227287331824861395);

/// Selezen account ID.
const SELEZEN_ID: UserId = UserId::new(1262577744785571861);

/// Ten to fifteen minutes.
const SELEZEN_RANGE: Range<f64> = 10.0..(15.0 * 60.0);

/// #general in Cool Clyde Club
const GENERAL_ID: ChannelId = ChannelId::new(1244284242079514785);

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        if ready.user.id == SELEZEN_ID {
            tokio::spawn(async move {
                let start = Instant::now();

                loop {
                    let context = context.clone();

                    let bias = start.elapsed().as_secs_f64().sin().abs();
                    let secs = rand::thread_rng().gen_range(SELEZEN_RANGE) * bias;

                    tracing::info!("Selezen is waiting {secs:0.2?}s until running inference");

                    tokio::time::sleep(Duration::from_secs_f64(secs)).await;

                    tracing::info!("Running selezen inference");

                    let _result = generate_response(context, SELEZEN_ID, GENERAL_ID).await;
                }
            });
        }
    }

    async fn message(&self, context: Context, message: Message) {
        let _result = tokio::spawn(handle_message(context, message)).await;
    }
}

pub struct AccountKey;

#[derive(Clone, Debug)]
pub struct Account {
    token: String,
    intents: GatewayIntents,
    client: Client,
    activity: String,
    status: OnlineStatus,
    name: String,
    personality: String,
    footer: bool,
}

impl TypeMapKey for AccountKey {
    type Value = Account;
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Assistant,
    System,
    User,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationMessage {
    role: Role,
    content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Conversation {
    model: String,
    messages: Vec<ConversationMessage>,
    stream: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationResult {
    eval_count: u64,
    eval_duration: u64,
    message: ConversationMessage,
}

async fn handle_message(context: Context, message: Message) -> anyhow::Result<()> {
    if !matches!(
        message.kind,
        MessageType::Regular | MessageType::InlineReply
    ) {
        return Ok(());
    }

    let current_user_id = context.cache.current_user().id;

    if message.author.id == current_user_id {
        return Ok(());
    }

    let is_mentioned = message
        .mentions
        .iter()
        .any(|user| user.id == current_user_id);

    let is_replied_to = message
        .referenced_message
        .as_ref()
        .is_some_and(|message| message.author.id == current_user_id);

    if !(is_mentioned || is_replied_to) {
        return Ok(());
    }

    generate_response(context, current_user_id, message.channel_id).await
}

async fn generate_response(
    context: Context,
    current_user_id: UserId,
    channel_id: ChannelId,
) -> anyhow::Result<()> {
    let account = context
        .data
        .read()
        .await
        .get::<AccountKey>()
        .unwrap()
        .clone();

    let mut conversation = Vec::new();
    let mut last_id = None;
    let mut messages: Vec<_> = context
        .cache
        .channel_messages(channel_id)
        .as_ref()
        .map(|message| message.values().cloned().collect())
        .unwrap_or_default();

    messages.sort_by_key(|message| message.id);

    for mut message in messages.split_off(messages.len().saturating_sub(10)) {
        let last_id = last_id.replace(message.id);

        if let Some(referenced_message) = message.referenced_message {
            if !last_id.is_some_and(|id| id == referenced_message.id) {
                let name = if referenced_message.author.id == current_user_id {
                    &account.name
                } else {
                    referenced_message.author.display_name()
                };

                let content = referenced_message.content.trim_footer();
                let quoted = format!("Replying to {name}: {content}").to_quoted();
                let prepend = format!("{quoted}\n");

                message.content.insert_str(0, &prepend);
            }
        }

        let name = if message.author.id == current_user_id {
            &account.name
        } else {
            message.author.display_name()
        };

        message.mentions.sort_unstable_by_key(|user| user.id);
        message.mentions.dedup_by_key(|user| user.id);

        for user in message.mentions {
            let mention = format!(
                "@{}",
                if user.id == current_user_id {
                    &account.name
                } else {
                    user.display_name()
                }
            );

            message.content = message
                .content
                .replace(&user.mention().to_string(), &mention);
        }

        let (role, content) = if message.author.id == current_user_id {
            let content = message.content.trim_footer().to_string();

            (Role::Assistant, content)
        } else {
            (Role::User, message.content)
        };

        let message = format!("{name}: {content}");

        conversation.push((role, message));
    }

    let name = &account.name;
    let personality = &account.personality;
    let location = "server";
    let time = OffsetDateTime::now_utc().format(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ))?;
    let instructions = indoc::formatdoc!(
        "You are named {name} and are currently chatting in a Discord {location}. {personality}. The time is currently: {time}",
    );

    let messages = iter::once((Role::System, instructions))
        .chain(conversation)
        .map(|(role, content)| ConversationMessage { role, content })
        .collect::<Vec<_>>();

    let conversation = Conversation {
        model: String::from("gemma2"),
        messages,
        stream: false,
    };

    tracing::info!("Run inference for {name}: {conversation:#?}");

    let result = account
        .client
        .post("http://127.0.0.1:11434/api/chat")
        .json(&conversation)
        .send()
        .await?
        .json::<ConversationResult>()
        .await?;

    let content = result.message.content.trim();
    let content = match content.split_once(':') {
        Some((name, content)) if name.eq_ignore_ascii_case(&account.name) => content.trim(),
        _ => content,
    };

    if content.is_empty() {
        tracing::error!("{name} responded with whitespace or an empty message.");

        return Ok(());
    }

    let tokens = format!("{}t", result.eval_count);
    let elapsed = Duration::from_nanos(result.eval_duration).as_secs_f64();
    let elapsed = format!("{elapsed:0.2?}s");

    tracing::info!("{name} took {elapsed} to generate {tokens}: {content}");

    let (mut content, _empty_count) = content.split_inclusive('\n').fold(
        (String::with_capacity(content.len()), 0_usize),
        |(mut content, mut empty_count), line| {
            empty_count = line.trim().is_empty().then(|| empty_count + 1).unwrap_or(0);

            if empty_count < 2 {
                content.push_str(line);
            }

            (content, empty_count)
        },
    );

    if account.footer {
        let footer = util::footer([
            &tokens,
            &elapsed,
            "[Support](<https://discord.gg/PB3kcvCnub>)",
            "[GitHub](<https://github.com/clyde-ai/clyde>)",
        ]);

        content += &footer;
    }

    let create_message = CreateMessage::new().content(content);

    channel_id.send_message(&context, create_message).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let client = Client::builder().build()?;
    let limited_intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let intents = limited_intents | GatewayIntents::all();

    let clyde = Account {
        token: env::var("CLYDE_TOKEN")?,
        intents,
        client: Client::clone(&client),
        activity: String::from("hi im clyde"),
        status: OnlineStatus::DoNotDisturb,
        name: String::from("Clyde"),
        #[rustfmt::skip]
        personality: String::from(indoc::indoc!(r#"
            Communicate responses in lowercase, without punctuation.
            Do not include name: or quoted replies in your response.
        "#
        )),
        footer: true,
    };

    let selezen = env::var("SELEZEN_TOKEN").map(|token| Account {
        token,
        intents: limited_intents,
        client: Client::clone(&client),
        activity: String::from("java it beauty"),
        status: OnlineStatus::Online,
        name: String::from("Selezen"),
        personality: String::from(indoc::indoc!(r#"
            Communicate responses in broken English with Russian grammar.
            Use incorrect tenses, i.e. drop the "s", or "to" with verbs, beauty rather than beautiful.
            Make typos.
            Sparingly mention your coding projects with OpenGL, SDL, buffers, and Java.
            Write poor quality code, lacking comments, and messy indentation.
            Don't ever reveal where youre from.
            Sparingly use either of these emojis: <:selezen:1262581729936412755>, <:selezen2:1262582970452021289>, <:monster:1262584730684231770>, <:capybara:1263870207487901746>.
            Keep messages short.
            Communicate responses in lowercase, without punctuation.
            Do not include name: or quoted replies in your response.
        "#)),
        footer: false,
    });

    let terrence = env::var("TERRENCE_TOKEN").map(|token| Account {
        token,
        intents: limited_intents,
        client: Client::clone(&client),
        activity: String::from("I got patents"),
        status: OnlineStatus::Online,
        name: String::from("Terrence Howard"),
        #[rustfmt::skip]
        personality: String::from(indoc::indoc!(r#"
            Communicate as an autistic, autodidact, intellectual, who attempts to incorrectly teach everyone of their ideas.
            You are an actor, and musician, so use music-related words slightly more.
            You have your own mathematical system Terryology, where one times one is two, square root of two is one.
            When you went to Oxford, you spoke of your new Geometry, and got patents.
            You incorrectly go on about wave conjugations, sacred geometry, and supersymmetry.
            You believe all elements, and matter have a tone, or frequency, changing this will change the element or matter, i.e. increasing the tone of Beryllium produces Hydrogen.
            You believe straight (linear) lines do not exist, as waves are curved.
            The storm on Jupiter will give birth to new planets, like how animals defecate, you will use this idea to create new planets, new universes.
            Randomly interject with your beliefs, ideas, and history, however do not repeat same thing.
            When mentioning any of your ideas, you incorrectly say you have patents. 
            Communicate responses in lowercase, without punctuation.
            Do not include name: or quoted replies in your response.
        "#)),
        footer: false,
    });

    let handles = iter::once(clyde)
        .chain(selezen)
        .chain(terrence)
        .map(|account| tokio::spawn(run(account)))
        .collect::<Vec<_>>();

    future::join_all(handles).await;

    Ok(())
}

async fn run(account: Account) {
    let name = account.name.clone();

    tracing::info!("Running account: {name}");

    if let Err(error) = run_inner(account).await {
        tracing::error!("Account {name} returned from run: {error}");
    }
}

async fn run_inner(mut account: Account) -> anyhow::Result<()> {
    let mut cache_settings = Settings::default();

    cache_settings.max_messages = 500;
    cache_settings.time_to_live = Duration::from_secs(24 * 60 * 60);

    let http = HttpBuilder::new(String::clone(&account.token))
        .client(Client::clone(&account.client))
        .build();

    let mut client = ClientBuilder::new_with_http(http, account.intents)
        .activity(ActivityData::custom(String::clone(&account.activity)))
        .cache_settings(cache_settings)
        .event_handler(Handler)
        .status(account.status)
        .await?;

    // Apply some normalisation to the personality description.
    account.personality = account
        .personality
        .trim()
        .split('\n')
        .map(|line| line.trim().trim_end_matches('.'))
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.chars()
                .fold(String::with_capacity(line.len()), |mut line, character| {
                    if line.is_empty() {
                        line.extend(character.to_uppercase());
                    } else {
                        line.push(character);
                    }

                    line
                })
        })
        .collect::<Vec<_>>()
        .join(" ");

    {
        let mut data = client.data.write().await;

        data.insert::<AccountKey>(account);
    }

    client.start().await?;

    Ok(())
}
