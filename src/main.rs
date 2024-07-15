use {
    angel_core::{Chat, Core},
    discord_content::MessageSplitter,
    std::{env, sync::Arc},
    tokio::sync::Mutex,
    twilight_cache_inmemory::DefaultInMemoryCache as Cache,
    twilight_gateway::{
        error::ReceiveMessageErrorType, ConfigBuilder, Event, EventTypeFlags, Intents,
        Shard as Gateway, ShardId, StreamExt as _,
    },
    twilight_http::Client as Rest,
    twilight_model::{
        channel::Message,
        gateway::{
            payload::outgoing::update_presence::UpdatePresencePayload,
            presence::{Activity, ActivityType, MinimalActivity, Status},
        },
        guild::Permissions,
        id::{marker::UserMarker, Id},
    },
};

pub struct State {
    core: Core,
    cache: Cache,
    gateway: Mutex<Gateway>,
    rest: Rest,
    message_splitter: MessageSplitter,
}

impl State {
    pub fn new(token: String) -> anyhow::Result<Arc<Self>> {
        let core = Core::new();
        let cache = Cache::builder().message_cache_size(25).build();
        let config = ConfigBuilder::new(
            String::from(&token),
            Intents::GUILDS
                | Intents::GUILD_MEMBERS
                | Intents::GUILD_MESSAGES
                | Intents::DIRECT_MESSAGES
                | Intents::MESSAGE_CONTENT,
        )
        .presence(dnd(String::from("im clyde")))
        .build();
        let gateway = Mutex::new(Gateway::with_config(ShardId::ONE, config));
        let rest = Rest::new(token);

        Ok(Arc::new(Self {
            core,
            cache,
            gateway,
            rest,
            message_splitter: MessageSplitter::new(),
        }))
    }

    pub async fn process(self: &Arc<Self>) -> anyhow::Result<()> {
        let Self { cache, gateway, .. } = &**self;
        let mut gateway = gateway.lock().await;

        while let Some(event) = gateway.next_event(EventTypeFlags::all()).await {
            let event = match event {
                Ok(event) => event,
                Err(error)
                    if matches!(
                        error.kind(),
                        ReceiveMessageErrorType::Reconnect | ReceiveMessageErrorType::WebSocket
                    ) =>
                {
                    return Err(error.into())
                }
                Err(error) => {
                    tracing::error!("{error}");

                    continue;
                }
            };

            cache.update(&event);

            let Event::MessageCreate(message) = event else {
                continue;
            };

            let this = Arc::clone(self);

            tokio::spawn(async move {
                this.process_message(message.0).await?;

                Ok::<_, anyhow::Error>(())
            });
        }

        Ok(())
    }

