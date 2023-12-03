use twilight_model::channel::message::MessageType;

pub trait MessageTypeExt {
    fn is_channel_message_pinned(&self) -> bool;
    fn is_guild_boost(&self) -> bool;
    fn is_regular(&self) -> bool;
    fn is_reply(&self) -> bool;
    fn is_user_join(&self) -> bool;
}

impl MessageTypeExt for MessageType {
    fn is_channel_message_pinned(&self) -> bool {
        matches!(self, MessageType::ChannelMessagePinned)
    }

    fn is_guild_boost(&self) -> bool {
        matches!(
            self,
            MessageType::GuildBoost
                | MessageType::GuildBoostTier1
                | MessageType::GuildBoostTier2
                | MessageType::GuildBoostTier3
        )
    }

    fn is_regular(&self) -> bool {
        matches!(self, MessageType::Regular)
    }

    fn is_reply(&self) -> bool {
        matches!(self, MessageType::Reply)
    }

    fn is_user_join(&self) -> bool {
        matches!(self, MessageType::UserJoin)
    }
}
