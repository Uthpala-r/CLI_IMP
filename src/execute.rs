use std::collections::HashMap;
use crate::Clock;
use crate::CliContext;
use crate::commandcompleter::CommandCompleter;
use crate::run_config::help_command;

#[derive(Clone)]
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub suggestions: Option<Vec<&'static str>>,
    pub suggestions1: Option<Vec<&'static str>>,
    pub suggestions2: Option<Vec<&'static str>>,
    pub options: Option<Vec<&'static str>>,
    pub execute: fn(&[&str], &mut CliContext, &mut Option<Clock>) -> Result<(), String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    UserMode,
    PrivilegedMode,
    ConfigMode,
    InterfaceMode,
}

pub fn execute_command(input: &str, commands: &HashMap<&str, Command>, context: &mut CliContext, clock: &mut Option<Clock>, completer: &mut CommandCompleter) {
    let mut normalized_input = input.trim();
    let showing_suggestions = normalized_input.ends_with('?');
    
    if showing_suggestions {
        normalized_input = normalized_input.trim_end_matches('?');
    }
     
    let parts: Vec<&str> = normalized_input.split_whitespace().collect();
   
    let available_commands = get_mode_commands(commands, &context.current_mode);

    // Handle command execution (when no '?' is present)
    if !showing_suggestions {
        let cmd_key = parts[0];
        
        /// Check if command exists in current mode
        let allowed_commands_for_walkup = get_mode_commands(commands, &context.current_mode);
        let cmd_in_current_mode = find_unique_command(cmd_key, &allowed_commands_for_walkup);
        
        if let Some(matched_cmd) = cmd_in_current_mode {
            execute_matched_command(matched_cmd, &parts, commands, context, clock);
        }
        return;
    }

    // Handle suggestions '?' logic
    match parts.len() {
        0 => {
            // Handle single word with ? (e.g., "?")
            help_command(&context);
            
        },            
        1 => {
            let command_name = parts[0].trim();
            // Handle single word with ? (e.g., "configure ?")
            let available_commands = get_mode_commands(commands, &context.current_mode);
            if available_commands.contains(&command_name) {
                // If it's an exact command match, show its subcommands
                if let Some(cmd) = commands.get(command_name) {
                    if let Some(suggestions) = &cmd.suggestions1 {
                        println!("Possible completions:");
                        for suggestion in suggestions {
                            println!("  {}", suggestion);
                        }
                    } else if let Some(options) = &cmd.options {
                        // Fall back to options if no suggestions1 are available
                        println!("Possible completions:");
                        for option in options {
                            println!("  {}", option);
                        }
                    } else {
                        println!("No subcommands or more options available");
                    }
                }
            } else {
                // If it's a partial command, show matching commands
                let suggestions: Vec<&str> = available_commands
                    .into_iter()
                    .filter(|cmd| cmd.starts_with(command_name))
                    .collect();

                if !suggestions.is_empty() {
                    println!("Possible completions for '{}?':", command_name);
                    for suggestion in suggestions {
                        println!("  {}", suggestion);
                    }
                } else {
                    if let Some(cmd) = commands.get(parts[0]) {
                        if let Some(options) = &cmd.options {
                            println!("Possible completions:");
                            for option in options {
                                println!("  {}", option);
                            }
                        } else {
                            println!("No more options available");
                        }
                    }
                }
            }
        },
        2 => {
            // Command with partial subcommand (e.g., "configure t?", "configure term?")
            let available_commands = get_mode_commands(commands, &context.current_mode);
            if available_commands.contains(&parts[0]) && !normalized_input.ends_with(' ') {
                if let Some(cmd) = commands.get(parts[0]) {
                    if let Some(suggestions) = &cmd.suggestions1 {
                        let partial = parts[1];
                        let matching: Vec<&str> = suggestions
                            .iter()
                            .filter(|&&s| s.starts_with(partial))
                            .map(|&s| s)
                            .collect();

                        if !matching.is_empty() {
                            println!("Possible completions:");
                            for suggestion in matching {
                                println!("  {}", suggestion);
                            }
                        } else {
                            println!("No matching commands found");
                        }
                    } else {
                        println!("No subcommands available");
                    }
                }
            } else {
                if let Some(cmd) = commands.get(parts[0]) {
                    if let Some(options) = &cmd.options {
                        println!("Possible completions:");
                        for option in options {
                            println!("  {}", option);
                        }
                    } else {
                        println!("No more options available");
                        //(cmd.execute)(&parts[1..], context, clock);
                    }
                }
            }
            // Handle subcommand suggestions
            let command_name = parts[0];
            let subcommand = parts[1];

            if let Some(cmd) = commands.get(command_name) {
                match command_name {
                    "ntp" => {
                        match subcommand {
                            "source" => {
                                println!("Possible completions:");
                                println!("<interface_name>  - Set source interface for NTP packets");
                            },
                            "server" => {
                                println!("Possible completions:");
                                println!("<ip-address>      - Configure NTP server with IPv4 address");
                            },
                            "authentication-key" => {
                                println!("Possible completions:");
                                println!("<key-number>      - Configure key number (1-65535)");
                            },
                            "trusted-key" => {
                                println!("Possible completions:");
                                println!("<key-number>      - Trusted authentication key number");
                            },
                            &_ => {}
                        }
                    },
                    // Add other commands here as needed
                    &_ => {}
                        
                }
            }
        },
        3 => {
            // Handle third-level suggestions (e.g., after "ntp server" or "ntp source")
            let command_name = parts[0];
            let subcommand = parts[1];

            if command_name == "ntp" {
                match subcommand {
                    "authentication-key" => {
                        println!("Possible completions:");
                        println!("md5              - Keyed Message Digest 5 algorithm");
                    },
                    _ => println!("No additional parameters available")
                }
            } else {
                println!("No additional parameters available");
            }
        },
        4 => {
            // Handle third-level suggestions (e.g., after "ntp server" or "ntp source")
            let command_name = parts[0];
            let subcommand = parts[1];

            if command_name == "ntp" {
                match subcommand {
                    "authentication-key" => {
                        println!("Possible completions:");
                        println!("<key-name>              - Configure key name");
                    },
                    _ => println!("No additional parameters available")
                }
            } else {
                println!("No additional parameters available");
            }
        },
        _ => {
            // Full command with ? (e.g., "configure terminal ?")
            println!("No additional parameters available");
        }
    }
    return;
}


