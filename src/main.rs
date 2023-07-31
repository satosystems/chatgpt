use clap::Parser;

#[derive(Debug, Parser)]
#[command(version)]
struct Options {
    #[arg(short, long, default_value = "gpt-3.5-turbo")]
    model: String,

    args: Option<Vec<String>>,
}

fn main() {
    let options = Options::parse();
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not defined");
    if let Some(args) = options.args {
        let request_body = chatgpt::RequestBody {
            model: options.model,
            messages: vec![chatgpt::Message {
                role: Some(String::from("user")),
                content: Some(args.join(" ")),
            }],
            temperature: None,
            stream: Some(true),
            user: None,
        };
        let future = chatgpt::completions(&api_key, &request_body, |cr, completion| {
            if cr == chatgpt::CallbackReason::Data {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                let content = completion.choices[0].delta.content.clone().unwrap();
                let bytes = content.as_bytes();
                std::io::Write::write_all(&mut handle, bytes).unwrap();
                std::io::Write::flush(&mut handle).unwrap();
            } else if cr == chatgpt::CallbackReason::End {
                println!();
            }
        });
        let result = futures::executor::block_on(future);
        if result.is_err() {
            eprintln!("Error: {:?}", result.err().unwrap());
            std::process::exit(1);
        }
    }
}
