use {
    base64::{engine::general_purpose::STANDARD, Engine},
    image::{codecs::jpeg::JpegEncoder, imageops},
    serde::{Deserialize, Serialize},
    std::{
        borrow::Cow,
        collections::hash_map::{self, HashMap},
        env,
        fmt::Write as _,
        io,
        path::PathBuf,
        time::Instant,
    },
    tracing::{debug, info},
    twilight_cache_inmemory::InMemoryCache,
    twilight_gateway::{Event, Intents, ShardId},
    twilight_model::{
        channel::{message::MessageType, Message},
        id::{
            marker::{AttachmentMarker, ChannelMarker, UserMarker},
            Id,
        },
    },
    twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder},
};

pub const CLYDE_ID: Id<UserMarker> = Id::new(1116684158199144468);

pub struct Clyde {
    cache: InMemoryCache,
    gateway: twilight_gateway::Shard,
    rest: twilight_http::Client,
    url_cache: HashMap<Id<AttachmentMarker>, String>,
}

#[derive(Serialize)]
pub struct LlamaImageData {
    pub data: String,
    pub id: u32,
}

#[derive(Serialize)]
pub struct LlamaRequest {
    pub image_data: Vec<LlamaImageData>,
    pub max_new_tokens: u32,
    pub n_predict: u32,
    pub prompt: String,
    pub repeat_penalty: f32,
    pub stop: Vec<String>,
    pub temperature: f32,
    pub top_k: f32,
    pub top_p: f32,
    pub truncate: u32,
}

#[derive(Deserialize)]
pub struct LlamaResponse {
    pub content: String,
    pub model: PathBuf,
}

impl Clyde {
    pub fn new(token: String) -> Self {
        Self {
            cache: InMemoryCache::builder().message_cache_size(50).build(),
            gateway: twilight_gateway::Shard::new(ShardId::ONE, token.clone(), Intents::all()),
            rest: twilight_http::Client::new(token),
            url_cache: HashMap::new(),
        }
    }

    pub async fn start_typing(&self, channel_id: Id<ChannelMarker>) -> anyhow::Result<()> {
        self.rest.create_typing_trigger(channel_id).await?;

        Ok(())
    }

    pub async fn process_message(&mut self, message: &Message) -> anyhow::Result<()> {
        if message.author.id == CLYDE_ID {
            debug!("Ignored self message");

            return Ok(());
        }

        for attachment in &message.attachments {
            if let Err(error) = self.process_url(attachment.id, &attachment.url).await {
                println!("{error:?}");
            }
        }

        let channel_id = message.channel_id;

        let Some(channel) = self.cache.channel(channel_id) else {
            debug!("Ignored unknown channel");

            return Ok(());
        };

        let Some(channel_name) = channel.name.as_deref() else {
            debug!("Ignored channel without a name");

            return Ok(());
        };

        let Some(channel_messages) = self.cache.channel_messages(channel_id) else {
            debug!("Ignored channel without any messages");

            return Ok(());
        };

        let mut channel_information =
            format!("You are currently in the channel #{channel_name} (<#{channel_id}>).\n");

        if let Some(channel_topic) = channel.topic.as_deref() {
            write!(channel_information, "Channel Topic: {channel_topic}\n")?;
        }

        let mut users = HashMap::new();
        let mut message_list = Vec::new();

        for message_id in channel_messages.iter().copied().rev() {
            let Some(message) = self.cache.message(message_id) else {
                continue;
            };

            let author_id = message.author();

            let Some(author) = self.cache.user(author_id) else {
                continue;
            };

            let author_name = author.name.as_str();

            users
                .entry(author_id)
                .or_insert_with(|| format!("{author_name} (<@{author_id}>)"));

            match message.kind() {
                MessageType::Regular => {
                    let content = message.content();
                    let mut message_information = format!("user\n{author_name}: {content}\n");

                    for embed in message.embeds() {
                        if let Some(embed_title) = embed.title.as_deref() {
                            write!(
                                message_information,
                                "{author_name} (embed): {embed_title}\n"
                            )?;
                        }

                        if let Some(embed_description) = embed.description.as_deref() {
                            write!(
                                message_information,
                                "{author_name} (embed): {embed_description}\n"
                            )?;
                        }

                        if let Some(embed_footer) = embed.footer.as_ref() {
                            let embed_footer_text = embed_footer.text.as_str();

                            write!(
                                message_information,
                                "{author_name} (embed): {embed_footer_text}\n"
                            )?;
                        }
                    }

                    message_list.push(message_information.trim().into());
                }
                MessageType::UserJoin => {
                    message_list.push(format!("system\nUser {author_name} has joined the server."));
                }
                MessageType::GuildBoost
                | MessageType::GuildBoostTier1
                | MessageType::GuildBoostTier2
                | MessageType::GuildBoostTier3 => {
                    message_list.push(format!(
                        "system\nUser {author_name} has boosted the server."
                    ));
                }
                _ => continue,
            }
        }

        if message_list.is_empty() {
            return Ok(());
        }

        let system_prompt = include_str!("system_prompt.txt").trim();

        let channel_information = channel_information.trim();

        let mut user_information = users.into_values().collect::<Vec<_>>();

        user_information.sort_unstable();

        let user_information = user_information.join("\n");
        let user_information = user_information.trim();

        let message_list = message_list.join("<|im_end|>\n<|im_start|>");
        let mut message_list = String::from(message_list.trim());

        message_list.insert_str(0, "<|im_start|>");
        message_list.push_str("<|im_end|>");

        let prompt = format!("<|im_start|>system\n{system_prompt}\n{channel_information}\n{user_information}<|im_end|>\n{message_list}\n<|im_start|>assistant\n");

        self.start_typing(channel_id).await?;

        info!("prompt = {prompt}");

        let start = Instant::now();
        let response = reqwest::Client::new()
            .post("http://127.0.0.1:8080/completion")
            .json(&LlamaRequest {
                //image_data,
                image_data: vec![],
                max_new_tokens: 2048,
                n_predict: 1000,
                prompt,
                repeat_penalty: 1.2,
                stop: vec![String::from("<|im_end|>")],
                temperature: 0.2,
                top_k: 50.0,
                top_p: 0.95,
                truncate: 1950,
            })
            .send()
            .await?
            .json::<LlamaResponse>()
            .await?;

        let elapsed = Instant::now().duration_since(start);
        let content = if response.content.is_empty() {
            "Didn't generate a response <:clyde:1180421652832591892>"
        } else {
            response.content.as_str()
        };

        let model = response
            .model
            .file_stem()
            .map(|name| name.to_string_lossy())
            .unwrap_or(Cow::Borrowed("mysterious model"));

        let embed = EmbedBuilder::new()
            .color(0x5865f2)
            .footer(EmbedFooterBuilder::new(format!(
                "{model} | {elapsed:.2?} | 0.2",
            )))
            .build();

        self.rest
            .create_message(message.channel_id) //, message.id)
            .content(&content)?
            .embeds(&[embed])?
            .await?;

        Ok(())
    }