fn execute_matched_command(matched_cmd: &str, parts: &[&str], commands: &HashMap<&str, Command>, context: &mut CliContext, clock: &mut Option<Clock>) {
    if let Some(cmd) = commands.get(matched_cmd) {
        execute_command_with_args(cmd, parts, context, clock);
    }
}

fn execute_command_with_args(cmd: &Command, parts: &[&str], context: &mut CliContext, clock: &mut Option<Clock>) {
    if let Some(suggestions) = &cmd.suggestions1 {
        match parts.len() {
            1 => {
                println!("Incomplete command. Subcommand required.");
            }
            2 => {
                if suggestions.is_empty() {
                    if let Err(err) = (cmd.execute)(&parts[1..], context, clock) {
                        println!("Error: {}", err);
                    }
                } else {
                    if let Some(matched_subcommand) = find_unique_subcommand(parts[1], suggestions) {
                        if let Err(err) = (cmd.execute)(&[matched_subcommand], context, clock) {
                            println!("Error: {}", err);
                        }
                    } else {
                        println!("Ambiguous or invalid subcommand: {}", parts[1]);
                    }
                }
            }
            _ => {
                if let Err(err) = (cmd.execute)(&parts[1..], context, clock) {
                    println!("Error: {}", err);
                }
            }
        }
    } else {
        if let Err(err) = (cmd.execute)(&parts[1..], context, clock) {
            println!("Error: {}", err);
        }
    }
}


