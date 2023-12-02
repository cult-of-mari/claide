use {
    base64::{engine::general_purpose::STANDARD, Engine},
    image::{codecs::jpeg::JpegEncoder, imageops},
    serde::{Deserialize, Serialize},
    std::{
        collections::hash_map::{self, HashMap},
        env,
        fmt::Write as _,
        io,
        path::PathBuf,
        time::{Duration, Instant},
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
    pub n_keep: i32,
    pub truncate: u32,
}

#[derive(Deserialize)]
pub struct LlamaResponse {
    pub content: String,
    pub model: PathBuf,
    pub stopped_eos: bool,
    pub stopped_limit: bool,
    pub stopping_word: String,
}

pub struct LlamaResult {
    pub content: String,
    pub model: String,
    pub duration: Duration,
    pub stop_reason: String,
}

#[derive(Deserialize, Serialize)]
pub struct LlamaTokenize {
    content: String,
}

#[derive(Deserialize, Serialize)]
pub struct LlamaDetokenize {
    tokens: Vec<i32>,
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

        if !matches!(
            message.kind,
            MessageType::Regular
                | MessageType::Reply
                | MessageType::UserJoin
                | MessageType::GuildBoost
                | MessageType::GuildBoostTier1
                | MessageType::GuildBoostTier2
                | MessageType::GuildBoostTier3
                | MessageType::ChannelMessagePinned
        ) {
            return Ok(());
        }

        if !(message
            .mentions
            .iter()
            .any(|mention| mention.id == CLYDE_ID)
            || message
                .referenced_message
                .as_ref()
                .is_some_and(|message| message.author.id == CLYDE_ID))
        {
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

            users.entry(author_id).or_insert_with(|| {
                (
                    author_name.to_string(),
                    format!("{author_name} (<@{author_id}>)"),
                )
            });

            match message.kind() {
                MessageType::Regular | MessageType::Reply => {
                    let content = message.content();

                    if author_id == CLYDE_ID {
                        message_list.push(format!("assistant\n{content}"));
                    } else {
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
                }
                MessageType::UserJoin => {
                    message_list.push(format!("system\nUser {author_name} has joined the server."));
                }
                MessageType::ChannelMessagePinned => {
                    message_list.push(format!("system\nUser {author_name} pinned a message."));
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

        let mut user_information = users
            .values()
            .map(|(_, u)| u.to_string())
            .collect::<Vec<_>>();

        user_information.sort_unstable();

        let user_information = user_information.join(",");
        let user_information = user_information.trim();

        let message_list = message_list.join("<|im_end|>\n<|im_start|>");
        let mut message_list = String::from(message_list.trim());

        message_list.insert_str(0, "<|im_start|>");
        message_list.push_str("<|im_end|>");

        let mut prompt = format!("<|im_start|>system\n{system_prompt}\n{channel_information}\nUsers in this channel:{user_information}<|im_end|>\nConversation:\n");
        info!("system_prompt = {prompt}");

        let _keep_tokens = llama_tokenize(&prompt).await?.len();
        let dynamic_prompt = format!("{message_list}\n<|im_start|>assistant\nClyde:");
        info!("dyanmic_prompt = {dynamic_prompt}");

        prompt.push_str(&dynamic_prompt);
        info!("full_prompt = {prompt}");

        self.start_typing(channel_id).await?;

        let in_tokens = llama_tokenize(&prompt).await?.len();

        let mut stop = vec![String::from("<|im_end|>")];

        stop.extend(users.values().map(|(u, _)| u.to_string()));

        let LlamaResult {
            content,
            model,
            duration,
            stop_reason,
        } = llama_completion(LlamaRequest {
            //image_data,
            image_data: vec![],
            max_new_tokens: 2048,
            n_predict: 2048,
            prompt,
            repeat_penalty: 1.2,
            stop,
            temperature: 0.2,
            top_k: 50.0,
            top_p: 0.95,
            n_keep: 0, //dbg!(keep_tokens).try_into().unwrap(),
            truncate: 1950,
        })
        .await?;

        let out_tokens = llama_tokenize(&content).await?.len();

        let content = if content.is_empty() {
            "<:clyde:1180421652832591892> *Clyde did not generate a response.*"
        } else {
            content.as_str()
        };

        let content = content.chars().collect::<Vec<_>>();
        let mut iter = content
            .chunks(1950)
            .map(|chunk| chunk.into_iter().collect::<String>())
            .map(|mut content| {
                if content.matches("```").count() % 2 == 1 {
                    content.push_str("```");
                }

                content
            })
            .peekable();

        while let Some(content) = iter.next() {
            let create_message = self
                .rest
                .create_message(message.channel_id)
                .content(&content)?;

            if iter.peek().is_none() {
                let embed = EmbedBuilder::new()
                    .color(0x5865f2)
                    .footer(EmbedFooterBuilder::new(format!(
                        "model {model} | duration {duration:.2?} | temperature 0.2 | top_k 50 | top_p 0.95 | repeat_penalty 1.2 | in_tokens {in_tokens} | out_tokens {out_tokens} | stop {stop_reason}",
                    )))
                    .build();

                create_message.embeds(&[embed])?.await?;
            } else {
                create_message.await?;
            }
        }

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

async fn llama_tokenize(string: &str) -> anyhow::Result<Vec<i32>> {
    let tokens = reqwest::Client::new()
        .post("http://127.0.0.1:8080/tokenize")
        .json(&LlamaTokenize {
            content: string.into(),
        })
        .send()
        .await?
        .json::<LlamaDetokenize>()
        .await?
        .tokens;

    Ok(tokens)
}

async fn llama_detokenize(tokens: &[i32]) -> anyhow::Result<String> {
    let content = reqwest::Client::new()
        .post("http://127.0.0.1:8080/detokenize")
        .json(&LlamaDetokenize {
            tokens: tokens.into(),
        })
        .send()
        .await?
        .json::<LlamaTokenize>()
        .await?
        .content;

    Ok(content)
}

async fn llama_completion(request: LlamaRequest) -> anyhow::Result<LlamaResult> {
    let start = Instant::now();
    let response = reqwest::Client::new()
        .post("http://127.0.0.1:8080/completion")
        .json(&request)
        .send()
        .await?
        .json::<LlamaResponse>()
        .await?;

    let duration = Instant::now().duration_since(start);

    Ok(LlamaResult {
        content: response.content.trim().into(),
        model: response.model.file_name().unwrap().to_string_lossy().into(),
        duration,
        stop_reason: if response.stopped_eos {
            "eos".into()
        } else if response.stopped_limit {
            "limit".into()
        } else {
            format!("token {}", response.stopping_word)
        },
    })
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

    llama_completion(LlamaRequest {
        //image_data,
        image_data: vec![],
        max_new_tokens: 2048,
        n_predict: 2048,
        prompt: String::from("<|im_start|>system\nhi<|im_end|>\n<|im_start|>assistant\n"),
        repeat_penalty: 1.2,
        stop: vec![String::from("<|im_end|>")],
        temperature: 0.2,
        top_k: 50.0,
        top_p: 0.95,
        n_keep: 0,
        truncate: 1950,
    })
    .await?;

    Clyde::new(token).run().await?;

    Ok(())
}