    pub async fn process_url(
        &mut self,
        attachment_id: Id<AttachmentMarker>,
        url: &str,
    ) -> anyhow::Result<()> {
        let hash_map::Entry::Vacant(entry) = self.url_cache.entry(attachment_id) else {
            debug!("Skipped `{url}`, already processed.");

            return Ok(());
        };

        debug!("Download `{url}`");

        let bytes = reqwest::get(url).await?.bytes().await?;
        let base64 = process_image(&bytes)?;

        entry.insert(base64);

        Ok(())
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let event = match self.gateway.next_event().await {
                Ok(event) => event,
                Err(error) if error.is_fatal() => {
                    return Err(error.into());
                }
                _ => return Ok(()),
            };

            self.cache.update(&event);

            if let Event::MessageCreate(message) = event {
                self.process_message(&message).await?;
            }
        }
    }
}

/// Process an image.
///
/// - Load image data from `bytes`.
/// - Resize to 512x512, maintaining aspect ratio.
/// - Quantize colour data.
/// - Encode as a JPEG with 65% quality.
/// - Encode as base64.
fn process_image(bytes: &[u8]) -> anyhow::Result<String> {
    debug!("Attempt to parse {} bytes as an image", bytes.len());

    let image = image::load_from_memory(&bytes)?;

    /*debug!("Resize to 512x512");

    let mut image = image.resize(512, 512, imageops::Triangle);*/

    debug!("Ensure 8-bit RGBA");

    let mut image = image.into_rgba8();

    debug!("Build NEUQUANT color map");

    let color_map = color_quant::NeuQuant::new(30, 128, image.as_raw());

    debug!("Apply dithering");

    imageops::dither(&mut image, &color_map);

    debug!("Encode as JPEG with 65% quality.");

    let mut jpeg = io::Cursor::new(Vec::new());

    image.write_with_encoder(JpegEncoder::new_with_quality(&mut jpeg, 60))?;

    debug!("Encode as base64");

    let mut base64 = String::new();

    STANDARD.encode_string(&jpeg.into_inner(), &mut base64);

    Ok(base64)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let token = env::var("CLYDE_TOKEN")?;

    Clyde::new(token).run().await?;

    Ok(())
}
