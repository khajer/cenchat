use tokio_tungstenite::tungstenite::Message;

pub enum Command {
    Name(String),
    Join(String),
    Leave,
    Who,
    Text(String),
    Unknown(String),
}

pub fn parse_line(line: &str) -> Command {
    if let Some(rest) = line.strip_prefix("/name ") {
        return Command::Name(rest.trim().to_string());
    }
    if line.trim() == "/name" {
        return Command::Name(String::new());
    }

    if let Some(rest) = line.strip_prefix("/join ") {
        return Command::Join(rest.trim().to_string());
    }
    if line.trim() == "/join" {
        return Command::Join(String::new());
    }

    if line.trim() == "/leave" {
        return Command::Leave;
    }

    if line.trim() == "/who" {
        return Command::Who;
    }

    if let Some(cmd) = line.trim().strip_prefix('/') {
        return Command::Unknown(cmd.split_whitespace().next().unwrap_or("").to_string());
    }

    Command::Text(line.to_string())
}

pub fn sys(text: impl Into<String>) -> Message {
    Message::Text(format!("SYS {}", text.into()).into())
}

pub fn err(text: impl Into<String>) -> Message {
    Message::Text(format!("ERR {}", text.into()).into())
}

pub fn chat(name: &str, text: &str) -> Message {
    Message::Text(format!("MSG {name}: {text}").into())
}

pub fn users_list(names: &[String]) -> Message {
    Message::Text(format!("USERS {}", names.join(" ")).into())
}
