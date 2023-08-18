use clap::Parser;
use rustyline;

#[derive(Debug, Parser)]
#[command(version)]
struct Options {
    #[arg(short, long, default_value = "gpt-3.5-turbo")]
    model: String,

    #[arg(short, long, default_value = None)]
    temperature: Option<f64>,

    #[arg(short, long)]
    no_stream: bool,

    #[arg(short, long, default_value = None)]
    user: Option<String>,

    args: Option<Vec<String>>,
}

fn request(
    api_key: &str,
    model: String,
    temperature: Option<f64>,
    stream: Option<bool>,
    user: Option<String>,
    messages: Vec<simple_chatgpt::Message>,
) -> String {
    let request_body = simple_chatgpt::RequestBody {
        model,
        messages,
        temperature,
        stream,
        user,
    };
    let contents = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
    let future = simple_chatgpt::completions(&api_key, &request_body, |cr, completion| match cr {
        simple_chatgpt::CallbackReason::Start => {
            // nothing to do
        }
        simple_chatgpt::CallbackReason::Data => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            let content = completion.unwrap().choices[0]
                .delta
                .content
                .clone()
                .unwrap();
            let bytes = content.as_bytes();
            std::io::Write::write_all(&mut handle, bytes).unwrap();
            std::io::Write::flush(&mut handle).unwrap();
            contents.borrow_mut().push_str(&content);
        }
        simple_chatgpt::CallbackReason::End => {
            // nothing to do
        }
        simple_chatgpt::CallbackReason::Done => {
            println!();
        }
        simple_chatgpt::CallbackReason::Error(line) => {
            eprintln!("Error: {}", line);
        }
    });
    let result = futures::executor::block_on(future);
    if result.is_err() {
        eprintln!("Error: {:?}", result.err().unwrap());
        std::process::exit(1);
    }
    std::rc::Rc::try_unwrap(contents).unwrap().into_inner()
}

fn main() {
    let options = Options::parse();
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not defined");
    let mut messages = Vec::new();
    if let Some(args) = options.args {
        messages.push(simple_chatgpt::Message {
            role: Some(String::from("user")),
            content: Some(args.join(" ")),
        });
        let reply = request(
            &api_key,
            options.model.clone(),
            options.temperature.clone(),
            if options.no_stream { None } else { Some(true) },
            options.user.clone(),
            messages.clone(),
        );
        messages.push(simple_chatgpt::Message {
            role: Some(String::from("system")),
            content: Some(reply),
        });
    }
    let home = std::env::var("HOME").expect("HOME is not defined");
    let history_file = format!("{}{}.chatgpt_history", home, std::path::MAIN_SEPARATOR);
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    let _ = rl.load_history(&history_file);
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.clone());
                messages.push(simple_chatgpt::Message {
                    role: Some(String::from("user")),
                    content: Some(line),
                });
                let reply = request(
                    &api_key,
                    options.model.clone(),
                    options.temperature.clone(),
                    if options.no_stream { None } else { Some(true) },
                    options.user.clone(),
                    messages.clone(),
                );
                messages.push(simple_chatgpt::Message {
                    role: Some(String::from("system")),
                    content: Some(reply),
                });
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(&history_file).unwrap();
}
