/*
# jeff

jeff is a command-line ai unlike any other. He's like your unstable uncle except he lives inside your computer and is a bash expert.

``` bash
jeff setup
jeff "hi jeff"
jeff "what directory am I in?"
jeff "compile this pls" ~/Downloads/hi.cpp
```

jeff tries to run a command
jeff corrects himself if he's wrong
jeff outputs the command output
jeff provides a summary of what happened
jeff makes user happy

```
To install (this will happen with an install script in the future):
cargo build --release
mv target/release/jeff /usr/local/bin
```

Disclaimer: jeff is a work in progress. He's not perfect, but he's learning.
Also, jeff isn't able to navigate your computer's file system yet. He's working on it.

*/
// https://platform.openai.com/account/billing/overview

// before releasing:    - [ ] remove hardcoded api key
//                      - [x] add install script
//                      - [ ] add uninstall script

use anyhow::Result;
use chat_gpt_lib_rs::{ChatGPTClient, ChatInput, Message, Model, Role};
use regex::Regex;
// use std::env;
use std::error::Error;
// use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
// use std::process::{Command, Stdio};
// use std::sync::Mutex;

static mut CURRENT_DIR: Option<PathBuf> = None;
// static mut API_KEY: Option<String> = None;
static BASE_URL: &str = "https://api.openai.com";
// static TRUNCATION: usize = 2028; // max characters

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let os = std::env::consts::OS;

    let api_key = "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"; // hardcode for testing
                                                                         /*
                                                                         let api_key = env::var("API_KEY").unwrap_or_else(|_| {
                                                                             println!("API_KEY not found. Please enter your API key:");
                                                                             let mut api_key = String::new();
                                                                             io::stdin()
                                                                                 .read_line(&mut api_key)
                                                                                 .expect("Failed to read line");
                                                                             api_key.trim().to_string()
                                                                         });
                                                                         */
    {
        // save key in downloads folder; this doesn't work yet
        let mut path = if os == "windows" {
            dirs::data_dir().unwrap_or_else(|| PathBuf::from(r"C:\Users\YourUsername\Downloads"))
        } else {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("~/Downloads"))
        };
        path.push("openai_api_key.txt");
        let mut file = std::fs::File::create(path)?;
        file.write_all(api_key.as_bytes())?;
    }

    let client = ChatGPTClient::new(&api_key, BASE_URL);
    let mut messages = vec![Message {
        role: Role::System,
        content: "You are jeff, part of a command-line app that helps the user. You can run bash commands. To run a command, use brackets: eg. [ls] or [cd ~/Downloads] and program will then run any commands for you. Use long commands if necessary, eg. [cat file.txt | grep 'bash' | wc -l]. Use ls and similar commands to get info about the system. The user's operating system is: ".to_string() + &os, }];

    loop {
        print!("_> ");
        stdout().flush().unwrap();
        let mut user_input = String::new();
        stdin().read_line(&mut user_input).unwrap();

        messages.push(Message {
            role: Role::User,
            content: user_input.trim().to_string(),
        });

        let input = ChatInput {
            // Define the input for the ChatGPTClient
            model: Model::Gpt_4Turbo,
            messages: messages.clone(),
            ..Default::default()
        };
        let response = client.chat(input).await?;
        let ai_message = &response.choices[0].message.content;
        println!("{}", ai_message);
        messages.push(Message {
            role: Role::Assistant,
            content: ai_message.clone(),
        });

        // HANDLE COMMANDS
        let extractor = Regex::new(r"\[(.*?)\]").unwrap();
        for cap in extractor.captures_iter(&ai_message) {
            let command = &cap[1];
            let output = run_bash_command(command);
            user_input = user_input.replace(&format!("[{}]", command), &output);
            messages.push(Message {
                role: Role::System,
                content: format!("The command `{}` output: {}", command, &output),
            });

            // Revision loop
            let mut satisfactory_result = false;
            while satisfactory_result == false {
                // ask chatgpt if the command was satisfactory
                messages.push(Message {
                        role: Role::System,
                        content: "Did the command you ran satisfy the user's request? Are there more steps to this process? If the command output looks good, DO NOT SAY ANYTHING. You are responsible for satisfying the user's request; try again if you fail.".to_string(),
                });
                let input2 = ChatInput {
                    model: Model::Gpt_4Turbo,
                    messages: messages.clone(),
                    ..Default::default()
                };
                let response = client.chat(input2).await?;
                let ai_message2 = &response.choices[0].message.content;
                // if the result is empty or includes "goodbye", then the result is satisfactory
                if ai_message2 == ""
                    || ai_message2.to_lowercase().contains("goodbye")
                    || ai_message2.to_lowercase().contains("")
                {
                    satisfactory_result = true;
                } else {
                    println!("{}", ai_message2);
                    messages.push(Message {
                        role: Role::Assistant,
                        content: ai_message2.clone(),
                    });
                }
            }
        }
    }
}

fn run_bash_command(command: &str) -> String {
    let mut command_parts = command.split_whitespace();
    let first_part = command_parts.next().unwrap_or("");

    if first_part == "cd" {
        // If it's a "cd" command, update the current directory
        let target_dir = command_parts.next().unwrap_or("/");
        let new_dir = PathBuf::from(target_dir);
        if new_dir.is_dir() {
            unsafe {
                CURRENT_DIR = Some(new_dir.clone());
            }
        } else {
            eprintln!("Error: Directory '{}' not found", target_dir);
            return String::new();
        }
    }

    let output = std::process::Command::new("bash") // fix this; the directory never updates
        .current_dir("/")
        .arg("-c")
        .arg(command)
        .output()
        .unwrap_or_else(|_| panic!("could not run command `{}`", command));

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if output.status.success() {
        println!("`{}` -> ", command);
        println!("{}", stdout);
        stdout
    } else {
        eprintln!("{}\n{}", stderr, command);
        stderr
    }
}
// "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" // TODO: remove when open sourcing
