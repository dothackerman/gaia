use std::env;
use std::io::{self, IsTerminal, Write};
use std::process::{Command, Stdio};

pub fn print_or_page(content: &str, force_pager: bool, disable_pager: bool) -> io::Result<()> {
    let use_pager = if disable_pager {
        false
    } else if force_pager {
        true
    } else {
        std::io::stdout().is_terminal()
    };

    if !use_pager {
        print!("{content}");
        return Ok(());
    }

    let pager = env::var("PAGER")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "less -FR".to_string());

    let mut child = match Command::new("sh")
        .arg("-c")
        .arg(&pager)
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            print!("{content}");
            return Ok(());
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes())?;
    }

    let _ = child.wait();
    Ok(())
}
