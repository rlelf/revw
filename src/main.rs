mod app;
mod input;
mod json_ops;
mod navigation;
mod rendering;
mod ui;

use anyhow::Result;
use clap::{Arg, Command};
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{fs, io::stdout, panic, path::PathBuf};

use app::{App, FormatMode};

fn main() -> Result<()> {
    // Set up panic handler to properly clean up terminal on crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Clean up terminal
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        let _ = execute!(stdout(), cursor::Show);

        // Call the original panic handler
        original_hook(panic_info);
    }));

    let matches = Command::new("revw")
        .version("0.1.0")
        .about("JSON to readable format viewer")
        .arg(Arg::new("file").help("JSON file to view").index(1))
        .arg(
            Arg::new("json")
                .long("json")
                .help("Use JSON editing mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stdout")
                .long("stdout")
                .help("Output to stdout instead of interactive mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output to file (use '-' for stdout)")
                .value_name("FILE"),
        )
        .get_matches();

    let format_mode = if matches.get_flag("json") {
        FormatMode::Edit
    } else {
        FormatMode::View
    };

    let stdout_mode = matches.get_flag("stdout");
    let output_file = matches.get_one::<String>("output");

    // If stdout mode or output file specified, run in non-interactive mode
    if stdout_mode || output_file.is_some() {
        let mut app = App::new(format_mode);

        // Load file if provided
        if let Some(file_path) = matches.get_one::<String>("file") {
            let path = PathBuf::from(file_path);
            let content = fs::read_to_string(&path)
                .map_err(|e| {
                    eprintln!("Error: Cannot read file '{}': {}", file_path, e);
                    std::process::exit(1);
                })
                .unwrap();

            app.json_input = content;
            app.convert_json();

            let output = app.rendered_content.join("\n");

            if let Some(output_path) = output_file {
                if output_path == "-" {
                    // Output to stdout
                    println!("{}", output);
                } else {
                    // Output to file
                    fs::write(output_path, output)?;
                }
            } else {
                // stdout flag was used
                println!("{}", output);
            }
        } else {
            eprintln!("Error: No input file specified");
            std::process::exit(1);
        }
    } else {
        // Interactive mode with better error handling
        let mut app = App::new(format_mode);

        // Load file if provided - no existence check for quick loading
        if let Some(file_path) = matches.get_one::<String>("file") {
            let path = PathBuf::from(file_path);
            app.load_file(path);
        }

        // Set up terminal with error handling
        let setup_result = (|| -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
            enable_raw_mode()?;
            let mut stdout = stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            execute!(stdout, cursor::Hide)?;
            let backend = CrosstermBackend::new(stdout);
            Ok(Terminal::new(backend)?)
        })();

        let mut terminal = match setup_result {
            Ok(term) => term,
            Err(e) => {
                eprintln!("Failed to initialize terminal: {}", e);
                return Err(e);
            }
        };

        // Run the app with proper cleanup
        let res = input::run_app(&mut terminal, app);

        // Always clean up, even if there was an error
        let _ = disable_raw_mode();
        let _ = execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = execute!(terminal.backend_mut(), cursor::Show);
        let _ = terminal.show_cursor();

        if let Err(err) = res {
            eprintln!("Application error: {}", err);
            std::process::exit(1);
        }
    }

    Ok(())
}
