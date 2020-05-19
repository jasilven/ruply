use anyhow::{anyhow, bail, format_err, Result};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, KeyPress};
use rustyline_derive::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::boxed::Box;
use structopt::StructOpt;

mod nrepl;
mod prepl;
mod repl;
mod util;
use repl::{Repl, Response};

#[derive(Helper)]
struct MyHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for MyHelper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for MyHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

fn main_loop(mut repl: Box<dyn Repl>) -> Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();

    let h = MyHelper {
        completer: rustyline::completion::FilenameCompleter::new(),
        highlighter: rustyline::highlight::MatchingBracketHighlighter::new(),
        hinter: rustyline::hint::HistoryHinter {},
        colored_prompt: "".to_owned(),
        validator: rustyline::validate::MatchingBracketValidator::new(),
    };
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyPress::Down, Cmd::LineDownOrNextHistory);
    rl.bind_sequence(KeyPress::Up, Cmd::LineUpOrPreviousHistory);
    rl.bind_sequence(KeyPress::Meta('N'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyPress::Meta('P'), Cmd::HistorySearchBackward);

    loop {
        let p = &format!("{}=> ", repl.namespace());
        rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{}\x1b[0m", p);
        match rl.readline(&p) {
            Ok(line) => {
                if !line.trim().is_empty() {
                    rl.add_history_entry(&line);
                    repl.send(&line)?;
                } else {
                    continue;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
        loop {
            match repl.recv()? {
                Response::StdErr(s) => {
                    print!("{}", &s);
                }
                Response::StdOut(s) => {
                    print!("{}", &s);
                }
                Response::Exception(s) => {
                    println!("{}", &s);
                    break;
                }
                Response::Other(_) => {}
                Response::Done(opt) => {
                    if let Some(s) = opt {
                        println!("{}", &s);
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}

#[derive(StructOpt, Debug)]
#[structopt(name = "ruply")]
struct Opt {
    /// Repl host
    #[structopt(short, default_value = "127.0.0.1")]
    host: String,

    /// Repl port
    #[structopt(short)]
    port: usize,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let repl = repl::get_repl(&opt.host, opt.port)?;

    println!(
        "\nConnected to {} at {}:{}",
        repl.name(),
        &opt.host,
        opt.port
    );
    println!("Exit: CTRL+D\n");

    main_loop(repl)?;

    Ok(())
}