// Get available commands for current mode
pub fn get_mode_commands<'a>(commands: &'a HashMap<&str, Command>, mode: &Mode) -> Vec<&'a str> {
    match mode {
        Mode::UserMode => {
            commands.keys()
                .filter(|&&cmd| {
                    cmd == "enable" ||
                    cmd == "ping" ||
                    cmd == "help" ||
                    cmd == "show" ||
                    cmd == "clear" ||
                    cmd == "reload" ||
                    cmd == "poweroff" ||
                    cmd == "connect" ||
                    cmd == "disable" ||
                    cmd == "ifconfig" ||
                    cmd == "traceroute" ||
                    cmd == "do" ||
                    cmd == "ip" ||
                    cmd == "netplan" ||
                    cmd == "exit"
                })
                .copied()
                .collect()
        },
        Mode::PrivilegedMode => {
            commands.keys()
                .filter(|&&cmd| {
                    cmd == "configure" ||
                    cmd == "ping" || 
                    cmd == "exit" || 
                    cmd == "write" ||
                    cmd == "help" ||
                    cmd == "show" ||
                    cmd == "copy" ||
                    cmd == "clock" ||
                    cmd == "clear" ||
                    cmd == "reload" ||
                    cmd == "poweroff" ||
                    cmd == "debug" ||
                    cmd == "undebug" ||
                    cmd == "connect" ||
                    cmd == "disable" ||
                    cmd == "traceroute" ||
                    cmd == "ssh" ||
                    cmd == "do" ||
                    cmd == "ip" ||
                    cmd == "netplan" ||
                    cmd == "ifconfig"
                })
                .copied()
                .collect()
        },
        Mode::ConfigMode => {
            commands.keys()
                .filter(|&&cmd| {
                    cmd == "hostname" || 
                    cmd == "ping" ||
                    cmd == "exit" ||
                    cmd == "clear" ||
                    cmd == "help" ||
                    cmd == "write" ||
                    cmd == "service" ||
                    cmd == "set" ||
                    cmd == "enable" ||
                    cmd == "copy" ||
                    cmd == "ifconfig" ||  
                    cmd == "ntp" ||
                    cmd == "no" || 
                    cmd == "reload" ||
                    cmd == "poweroff" ||
                    cmd == "connect" ||
                    cmd == "disable" ||
                    cmd == "traceroute" ||
                    cmd == "interface" ||
                    cmd == "ip" ||
                    cmd == "netplan" ||
                    cmd == "do"
                })
                .copied()
                .collect()
        },
        Mode::InterfaceMode => {
            commands.keys()
                .filter(|&&cmd| {
                    cmd == "shutdown" ||
                    cmd == "no" ||
                    cmd == "exit" ||
                    cmd == "clear" ||
                    cmd == "help" ||
                    cmd == "write" ||
                    cmd == "reload" ||
                    cmd == "poweroff" ||
                    cmd == "ip" ||
                    cmd == "netplan" ||
                    cmd == "do" 
                })
                .copied()
                .collect()
        }
    }
}

pub fn find_unique_command<'a>(partial: &str, available_commands: &[&'a str]) -> Option<&'a str> {
    let matches: Vec<&str> = available_commands
        .iter()
        .filter(|&&cmd| cmd.starts_with(partial))
        .copied()
        .collect();

    if matches.len() == 1 {
        Some(matches[0])
    } else {
        None
    }
}

pub fn find_unique_subcommand<'a>(partial: &str, suggestions: &'a [&str]) -> Option<&'a str> {
    let matches: Vec<&str> = suggestions
        .iter()
        .filter(|&&s| s.starts_with(partial))
        .copied()
        .collect();

    if matches.len() == 1 {
        Some(matches[0])
    } else {
        None
    }
}