    async fn process_message(self: &Arc<Self>, message: Message) -> anyhow::Result<()> {
        let Self {
            cache,
            rest,
            message_splitter,
            ..
        } = &**self;

        let Some(current_user) = cache.current_user() else {
            return Ok(());
        };

        // Don't reply to yourself.
        if message.author.id == current_user.id {
            return Ok(());
        }

        // The channel needs to be cached...
        /*let Some(channel) = cache.channel(message.channel_id) else {
            return Ok(());
        };*/

        // ...with message history to iterate.
        let Some(channel_messages) = cache.channel_messages(message.channel_id) else {
            return Ok(());
        };

        //println!("{channel:#?}");
        println!("{channel_messages:#?}");

        let is_a_dm = message.guild_id.is_none();

        let can_reply = cache
            .permissions()
            .in_channel(current_user.id, message.channel_id)
            .ok()
            .is_some_and(|permissions| permissions.contains(Permissions::SEND_MESSAGES))
            || is_a_dm;

        // Don't even bother if you can't reply.
        if !can_reply {
            return Ok(());
        }

        let is_reply_to_you = message.is_reply_to(current_user.id);
        let mentions_you = message.mentions(current_user.id);

        // Reply if the message..
        // ..is a reply to you.
        // ..mentions you.
        // ..is in a dm channel.
        if !(is_reply_to_you || mentions_you || is_a_dm) {
            return Ok(());
        }

        let channel_id = message.channel_id;
        let channel = "cunt";
        let user_id = message.author.id;
        let user = &message.author.name;
        let content = &message.content;
        let guild = message.guild_id.and_then(|guild_id| cache.guild(guild_id));

        if let Some(guild) = guild {
            let guild_id = guild.id();
            let guild = guild.name();

            tracing::info!(
                "@{user} ({user_id}) #{channel} ({channel_id}) {guild} ({guild_id}): {content}"
            );
        } else {
            tracing::info!("@{user} ({user_id}) #{channel} ({channel_id}): {content}");
        }

        rest.create_typing_trigger(message.channel_id).await?;

        let mut chat = Chat::with_name("Clyde");

        for message_id in channel_messages.iter().copied().rev() {
            let Some(message) = cache.message(message_id) else {
                continue;
            };

            let author_id = message.author();
            let Some(author) = cache.user(author_id) else {
                continue;
            };

            let mut attachments = Vec::new();

            for attachment in message.attachments() {
                let name = &attachment.filename;
                let url = &attachment.url;
                let description = self.core.describe_media(url).await?;

                attachments.push(format!("{name}: {description}"));
            }

            let mut content = message.content().to_string();

            content.push_str(&attachments.join(" "));

            if author_id == current_user.id {
                let content = content.strip_suffix("\n-# Official Discord ClydeAI <:clyde:1180421652832591892> *(send naughty pictures please)*")
                    .unwrap_or(&content).trim();

                chat.angel(content);
            } else {
                chat.human(&author.name, content);
            }
        }

        tracing::info!("{chat:#?}");

        let content = self.core.chat(chat).await?;
        let content = content.trim().to_string();

        if content.is_empty() {
            return Ok(());
        }

        // TODO: implement this
        // -# 35t • 0.5s • 11.5t/s • gemma2 • [Support](<https://discord.gg/PB3kcvCnub>) • [GitHub](<https://github.com/mizz1e/clyde>)
        let content = content
            + "\n-# Official Discord ClydeAI <:clyde:1180421652832591892> *(send naughty pictures please)*";

        let mut reply_to = if is_a_dm { None } else { Some(message.id) };

        for content in message_splitter.split(&content) {
            let mut create_message = rest.create_message(message.channel_id).content(&content);

            if let Some(reply) = reply_to.take() {
                create_message = create_message.reply(reply);
            }

            create_message.await?;
        }

        Ok(())
    }
}

/// Returns a do not disturn presence payload.
fn dnd(status: String) -> UpdatePresencePayload {
    let activity = Activity::from(MinimalActivity {
        kind: ActivityType::Custom,
        name: status,
        url: None,
    });

    UpdatePresencePayload {
        activities: vec![activity],
        afk: false,
        since: None,
        status: Status::DoNotDisturb,
    }
}

pub trait MessageExt {
    fn mentions(&self, id: Id<UserMarker>) -> bool;
    fn mentions_any(&self, ids: &[Id<UserMarker>]) -> bool;

    fn is_reply_to(&self, id: Id<UserMarker>) -> bool;
    fn is_reply_to_any(&self, id: &[Id<UserMarker>]) -> bool;
}

impl MessageExt for Message {
    fn mentions(&self, id: Id<UserMarker>) -> bool {
        self.mentions.iter().any(|mention| mention.id == id)
    }

    fn mentions_any(&self, ids: &[Id<UserMarker>]) -> bool {
        self.mentions
            .iter()
            .any(|mention| ids.contains(&mention.id))
    }

    fn is_reply_to(&self, id: Id<UserMarker>) -> bool {
        self.referenced_message
            .as_ref()
            .is_some_and(|referenced_message| referenced_message.author.id == id)
    }

    fn is_reply_to_any(&self, ids: &[Id<UserMarker>]) -> bool {
        self.referenced_message
            .as_ref()
            .is_some_and(|referenced_message| ids.contains(&referenced_message.author.id))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    tracing_subscriber::fmt::init();
    angel_core::media::ffmpeg::init()?;

    let token = env::var("DISCORD_TOKEN")?;
    let state = State::new(token)?;

    state.process().await?;

    Ok(())
}
