use chrono::{Timelike, Utc};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::{
    io::{stdin, stdout, BufRead, BufReader, Write},
    net::TcpStream,
    thread,
};

fn main() {
    let mut address = String::new();

    // Read server address
    println!("Enter IRC server IP address and port separated by a semicolon:");
    stdin().read_line(&mut address).unwrap();
    // Remove the trailing newline
    address = address.trim().to_owned();

    println!("Connecting to {} ...", &address);

    // Initialize a stream variable and try to connect to the server
    let mut stream = TcpStream::connect(&address).unwrap();

    println!("Connected!");

    let cloned_stream = stream.try_clone().unwrap();
    thread::spawn(move || handle_messages(cloned_stream));

    let mut nickname = String::new();
    println!("Enter your nickname:");
    stdin().read_line(&mut nickname).unwrap();
    nickname = nickname.trim().to_owned();

    login(stream.try_clone().unwrap(), nickname);

    let mut input = String::new();
    let mut current_channel = String::new();

    loop {
        stdin().read_line(&mut input).unwrap();
        input = input.trim().to_owned();
        let splitted_input = input.split(' ').collect::<Vec<&str>>();
        match splitted_input[0] {
            "!ch" => {
                if splitted_input.len() < 2 {
                    exo_error("Please provide a channel name!");
                } else {
                    //if current_channel != "" {
                    if !current_channel.is_empty() {
                        stream
                            .write_all(format!("PART {}\r\n", current_channel).as_bytes())
                            .unwrap();
                        stream.flush().unwrap();
                    }
                    current_channel = splitted_input[1].to_owned();
                    stream
                        .write_all(format!("JOIN {}\r\n", current_channel).as_bytes())
                        .unwrap();
                    stream.flush().unwrap();
                }
            }
            "!curch" => {
                exo_sysmsg(&format!("The current channel is {}", current_channel));
            }
            "!q" => {
                let reason = if splitted_input.len() < 2 {
                    "Leaving."
                } else {
                    splitted_input[1]
                };
                stream
                    .write_all(format!("QUIT {}\r\n", reason).as_bytes())
                    .unwrap();
                stream.flush().unwrap();
                return;
            }
            "!ls" => {
                stream.write_all("LIST\r\n".as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            "!raw" => {
                let mut raw_message = String::new();
                println!("Type in the raw IRC message:");
                stdin().read_line(&mut raw_message).unwrap();
                raw_message = format!("{}\r\n", raw_message.trim());
                stream.write_all(raw_message.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            _ => {
                if String::is_empty(&current_channel) {
                    exo_error("Select a channel! (!ch {{channel name}})");
                } else {
                    stream
                        .write_all(format!("PRIVMSG {} :{}\r\n", current_channel, input).as_bytes())
                        .unwrap();
                    stream.flush().unwrap();
                }
            }
        }
        input = "".to_owned();
    }
}

fn handle_messages(stream_var: TcpStream) {
    let mut data = String::new();
    let mut reader = BufReader::new(&stream_var);

    loop {
        reader.read_line(&mut data).unwrap();
        let splitted_data = data.split(' ').collect::<Vec<&str>>();
        match splitted_data[0] {
            "PING" => {
                exo_debug("Got pinged!");
                let mut tmp_stream = stream_var.try_clone().unwrap();
                tmp_stream
                    .write_all(format!("PONG {}", splitted_data[1]).as_bytes())
                    .unwrap();
                tmp_stream.flush().unwrap();
            }
            _ => {
                exo_info(data.trim_end());
            }
        }
        data = "".to_owned();
    }
}

fn login(mut stream: TcpStream, nickname: String) {
    stream
        .write_all(format!("NICK {}\r\n", nickname).as_bytes())
        .unwrap();
    stream.flush().unwrap();

    stream
        .write_all(format!("USER {} 0 * :{}\r\n", nickname, nickname).as_bytes())
        .unwrap();
    stream.flush().unwrap();
}

fn exo_error(message: &str) {
    execute!(
        stdout(),
        SetForegroundColor(Color::Red),
        Print("ERROR"),
        ResetColor,
        Print(format!(": {}\n", message))
    )
    .unwrap()
}

fn exo_info(message: &str) {
    let now = Utc::now();
    execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        Print(format!(
            "{:02}:{:02}:{:02}",
            now.hour(),
            now.minute(),
            now.second()
        )),
        ResetColor,
        Print(format!(" {}\n", message))
    )
    .unwrap()
}

#[cfg(debug_assertions)]
fn exo_debug(message: &str) {
    let now = Utc::now();
    execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        Print(format!(
            "{:02}:{:02}:{:02}",
            now.hour(),
            now.minute(),
            now.second()
        )),
        SetForegroundColor(Color::Yellow),
        Print(" DEBUG"),
        ResetColor,
        Print(format!(" {}\n", message))
    )
    .unwrap()
}

#[cfg(not(debug_assertions))]
fn exo_debug(_message: &str) {}

fn exo_sysmsg(message: &str) {
    let now = Utc::now();
    execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        Print(format!(
            "{:02}:{:02}:{:02}",
            now.hour(),
            now.minute(),
            now.second()
        )),
        SetForegroundColor(Color::Magenta),
        Print(" INFO"),
        ResetColor,
        Print(format!(" {}\n", message))
    )
    .unwrap()
}
