mod app;
mod config;
mod content_ops;
mod input;
mod json_ops;
mod markdown_ops;
mod navigation;
mod rendering;
mod syntax_highlight;
mod toon_ops;
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
        .version(env!("BUILD_VERSION"))
        .about("A vim-like TUI for managing notes and resources")
        .after_help(
            "EXAMPLES:\n  \
            # Open file in interactive mode\n  \
            revw file.json\n  \
            revw file.md\n  \
            revw file.toon\n\n  \
            # Output to stdout\n  \
            revw --stdout file.json\n  \
            revw --stdout file.md\n  \
            revw --stdout file.toon\n\n  \
            # Output to file\n  \
            revw --output output.txt file.json\n  \
            revw --output output.txt file.md\n  \
            revw --output output.txt file.toon\n\n  \
            # Output in different formats\n  \
            revw --stdout --markdown file.json\n  \
            revw --stdout --json file.md\n  \
            revw --stdout --toon file.json\n\n  \
            # Export to PDF\n  \
            revw --pdf file.json\n  \
            revw --pdf file.md\n  \
            revw --pdf file.toon\n\n  \
            # Input from file (supports .json, .md, .toon)\n  \
            revw --input data.json file.json\n  \
            revw --input data.toon file.md\n  \
            revw --input data.md file.toon\n\n  \
            # Section-specific operations\n  \
            revw --input data.json --inside file.json\n  \
            revw --input data.toon --outside file.md\n  \
            revw --input data.json --append --inside file.toon\n\n\
            SUPPORTED FILE FORMATS:\n  \
            JSON (file.json):\n  \
            {\n    \
            \"outside\": [{\"name\": \"Resource\", \"context\": \"Description\", \"url\": \"https://...\", \"percentage\": 100}],\n    \
            \"inside\": [{\"date\": \"2025-01-01 00:00:00\", \"context\": \"Note content\"}]\n  \
            }\n\n  \
            Markdown (file.md):\n  \
            ## OUTSIDE\n  \
            ### Resource\n  \
            Description\n  \
            **URL:** https://...\n  \
            **Percentage:** 100%\n  \
            ## INSIDE\n  \
            ### 2025-01-01 00:00:00\n  \
            Note content\n\n  \
            Toon (file.toon):\n  \
            outside[1]{name,context,url,percentage}:\n    \
            Resource,Description,https://...,100\n  \
            inside[1]{date,context}:\n    \
            2025-01-01 00:00:00,Note content\n\n\
            For more help, run 'revw' and press :h or ?"
        )
        .arg(Arg::new("file").help("JSON, Markdown, or Toon file to view").index(1))
        .arg(
            Arg::new("edit")
                .long("edit")
                .help("Use Edit mode")
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
                .long("output")
                .help("Output to file (use '-' for stdout)")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("inside")
                .long("inside")
                .help("Output only INSIDE section")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("outside")
                .long("outside")
                .help("Output only OUTSIDE section")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("markdown")
                .long("markdown")
                .help("Output in Markdown format")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Output in JSON format")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("toon")
                .long("toon")
                .help("Output in Toon format")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("pdf")
                .long("pdf")
                .help("Export to PDF format")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("input")
                .long("input")
                .help("Input from file")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("append")
                .long("append")
                .help("Append mode - append input instead of overwriting")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let format_mode = if matches.get_flag("edit") {
        FormatMode::Edit
    } else {
        FormatMode::View
    };

    let stdout_mode = matches.get_flag("stdout");
    let output_file = matches.get_one::<String>("output");
    let inside_only = matches.get_flag("inside");
    let outside_only = matches.get_flag("outside");
    let markdown_mode = matches.get_flag("markdown");
    let json_mode = matches.get_flag("json");
    let toon_mode = matches.get_flag("toon");
    let pdf_mode = matches.get_flag("pdf");
    let input_file = matches.get_one::<String>("input");
    let append_mode = matches.get_flag("append");

    // If PDF mode, export to PDF and exit
    if pdf_mode {
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

            // Check if file is Markdown or Toon
            let is_markdown = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false);
            let is_toon = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("toon"))
                .unwrap_or(false);

            if is_markdown {
                app.file_path = Some(path);
                app.markdown_input = content;
                // Convert markdown to JSON for processing
                if let Ok(json) = app.parse_markdown(&app.markdown_input.clone()) {
                    app.json_input = json;
                }
                app.convert_json();
            } else if is_toon {
                app.file_path = Some(path);
                app.toon_input = content;
                // Convert toon to JSON for processing
                if let Ok(json) = app.parse_toon(&app.toon_input.clone()) {
                    app.json_input = json;
                }
                app.convert_json();
            } else {
                app.file_path = Some(path);
                app.json_input = content;
                app.convert_json();
            }

            // Export to PDF
            match app.export_to_pdf() {
                Ok(pdf_path) => {
                    println!("PDF exported to: {}", pdf_path);
                }
                Err(e) => {
                    eprintln!("Error: Failed to export PDF: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Error: No input file specified for PDF export");
            std::process::exit(1);
        }
    } else if stdout_mode || output_file.is_some() {
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

            // Check if file is Markdown or Toon
            let is_markdown = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false);
            let is_toon = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("toon"))
                .unwrap_or(false);

            if is_markdown {
                app.file_path = Some(path);
                app.markdown_input = content;
                // Convert markdown to JSON for processing
                if let Ok(json) = app.parse_markdown(&app.markdown_input.clone()) {
                    app.json_input = json;
                }
                app.convert_json();
            } else if is_toon {
                app.file_path = Some(path);
                app.toon_input = content;
                // Convert toon to JSON for processing
                if let Ok(json) = app.parse_toon(&app.toon_input.clone()) {
                    app.json_input = json;
                }
                app.convert_json();
            } else {
                app.json_input = content;
                app.convert_json();
            }

            let output = if format_mode == FormatMode::Edit {
                // In Edit mode, output the JSON as-is
                app.json_input.clone()
            } else if markdown_mode {
                // Markdown mode: format entries as Markdown
                let mut output_lines = Vec::new();

                // Parse JSON to determine which section each entry belongs to
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&app.json_input) {
                    if let Some(obj) = json_value.as_object() {
                        // OUTSIDE section
                        if !inside_only || outside_only {
                            if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                                if !outside.is_empty() {
                                    output_lines.push("## OUTSIDE".to_string());
                                    output_lines.push("".to_string());

                                    for item in outside {
                                        if let Some(item_obj) = item.as_object() {
                                            let name = item_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                            let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                                            let url = item_obj.get("url").and_then(|v| v.as_str());
                                            let percentage = item_obj.get("percentage").and_then(|v| v.as_i64());

                                            if !name.is_empty() {
                                                output_lines.push(format!("### {}", name));
                                            }

                                            // Replace literal \n with actual newlines in context
                                            if !context.is_empty() {
                                                let formatted_context = context.replace("\\n", "\n");
                                                output_lines.push(formatted_context);
                                            }

                                            // Only output URL if it's not null and not empty
                                            if let Some(url_str) = url {
                                                if !url_str.is_empty() {
                                                    output_lines.push("".to_string());
                                                    output_lines.push(format!("**URL:** {}", url_str));
                                                }
                                            }

                                            // Only output percentage if it's not null
                                            if let Some(pct) = percentage {
                                                output_lines.push("".to_string());
                                                output_lines.push(format!("**Percentage:** {}%", pct));
                                            }

                                            // Only add blank line if we had any content
                                            if !name.is_empty() || !context.is_empty() || url.is_some() || percentage.is_some() {
                                                output_lines.push("".to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // INSIDE section
                        if !outside_only || inside_only {
                            if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                                if !inside.is_empty() {
                                    output_lines.push("## INSIDE".to_string());
                                    output_lines.push("".to_string());

                                    for item in inside {
                                        if let Some(item_obj) = item.as_object() {
                                            let date = item_obj.get("date").and_then(|v| v.as_str()).unwrap_or("");
                                            let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");

                                            if !date.is_empty() {
                                                output_lines.push(format!("### {}", date));
                                            }

                                            // Replace literal \n with actual newlines in context
                                            if !context.is_empty() {
                                                let formatted_context = context.replace("\\n", "\n");
                                                output_lines.push(formatted_context);
                                            }

                                            // Only add blank line if we had content
                                            if !date.is_empty() || !context.is_empty() {
                                                output_lines.push("".to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                output_lines.join("\n")
            } else if json_mode {
                // JSON mode: output as JSON (already in JSON format in app.json_input)
                // Apply section filtering if needed
                if inside_only || outside_only {
                    if let Ok(mut json_value) = serde_json::from_str::<serde_json::Value>(&app.json_input) {
                        if let Some(obj) = json_value.as_object_mut() {
                            if inside_only {
                                obj.remove("outside");
                            }
                            if outside_only {
                                obj.remove("inside");
                            }
                            serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| app.json_input.clone())
                        } else {
                            app.json_input.clone()
                        }
                    } else {
                        app.json_input.clone()
                    }
                } else {
                    // Output full JSON
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&app.json_input) {
                        serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| app.json_input.clone())
                    } else {
                        app.json_input.clone()
                    }
                }
            } else if toon_mode {
                // Toon mode: convert JSON to Toon format
                let json_to_convert = if inside_only || outside_only {
                    if let Ok(mut json_value) = serde_json::from_str::<serde_json::Value>(&app.json_input) {
                        if let Some(obj) = json_value.as_object_mut() {
                            if inside_only {
                                obj.remove("outside");
                            }
                            if outside_only {
                                obj.remove("inside");
                            }
                            serde_json::to_string(&json_value).unwrap_or_else(|_| app.json_input.clone())
                        } else {
                            app.json_input.clone()
                        }
                    } else {
                        app.json_input.clone()
                    }
                } else {
                    app.json_input.clone()
                };

                // Convert to Toon format
                match toon_ops::ToonOperations::json_to_toon(&json_to_convert) {
                    Ok(toon_content) => toon_content,
                    Err(e) => {
                        eprintln!("Error converting to Toon: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // In View mode, format the entries for text output
                if app.relf_entries.is_empty() {
                    // No entries parsed, output raw content or rendered lines
                    if !app.rendered_content.is_empty() {
                        app.rendered_content.join("\n")
                    } else {
                        app.json_input.clone()
                    }
                } else {
                    // Format entries as text
                    let mut output_lines = Vec::new();
                    let mut outside_entries: Vec<String> = Vec::new();
                    let mut inside_entries: Vec<String> = Vec::new();

                    // Parse JSON to determine which section each entry belongs to
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&app.json_input) {
                        if let Some(obj) = json_value.as_object() {
                            if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                                for item in outside {
                                    if let Some(item_obj) = item.as_object() {
                                        let name = item_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                        let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                                        let url = item_obj.get("url").and_then(|v| v.as_str()).unwrap_or("");
                                        let percentage = item_obj.get("percentage").and_then(|v| v.as_i64());

                                        let mut entry = String::new();
                                        entry.push_str(name);
                                        if !context.is_empty() {
                                            entry.push_str(&format!("\n{}", context));
                                        }
                                        if !url.is_empty() {
                                            entry.push_str(&format!("\n{}", url));
                                        }
                                        // Only add percentage if not null
                                        if let Some(pct) = percentage {
                                            entry.push_str(&format!("\n{}%", pct));
                                        }
                                        outside_entries.push(entry);
                                    }
                                }
                            }

                            if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                                for item in inside {
                                    if let Some(item_obj) = item.as_object() {
                                        let mut entry_parts = Vec::new();
                                        for (_key, value) in item_obj {
                                            let value_str = match value {
                                                serde_json::Value::String(s) => s.clone(),
                                                serde_json::Value::Number(n) => n.to_string(),
                                                serde_json::Value::Bool(b) => b.to_string(),
                                                _ => value.to_string(),
                                            };
                                            if !value_str.is_empty() {
                                                entry_parts.push(value_str);
                                            }
                                        }
                                        inside_entries.push(entry_parts.join("\n"));
                                    }
                                }
                            }
                        }
                    }

                    // Filter based on --inside or --outside flags
                    if inside_only && !outside_only {
                        // Only INSIDE section
                        if !inside_entries.is_empty() {
                            output_lines.push("INSIDE".to_string());
                            output_lines.push("".to_string());
                            for entry in inside_entries {
                                output_lines.push(entry);
                                output_lines.push("".to_string());
                            }
                        }
                    } else if outside_only && !inside_only {
                        // Only OUTSIDE section
                        if !outside_entries.is_empty() {
                            output_lines.push("OUTSIDE".to_string());
                            output_lines.push("".to_string());
                            for entry in outside_entries {
                                output_lines.push(entry);
                                output_lines.push("".to_string());
                            }
                        }
                    } else {
                        // Both sections (default behavior)
                        if !outside_entries.is_empty() {
                            output_lines.push("OUTSIDE".to_string());
                            output_lines.push("".to_string());
                            for entry in outside_entries {
                                output_lines.push(entry);
                                output_lines.push("".to_string());
                            }
                        }

                        if !inside_entries.is_empty() {
                            output_lines.push("INSIDE".to_string());
                            output_lines.push("".to_string());
                            for entry in inside_entries {
                                output_lines.push(entry);
                                output_lines.push("".to_string());
                            }
                        }
                    }

                    output_lines.join("\n")
                }
            };

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

        // Process input file if provided
        if let Some(input_path) = input_file {
            // Read from file
            let input_content = fs::read_to_string(input_path)
                .map_err(|e| {
                    eprintln!("Error: Cannot read input file '{}': {}", input_path, e);
                    std::process::exit(1);
                })
                .unwrap();

            // Convert input content format if needed (JSON -> Markdown or Markdown -> JSON)
            // Check if input is JSON and target is Markdown, or vice versa
            let input_path_obj = PathBuf::from(input_path);
            let input_is_json = input_path_obj.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false);

            let target_is_markdown = app.file_path.as_ref()
                .and_then(|p| p.extension())
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false);

            let converted_content = if input_is_json && target_is_markdown {
                // Convert JSON to Markdown format
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&input_content) {
                    let mut markdown_lines = Vec::new();

                    if let Some(obj) = json_val.as_object() {
                        // Convert OUTSIDE
                        if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                            if !outside.is_empty() {
                                markdown_lines.push("## OUTSIDE".to_string());
                                for item in outside {
                                    if let Some(item_obj) = item.as_object() {
                                        let name = item_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                        let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                                        let url = item_obj.get("url").and_then(|v| v.as_str());
                                        let percentage = item_obj.get("percentage").and_then(|v| v.as_i64());

                                        if !name.is_empty() {
                                            markdown_lines.push(format!("### {}", name));
                                        }
                                        if !context.is_empty() {
                                            let formatted_context = context.replace("\\n", "\n");
                                            markdown_lines.push(formatted_context);
                                        }
                                        if let Some(url_str) = url {
                                            if !url_str.is_empty() {
                                                markdown_lines.push("".to_string());
                                                markdown_lines.push(format!("**URL:** {}", url_str));
                                            }
                                        }
                                        if let Some(pct) = percentage {
                                            markdown_lines.push("".to_string());
                                            markdown_lines.push(format!("**Percentage:** {}%", pct));
                                        }
                                        markdown_lines.push("".to_string());
                                    }
                                }
                            }
                        }

                        // Convert INSIDE
                        if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                            if !inside.is_empty() {
                                markdown_lines.push("## INSIDE".to_string());
                                for item in inside {
                                    if let Some(item_obj) = item.as_object() {
                                        let date = item_obj.get("date").and_then(|v| v.as_str()).unwrap_or("");
                                        let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");

                                        if !date.is_empty() {
                                            markdown_lines.push(format!("### {}", date));
                                        }
                                        if !context.is_empty() {
                                            let formatted_context = context.replace("\\n", "\n");
                                            markdown_lines.push(formatted_context);
                                        }
                                        markdown_lines.push("".to_string());
                                    }
                                }
                            }
                        }
                    }

                    markdown_lines.join("\n")
                } else {
                    input_content.clone()
                }
            } else if !input_is_json && !target_is_markdown {
                // Convert Markdown to JSON format
                match app.parse_markdown(&input_content) {
                    Ok(json) => json,
                    Err(_) => input_content.clone()
                }
            } else {
                input_content.clone()
            };

            // Determine how to process the input based on flags
            if append_mode {
                // Append mode: :va/:vai/:vao behavior
                if inside_only {
                    // Append INSIDE only (:vai)
                    app.paste_inside_append_from_text(&converted_content);
                } else if outside_only {
                    // Append OUTSIDE only (:vao)
                    app.paste_outside_append_from_text(&converted_content);
                } else {
                    // Append both (:va)
                    app.paste_all_append_from_text(&converted_content);
                }
            } else {
                // Overwrite mode: :v/:vi/:vo behavior
                if inside_only {
                    // Overwrite INSIDE only (:vi)
                    app.paste_inside_from_text(&converted_content);
                } else if outside_only {
                    // Overwrite OUTSIDE only (:vo)
                    app.paste_outside_from_text(&converted_content);
                } else {
                    // Overwrite all (:v)
                    app.paste_from_text(&converted_content);
                }
            }

            // Save and exit immediately when --input is used
            app.save_file();
            return Ok(());
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
