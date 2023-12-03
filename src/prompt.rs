use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Channel<'a> {
    pub name: &'a str,
    pub topic: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Author<'a> {
    Assistant,
    System,
    User(&'a str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Message<'a> {
    pub author: Author<'a>,
    pub content: &'a str,
}

impl<'a> fmt::Display for Channel<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { name, topic } = self;

        write!(fmt, "{name}")?;

        if let Some(topic) = topic {
            write!(fmt, ": {topic}\n")?;
        }

        Ok(())
    }
}

impl<'a> fmt::Display for Author<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Author::Assistant => write!(fmt, "assistant\nClyde:"),
            Author::System => write!(fmt, "system\n"),
            Author::User(username) => {
                let username = username.trim();

                write!(fmt, "user\n{username}:")
            }
        }
    }
}

impl<'a> fmt::Display for Message<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { author, content } = self;
        let content = content.trim();

        write!(fmt, "<|im_start|>{author}{content}<|im_end|>\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel() {
        let channel = Channel {
            name: "general",
            topic: Some("chat here"),
        };

        assert_eq!(channel.to_string(), "general: chat here\n");
    }

    #[test]
    fn assistant() {
        let message = Message {
            author: Author::Assistant,
            content: "hi",
        };

        assert_eq!(
            message.to_string(),
            "<|im_start|>assistant\nClyde:hi<|im_end|>\n"
        );
    }

    #[test]
    fn system() {
        let message = Message {
            author: Author::System,
            content: "User Clyde has joined the guild.",
        };

        assert_eq!(
            message.to_string(),
            "<|im_start|>system\nUser Clyde has joined the guild.<|im_end|>\n"
        );
    }

    #[test]
    fn user() {
        let message = Message {
            author: Author::User("mari"),
            content: "hi",
        };

        assert_eq!(message.to_string(), "<|im_start|>user\nmari:hi<|im_end|>\n");
    }
}
