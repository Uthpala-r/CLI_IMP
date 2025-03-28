/// External crates for the CLI application
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::Path;
use std::fs::File;
use std::fs;
use std::io::{self, Write, BufRead, BufReader};
use std::str::FromStr;
use rpassword::read_password;
use std::process::Command as ProcessCommand;
use std::process::Stdio;

use crate::run_config::{get_running_config, help_command, save_running_to_startup};
use crate::execute::Command;
use crate::cliconfig::CliContext;
use crate::execute::Mode;
use crate::clock_settings::{handle_clock_set, parse_clock_set_input, handle_show_clock, handle_show_uptime};
use crate::network_config::{calculate_broadcast, get_netplan_file, execute_command_with_output, terminate_ssh_session, get_available_int, ip_with_cidr, print_interface, format_flags, get_system_interfaces, connect_via_ssh, execute_spawn_process, InterfaceConfig, InterfacesConfig, STATUS_MAP, IP_ADDRESS_STATE,  SELECTED_INTERFACE, ROUTE_TABLE};
use crate::network_config::NtpAssociation;
use crate::passwd::{PASSWORD_STORAGE, set_enable_password, set_enable_secret, get_enable_password, get_enable_secret, encrypt_password};
use crate::show_c::{show_clock, show_uptime, show_version, show_sessions, show_controllers, show_history, show_run_conf, show_start_conf, show_interfaces, show_ip_int_br, show_ip_route, show_login, show_ntp, show_ntp_asso, show_proc};

/// Builds and returns a `HashMap` of available commands, each represented by a `Command` structure.
/// 
/// This function initializes a registry of commands that can be executed in different modes
/// (e.g., `UserMode`, `PrivilegedMode`, `ConfigMode`, etc.) within a router-like system.
/// Each command is associated with a name, description, suggestions for usage, and an execution
/// function that defines its behavior.
///
/// The commands registered include:
/// - `enable`: Switches from User EXEC mode to Privileged EXEC mode.
///     - `enable secret`: Sets a secret password for privileged EXEC mode access, using a stronger hash for security than the `enable password` command.
///     - `enable password`: Configures a password for privileged EXEC mode access. This password is weaker than the `enable secret` and should be avoided when possible.
/// - `configure terminal`: Enters Global Configuration mode.
/// - `interface`: Enters Interface Configuration mode for a specified interface. Should enter the interface name as an input
///     - `interface range`: Enters the Interface Configuration mode but for the entire range.
/// - `hostname`: Changes the hostname of the device.
/// - `ifconfig`: Displays or configures network details of the router.
/// - `exit`: This command navigates through the modes in reverse order (eg: ConfigMode --> UserMode)
/// - `disable`: Exits the privileged commands
/// - `show`: Displays all the show commands when specific command is passed
///     - `show running-config`: Displays the current running configuration from a JSON file.
///     - `show startup-config`: Displays the initial configuration settings stored in the NVRAM of a router, which are loaded upon booting the device.
///     - `show version`: Displays the software version information.
///     - `show clock`: Displays the current clock date and time.
///     - `show interfaces`: Displays statistics for all interfaces, including a brief overview or detailed information.
///     - `show ip interfaces brief`: Displays a summary of the router interfaces
///     - `show ntp associations`: Displays the status of NTP associations with servers or clients, showing the synchronization status and other details.
///     - `show ntp`: Displays information about the current NTP configuration, associations, and synchronization status.
///     - `show processes`: Shows the system processes and memories
///     - `show uptime`: Shows the time from the last reboot
///     - `show login`: Shows the details about login delay
/// - `reload`: Reloads the system
/// - `debug all`: Debug all the processors
/// - `undebug all`: Undebug all the processors
/// - `write memory`: Saves the running configuration to the startup configuration.
/// - `copy running-config`: Copies the running configuration to the startup configuration or to a new file if mentioned.
/// - `help`: Displays a list of available commands.
/// - `clock set`: Changes the device's clock date and time.
/// - `ip address`: Assigns an IP address and netmask to the selected interface.
/// - `shutdown`: Disable a router's interface
/// - `no`: Execute the opposite of the commands
///     - `no shutdown`: Enable a router's interface 
///     - `no ntp server`: Disable NTP
/// - `ntp`: Defines all the ntp commands
///     - `ntp source`: To use a particular source address in Network Time Protocol (NTP) packets
///     - `ntp server`: Configures the NTP server for synchronizing time on the device, ensuring that the device’s clock is accurate.
///     - `ntp master`: Configures the device as an NTP master, meaning it will serve time to other devices in the network.
///     - `ntp authenticate`: Enables NTP authentication, which allows the NTP client to authenticate time synchronization requests from servers.
///     - `ntp authentication-key`: Defines the key used for authenticating NTP messages, providing security to NTP transactions.
///     - `ntp trusted-key`: Specifies which authentication key(s) are trusted to authenticate NTP messages
/// - `service password-encryption`: Enables password encryption for storing sensitive passwords in the device’s configuration, ensuring they are not stored in plain text.
/// - `ssh`: Connect to ssh or define ssh version
/// - `ping`: Confirms the connection between ip addresses
/// - `traceroute`: To track the packet transfer path
/// 
/// # Returns
/// A `HashMap` where the keys are command names (as `&'static str`) and the values are the corresponding `Command` structs.
/// Each `Command` struct contains the `name`, `description`, `suggestions`, and an `execute` function.
pub fn build_command_registry() -> HashMap<&'static str, Command> {
    let mut commands = HashMap::new();

    commands.insert("enable", Command {
        name: "enable",
        description: "Enter privileged EXEC mode",
        suggestions: Some(vec!["password", "secret"]),
        suggestions1: None,
        suggestions2: None,
        options: Some(vec!["<password>      - Enter the password/secret>"]),
        execute: |args, context, _| {
            if args.is_empty(){
                if matches!(context.current_mode, Mode::UserMode) {

                    let stored_password = get_enable_password();
                    let stored_secret = get_enable_secret();

                    fn proceed_to_priv_mode(context: &mut CliContext){
                        context.current_mode = Mode::PrivilegedMode;
                        context.prompt = format!("{}#", context.config.hostname);
                        println!("Entering privileged EXEC mode...");
                    }
        
                    if stored_password.is_none() && stored_secret.is_none() {
                        // No passwords stored, directly go to privileged EXEC mode
                        proceed_to_priv_mode(context);
                        return Ok(());
                    }
        
                    // Prompt for the enable password
                    if stored_secret.is_none() && stored_password.is_some() {
                        println!("Enter password:");
                        let input_password = read_password().unwrap_or_else(|_| "".to_string());
                        let hashed_input = encrypt_password(&input_password);
            
                        if let Some(ref stored_password) = stored_password {
                            if hashed_input == *stored_password {
                                // Correct enable password, proceed to privileged mode
                                proceed_to_priv_mode(context);
                                return Ok(());
                            }
                        }
                        return Err("Incorrect password.".into());
                    }

                    if stored_password.is_none() && stored_secret.is_some(){
                        println!("Enter secret:");
                        let input_secret= read_password().unwrap_or_else(|_| "".to_string());
                        let hashed_input = encrypt_password(&input_secret);
            
                        if let Some(ref stored_secret) = stored_secret {
                            if hashed_input == *stored_secret {
                                // Correct enable password, proceed to privileged mode
                                proceed_to_priv_mode(context);
                                return Ok(());
                            }
                        }
                        return Err("Incorrect secret.".into());
                    }
            
                    // If secret is stored, prompt for it if password check fails
                    if stored_password.is_some() && stored_secret.is_some() {
                        println!("Enter password:");
                        let input_password = read_password().unwrap_or_else(|_| "".to_string());
                        println!("Enter secret:");
                        let input_secret = read_password().unwrap_or_else(|_| "".to_string());

                        // Hash both inputs
                        let hashed_password = encrypt_password(&input_password);
                        let hashed_secret = encrypt_password(&input_secret);
        
                        if let (Some(ref stored_secrets), Some(ref stored_passwords)) = 
                                (stored_secret, stored_password) {
                            if hashed_secret == *stored_secrets && hashed_password == *stored_passwords {
                                // Both correct, proceed to privileged mode
                                proceed_to_priv_mode(context);
                                return Ok(());
                            }
                        }
                        return Err("Incorrect password or secret.".into());
                    }
        
                    // If neither password nor secret matches, return an error
                    Err("Incorrect password or secret.".into())
                } else {
                    Err("The 'enable' command is only available in User EXEC mode.".into())
                }
            } else {
                match &args[0][..]{
                    "password" => {
                        if matches!(context.current_mode, Mode::ConfigMode) {
                            if args.len() != 2 {
                                Err("You must provide the enable password.".into())
                            } else {
                                let password = &args[1];
                                let hashed_password = encrypt_password(password);
                                set_enable_password(&hashed_password);
                                context.config.enable_password = Some(password.to_string());
                                println!("Enable password set.");
                                Ok(())
                            }
                        } else {
                            Err("The 'enable password' command is only available in Config mode.".into())
                        }
                    },
                    "secret" => {
                        if matches!(context.current_mode, Mode::ConfigMode) {
                            if args.len() != 2 {
                                Err("You must provide the enable secret password.".into())
                            } else {
                                let secret = &args[1];
                                let hashed_secret = encrypt_password(secret);
                                set_enable_secret(&hashed_secret);
                                context.config.enable_secret = Some(secret.to_string());
                                println!("Enable secret password set.");
                                Ok(())
                            }
                        } else {
                            Err("The 'enable secret' command is only available in Config mode.".into())
                        }
                    },
                    _=> Err(format!("Unknown enable subcommand: {}", args[0]).into())
                }
            }
        },
    });

    commands.insert("configure", Command {
        name: "configure terminal",
        description: "Enter global configuration mode",
        suggestions: Some(vec!["terminal"]),
        suggestions1: Some(vec!["terminal"]),
        suggestions2: None,
        options: None,
        execute: |args, context, _| {
            if matches!(context.current_mode, Mode::PrivilegedMode) {
                if args.len() == 1 && args[0] == "terminal" {
                    context.current_mode = Mode::ConfigMode;
                    context.prompt = format!("{}(config)#", context.config.hostname);
                    println!("Enter configuration commands, one per line.  End with CNTL/Z");
                    Ok(())
                } else {
                    Err("Invalid arguments provided to 'configure terminal'. This command does not accept additional arguments.".into())
                }
            } else {
                Err("The 'configure terminal' command is only available in Privileged EXEC mode.".into())
            }
        },
    });

    commands.insert("interface", Command {
        name: "interface",
        description: "Enter Interface configuration mode",
        suggestions: None,
        suggestions1: None,
        suggestions2: None,
        options: Some(vec!["<interface-name>    - Specify a valid interface name"]),
        execute: |args, context, _| {
            if matches!(context.current_mode, Mode::ConfigMode | Mode::InterfaceMode) {
                
                let (interface_list, interfaces_list) = match get_available_int() {
                    Ok(list) => list,
                    Err(e) => return Err(e),
                };
                
                //let args: Vec<String> = std::env::args().skip(1).collect();
                if args.is_empty() {
                    return Err(format!("Please specify a valid interface. Available interfaces: {}", interfaces_list));
                }
    
                if args.len() == 1 {
                    let net_interface = &args[0];
                    if interface_list.iter().any(|i| i == net_interface) {
                        context.current_mode = Mode::InterfaceMode;
                        context.selected_interface = Some(net_interface.to_string());
                        context.prompt = format!("{}(config-if)#", context.config.hostname);
                        println!("Entering Interface configuration mode for: {}", net_interface);

                        // Store the selected interface globally
                        let mut selected = SELECTED_INTERFACE.lock().unwrap();
                        *selected = Some(net_interface.to_string());

                        Ok(())
                    } else {
                        Err(format!("Invalid interface: {}. Available interfaces: {}", net_interface, interfaces_list))
                    }
                } else {
                    Err(format!("Invalid number of arguments. Usage: interface <interface-name>").into())
                }
            } else {
                Err("The 'interface' command is only available in Global Configuration mode and interface configuration mode.".into())
            }
        },
    });

    commands.insert("connect", Command {
        name: "connect",
        description: "Connect to network processor or crypto module",
        suggestions: Some(vec!["network", "crypto"]),
        suggestions1: Some(vec!["network", "crypto"]),
        suggestions2: None,
        options: None,
        execute: |args, _context, _| {    
            if args.len() != 1 {
                return Err("Invalid number of arguments. Usage: connect <network|crypto>".into());
            }
    
            match args[0] {
                "network" => {
                    println!("Connecting to network processor...");
                    //connect_via_ssh("pnfcli", "192.168.253.146")?; //Replace with actual details of NP
                    connect_via_ssh("root", "192.168.101.100")?;    //OpenWRT VM
                    println!("Connected successfully!");
                    Ok(())
                },
                "crypto" => {
                    println!("Connecting to crypto module...");
                    connect_via_ssh("pnfcli", "192.168.253.147")?; //Replace with actual details of SEM
                    println!("Connected successfully!");
                    Ok(())
                },
                _ => Err("Invalid argument. Use 'network' or 'crypto'".into())
            }
        },
    });


    commands.insert("exit", Command {
        name: "exit",
        description: "Exit the current mode and return to the previous mode.",
        suggestions: None,
        suggestions1: None,
        suggestions2: None,
        options: None,
        execute: |args, context, _| {
            if args.is_empty() {
                match context.current_mode {
                    Mode::InterfaceMode => {
                        context.current_mode = Mode::ConfigMode;
                        context.prompt = format!("{}(config)#", context.config.hostname);
                        println!("Exiting Interface Configuration Mode...");
                        Ok(())
                    }
                    Mode::ConfigMode => {
                        context.current_mode = Mode::PrivilegedMode;
                        context.prompt = format!("{}#", context.config.hostname);
                        println!("Exiting Global Configuration Mode...");
                        Ok(())
                    }
                    Mode::PrivilegedMode => {
                        context.current_mode = Mode::UserMode;
                        context.prompt = format!("{}>", context.config.hostname);
                        println!("Exiting Privileged EXEC Mode...");
                        Ok(())
                    }
                    Mode::UserMode => {
                        println!("Already at the top level. No mode to exit.");
                        Err("No mode to exit.".into())
                    }
                }
            } else if args.len() == 1 && args[0] == "ssh" {
                println!("Terminating SSH session...");
                terminate_ssh_session();
                Ok(())
            }
            else {
                Err("Command is either 'exit' , 'exit cli' or 'exit ssh'".into())
            }
        },
    });

    commands.insert("disable", Command {
        name: "disable",
        description: "Exit the Privileged EXEC mode and return to the USER EXEC mode.",
        suggestions: None,
        suggestions1: None,
        suggestions2: None,
        options: None,
        execute: |_args, context, _| {
            if _args.is_empty() {
                match context.current_mode {
                    Mode::InterfaceMode => {
                        println!("This command only works at the Privileged Mode.");
                        Err("This command only works at the Privileged Mode.".into())
                    
                    }
                    Mode::ConfigMode => {
                        println!("This command only works at the Privileged Mode.");
                        Err("This command only works at the Privileged Mode.".into())
                    
                    }
                    Mode::PrivilegedMode => {
                        context.current_mode = Mode::UserMode;
                        context.prompt = format!("{}>", context.config.hostname);
                        println!("Exiting Privileged EXEC Mode...");
                        Ok(())
                    }
                    Mode::UserMode => {
                        println!("Already at the top level. No mode to exit.");
                        Err("No mode to exit.".into())
                    }
                }
            } else {
                Err("Invalid arguments provided to 'exit'. This command does not accept additional arguments.".into())
            }
        },
    });
    
    commands.insert("reload", Command {
        name: "reload",
        description: "Reload the system",
        suggestions: None,
        suggestions1: None,
        suggestions2: None,
        options: None,
        execute: |_, context, _| {
    
            println!("Proceed with reload? [yes/no]:");
            let mut reload_confirm = String::new();
            std::io::stdin().read_line(&mut reload_confirm).expect("Failed to read input");
            let reload_confirm = reload_confirm.trim();
    
            if ["yes", "y", ""].contains(&reload_confirm.to_ascii_lowercase().as_str()) {
                  
                execute_spawn_process("sudo", &["reboot"]);
                Ok(())
                
            } else if ["no", "n"].contains(&reload_confirm.to_ascii_lowercase().as_str()) {
                println!("Reload aborted.");
                Ok(())
            } else {
                Err("Invalid input. Please enter 'yes', 'y', or 'no'.".into())
            }
        },
    });

    commands.insert("poweroff", Command {
        name: "poweroff",
        description: "Shutdown the Management PC",
        suggestions: None,
        suggestions1: None,
        suggestions2: None,
        options: None,
        execute: |_, context, _| {
    
            println!("Do you want to shutdown the PC? [yes/no]:");
            let mut reload_confirm = String::new();
            std::io::stdin().read_line(&mut reload_confirm).expect("Failed to read input");
            let reload_confirm = reload_confirm.trim();
    
            if ["yes", "y", ""].contains(&reload_confirm.to_ascii_lowercase().as_str()) {
                fs::remove_file("history.txt");  
                execute_spawn_process("sudo", &["shutdown", "now"]);
                Ok(())
                
            } else if ["no", "n"].contains(&reload_confirm.to_ascii_lowercase().as_str()) {
                println!("Reload aborted.");
                Ok(())
            } else {
                Err("Invalid input. Please enter 'yes', 'y', or 'no'.".into())
            }
        },
    });
    
    commands.insert("debug", Command {
        name: "debug all",
        description: "To turn on all the possible debug levels",
        suggestions: Some(vec!["all"]),
        suggestions1: Some(vec!["all"]),
        suggestions2: None,
        options: None,
        execute: |args, context, _| {
            if matches!(context.current_mode, Mode::PrivilegedMode) {
                if args.len() == 1 && args[0] == "all" {
                    println!("This may severely impact network performance. Continue? (yes/[no]):");
    
                    let mut save_input = String::new();
                    std::io::stdin().read_line(&mut save_input).expect("Failed to read input");
                    let save_input = save_input.trim();
            
                    if ["yes", "y", ""].contains(&save_input.to_ascii_lowercase().as_str()) {
                        println!("All possible debugging has been turned on");
                        //Execution must be provided
                        Ok(())
                    } else if ["no", "n"].contains(&save_input.to_ascii_lowercase().as_str()){
                        println!("Returned");
                        //Execution must be provided
                        Ok(())
                    } else {
                        return Err("Invalid input. Please enter 'yes' or 'no'.".into());
                    }
                } else {
                    Err("Invalid arguments provided to 'debug all'. This command does not accept additional arguments.".into())
                }
            } else {
                Err("The 'debug all' command is only available in Privileged EXEC mode.".into())
            }
        },
    });

    commands.insert("undebug", Command {
        name: "undebug all",
        description: "Turning off all possible debugging processes",
        suggestions: Some(vec!["all"]),
        suggestions1: Some(vec!["all"]),
        suggestions2: None,
        options: None,
        execute: |args, context, _| {
            if matches!(context.current_mode, Mode::PrivilegedMode) {
                if args.len() == 1 && args[0] == "all" {
                    println!("All possible debugging has been turned off");
                    Ok(())
                } else {
                    Err("Invalid arguments provided to 'undebug all'. This command does not accept additional arguments.".into())
                }
            } else {
                Err("The 'undebug all' command is only available in Privileged EXEC mode.".into())
            }
        },
    });

    commands.insert("hostname", Command {
        name: "hostname",
        description: "Set the device hostname",
        suggestions: None,
        suggestions1: None,
        suggestions2: None,
        options: Some(vec!["<new-hostname>    - Enter a new hostname"]),
        execute: |args, context, _| {
            if let Mode::ConfigMode = context.current_mode {
                if let Some(new_hostname) = args.get(0) {
                    
                    context.config.hostname = new_hostname.to_string();
    
                    match context.current_mode {
                        Mode::ConfigMode => {
                            context.prompt = format!("{}(config)#", new_hostname);
                        }
                        Mode::PrivilegedMode => {
                            context.prompt = format!("{}#", new_hostname);
                        }
                        _ => {
                            context.prompt = format!("{}>", new_hostname);
                        }
                    }
    
                    println!("Hostname changed to '{}'", new_hostname);
                    Ok(())
                } else {
                    Err("Please specify a new hostname. Usage: hostname <new_hostname>".into())
                }
            } else {
                Err("The 'hostname' command is only available in Global Configuration Mode.".into())
            }
        },
    });

    commands.insert(
        "ifconfig",
        Command {
            name: "ifconfig",
            description: "Configure a network interface",
            suggestions: None,
            suggestions1: None,
            suggestions2: None,
            options: Some(vec![
                "<interface>         - Network interface name"
            ]),
            execute: |args, _, _| {
                // Display all interfaces if no arguments
                if args.is_empty() {
                    println!("System Network Interfaces:");
                    println!("-------------------------");
                    println!("{}", get_system_interfaces(None));
                    
                    return Ok(());
                } else {
                    // Get details for the specified interface
                    let interface_name = &args[0];
                    println!("Interface: {}", interface_name);
                    println!("-------------------------");
                    
                    // Get the specified interface details
                    let interface_details = get_system_interfaces(Some(interface_name));
                    
                    if interface_details.is_empty() {
                        println!("Interface '{}' not found.", interface_name);
                    } else {
                        println!("{}", interface_details);
                    }
                }
    
                Ok(())
            },
        },
    );

    commands.insert(
        "show",
        Command {
            name: "show",
            description: "Display all the show commands when specific command is passed in the specific mode",
            suggestions: Some(vec![
                "running-config",
                "startup-config",
                "version",
                "ntp",
                "processes",
                "clock",
                "uptime",
                "controllers",
                "history",
                "sessions",
                "interfaces",
                "ip",
                "login"
            ]),
            suggestions1: Some(vec![
                "running-config",
                "startup-config",
                "version",
                "ntp",
                "processes",
                "clock",
                "uptime",
                "controllers",
                "history",
                "sessions",
                "interfaces",
                "ip",
                "login"
            ]),
            suggestions2: None,
            options: None,
            execute: |args, context, clock| {
                if matches!(context.current_mode, Mode::UserMode | Mode ::PrivilegedMode){
                    return match args.get(0) {
                        Some(&"clock") => {
                            show_clock(clock);
                            Ok(())
                        },
                        Some(&"uptime") => {
                            show_uptime(clock);
                            Ok(())
                        },
                        Some(&"version") => {
                            show_version();
                            Ok(())
                        },
                        
                        Some(&"sessions") if matches!(context.current_mode, Mode::UserMode) => {
                            show_sessions();
                            Ok(())
                        },

                        Some(&"controllers") if matches!(context.current_mode, Mode::UserMode) => {
                            
                            show_controllers();  
                            Ok(())
                        },
                        Some(&"history") if matches!(context.current_mode, Mode::UserMode | Mode::PrivilegedMode) => {
                            show_history();
                            Ok(())
                        },
                        
                        Some(&"running-config") if matches!(context.current_mode, Mode::PrivilegedMode) => {
                            show_run_conf(&context);
                            Ok(())
                        },

                        Some(&"startup-config") if matches!(context.current_mode, Mode::PrivilegedMode) => {
                            show_start_conf(&context);
                            Ok(())
                        },

                        Some(&"interfaces") if matches!(context.current_mode, Mode::PrivilegedMode) => {
                            show_interfaces();
                            Ok(())
                        },

                        Some(&"ip") => {
                            match args.get(1) {
                                Some(&"interface") => {
                                    match args.get(2) {
                                        Some(&"brief") => {
                                            show_ip_int_br();
                                            Ok(())
                                        },
                                        _ => Err("Invalid interface subcommand. Use 'brief'".into())
                                    }
                                }
                                Some(&"route") => {
                                    show_ip_route();
                                    Ok(())
                                }
                                _ => Err("Invalid IP subcommand. Use 'interface brief'".into())
                            }
                        },

                        Some(&"login") if matches!(context.current_mode, Mode::PrivilegedMode) => {
                            show_login();
                            Ok(())
                        },
                        
                        Some(&"ntp") if matches!(context.current_mode, Mode::PrivilegedMode) => {
                            match args.get(1) {
                                Some(&"associations") => {
                                    show_ntp_asso(&context);
                                    Ok(())
                                },
                                None => {
                                    show_ntp(&context);
                                    Ok(())
                                },
                                _ => Err("Invalid NTP subcommand. Use 'associations' or no subcommand".into())
                            }
                        },
                        
                        Some(&"processes") if matches!(context.current_mode, Mode::PrivilegedMode) => {
                            show_proc();
                            Ok(())
                            
                            
                        },
                        
                        Some(cmd) => {
                            println!("Invalid show command: {}", cmd);
                            Ok(())
                        },

                        None => {
                            println!("Missing parameter. Usage: show <command>");
                            Ok(())
                        }
                    }

                }
                else {
                    return Err("Show commands are only available in User EXEC mode and Privileged EXEC mode.".into());
                }
            },
        },
    );

    commands.insert(
        "do",
        Command {
            name: "do",
            description: "Execute privileged EXEC commands from any configuration mode",
            suggestions: Some(vec!["show"]),
            suggestions1: Some(vec!["show"]),
            suggestions2: Some(vec![
                "running-config",
                "startup-config",
                "version",
                "ntp",
                "processes",
                "clock",
                "uptime",
                "controllers",
                "history",
                "sessions",
                "ip",
                "interfaces",
                "login"
            ]),
            options: None,
            execute: |args, context, clock| {
                // Check if the first argument is "show"
                match args.get(0) {
                    Some(&"show") => {
                        let show_args: Vec<&str> = args.iter().skip(1).copied().collect();
                        
                        match show_args.get(0) {
                            Some(&"clock") => {
                                show_clock(clock);
                                Ok(())
                            },
                            Some(&"uptime") => {
                                show_uptime(clock);
                                Ok(())
                            },
                            Some(&"version") => {
                                show_version();
                                Ok(())
                            },
                            Some(&"sessions") => {
                                show_sessions();
                                Ok(())
                            },
                            Some(&"controllers") => {
                                show_controllers();                                
                                Ok(())
                            },
                            Some(&"history") => {
                                show_history();
                                Ok(())
                            },
                            Some(&"running-config") => {
                                show_run_conf(&context);
                                Ok(())
                            },
                            Some(&"startup-config") => {
                                show_start_conf(&context);
                                Ok(())
                            },
                            Some(&"interfaces") => {
                                show_interfaces();
                                Ok(())
                            },
                            Some(&"ip") => {
                                match args.get(2) {
                                    Some(&"interface") => {
                                        match args.get(3) {
                                            Some(&"brief") => {
                                                show_ip_int_br();
                                                Ok(())
                                            },
                                            _ => Err("Invalid interface subcommand. Use 'brief'".into())
                                        }
                                    }
                                    Some(&"route") => {
                                        show_ip_route();
                                        Ok(())
                                    }
                                    _ => Err("Invalid IP subcommand. Use 'interface brief'".into())
                                }
                            },
                            Some(&"login") => {
                                show_login();
                                Ok(())
                            },
                            Some(&"ntp") => {
                                match show_args.get(1) {
                                    Some(&"associations") => {
                                        show_ntp_asso(&context);
                                        Ok(())
                                    },
                                    None => {
                                        show_ntp(&context);
                                        Ok(())
                                    },
                                    _ => Err("Invalid NTP subcommand. Use 'associations' or no subcommand".into())
                                }
                            },
                            Some(&"processes") => {
                                show_proc();
                                Ok(())
                            },
                            Some(cmd) => {
                                println!("Invalid show command: {}", cmd);
                                Ok(())
                            },
                            None => {
                                println!("Missing parameter. Usage: do show <command>");
                                Ok(())
                            }
                        }
                    },
                    Some(cmd) => {
                        println!("Invalid do command: {}", cmd);
                        Ok(())
                    },
                    None => {
                        println!("Missing parameter. Usage: do <command>");
                        Ok(())
                    }
                }
            },
        },
    );
    
    commands.insert(
        "write",
        Command {
            name: "write memory",
            description: "Save the running configuration to the startup configuration",
            suggestions: Some(vec!["memory"]),
            suggestions1: Some(vec!["memory"]),
            suggestions2: None,
            options: None,
            execute: |args, context, _| {
                if matches!(context.current_mode, Mode::UserMode | Mode::PrivilegedMode | Mode::ConfigMode) {
                    if args.len() == 1 && args[0] == "memory" {
                        save_running_to_startup(context);
                        Ok(())
                    
                    } else {
                        Err("Invalid arguments provided to 'write memory'. This command does not accept additional arguments.".into())
                    }
                } else {
                    Err("The 'write memory' command is only available in Privileged EXEC mode.".into())
                }
            },
        },
    );
    

    commands.insert(
        "copy",
        Command {
            name: "copy",
            description: "Copy running configuration",
            suggestions: Some(vec!["running-config"]),
            suggestions1: Some(vec!["running-config"]),
            suggestions2: Some(vec!["startup-config"]),
            options: Some(vec!["<file_name>     - Enter the file name or 'startup-config'",
            "startup-config"]),
            execute: |args, context, _| {
                if !matches!(context.current_mode, Mode::PrivilegedMode | Mode::ConfigMode) {
                    return Err("The 'copy' command is only available in Privileged EXEC mode, Config mode and interface mode".into());
                }

                else if args[1] == "startup-config"{
                    
                    save_running_to_startup(context);
                    Ok(())
                    
                }

                else if args[1] != "startup-config"{
                    let file_name = args[1];
                    let running_config = get_running_config(context); 
                    let file_path = Path::new(file_name);
                    
                    match File::create(file_path) {
                        Ok(mut file) => {
                            if let Err(err) = file.write_all(running_config.as_bytes()) {
                                eprintln!("Error writing to the file: {}", err);
                                return Err(err.to_string());
                            }
                            println!("Running configuration copied to {}", file_name);
                            Ok(())
                        }
                        Err(err) => {
                            eprintln!("Error creating the file: {}", err);
                            Err(err.to_string())
                        }
                    }
                } else {
                    println!("the command should be 'copy running-config startup-config|<file-name>'.");
                    Ok(())
                }
            },
        },
    );

    commands.insert(
        "help",
        Command {
            name: "help",
            description: "Display available commands for current mode",
            suggestions: None,
            suggestions1: None,
            suggestions2: None,
            options: None,
            execute: |_args, context, _| {
                help_command(&context);
                Ok(())
            }
        },
    );
    

    commands.insert(
        "clock",
        Command {
            name: "clock set",
            description: "Change the clock date and time",
            suggestions: Some(vec!["set"]),
            suggestions1: Some(vec!["set"]),
            suggestions2: None,
            options: Some(vec!["<hh:mm:ss>      - Enter the time in this specified format",
                "<day>      - Enter the day '1-31'",
                "<month>    - Enter a valid month",
                "<year>     - Enter the year"]),
            execute: |args, context, clock| {
                if matches!(context.current_mode, Mode::PrivilegedMode) {
                    if args.len() > 1 && args[0] == "set" {   
                        if let Some(clock) = clock {

                            let input = args.join(" ");
            
                            match parse_clock_set_input(&input) {
                                Ok((time, day, month, year)) => {
                        
                                    handle_clock_set(time, day, month, year, clock);
                                    Ok(())
                                }
                                Err(err) => Err(err), 
                            }
                        } else {
                            Err("Clock functionality is unavailable.".to_string())
                        }
                    } else {
                        Err("Correct Usage of 'clock set' command is 'clock set <hh:mm:ss> <day> <month> <year>'.".into())
                    }
                }
                else {
                    Err("The 'clock set' command is only available in Privileged EXEC mode.".into())
                }
            },
        },
    );
    
    
    commands.insert(
        "ip",
        Command {
            name: "ip",
            description: "Define all the ip commands",
            suggestions: Some(vec!["address", "route"]),
            suggestions1: Some(vec!["address", "route"]),
            suggestions2: None,
            options: Some(vec![
                "<IP_Address>   - Enter the IP Address",
                "<subnetmask>   - Enter the subnet mask"
            ]),
            execute: |args, context, _| {
                if args.is_empty() {
                    return Err("Incomplete command. The command should be 'ip address <IP address> <subnet_mask>".into());
                }
                else if args[0] == "address"{
                    if args.len() == 1 {
                        println!("Interface detils");
                    
                        execute_spawn_process("ip", &["a"]);
                        return Ok(());
                    } else if args.len() == 3 {
                        if matches!(context.current_mode, Mode::InterfaceMode) {
                            let ip_address = &args[1];
                            let subnet_mask = &args[2];
                            let selected_interface = SELECTED_INTERFACE.lock().unwrap();
                            if let Some(ref interface) = *selected_interface {
                                match ip_with_cidr(ip_address, subnet_mask) {
                                    Ok(result) => {
                                        execute_spawn_process("sudo", &["ifconfig", interface, ip_address, "netmask", subnet_mask, "up"])?;
                                        
                                        // Create Netplan configuration content
                                        let netplan_content = format!(
                                            "network:\n  ethernets:\n    {}:\n      dhcp4: no\n      addresses:\n        - {}\n      nameservers:\n        addresses: [8.8.8.8, 8.8.4.4]",
                                            interface, &result
                                        );
                                        
                                        // Write to Netplan config file
                                        let netplan_cmd = format!("echo '{}' | sudo tee /etc/netplan/*.yaml", netplan_content);
                                        execute_spawn_process("sh", &["-c", &netplan_cmd])?;

                                        println!("IP address {} is configured to the interface {}", &result, interface);
                                        return Ok(());
                                    }, 
                                    Err(e) => return Err(format!("Failed to configure the IP address: {}", e))
                                }
                            } else {
                                return Err("No interface selected. Use the 'interface' command first.".into());
                            }
                        } else{
                            return Err("The 'ip address' command is only available in Interface Configuration mode.".into());
                        }
                    } else {
                        return Err("Invalid command format. Use: 'ip address <IP address> <subnet_mask>'".into());
                    }   
                } else if args[0] == "route" {
                    if matches!(context.current_mode, Mode::ConfigMode) {
                        if args.len() < 5 {
                            return Err("Usage: ip route <ip_address> <netmask> <exit_interface> <next_hop>".into());
                        }

                        let ip_addr = &args[1];
                        let netmask = &args[2];
                        let exit_int = &args[3];
                        let nxt_hop = &args[4];

                        let cidr_notation = match ip_with_cidr(ip_addr, netmask) {
                            Ok(result) => result,
                            Err(e) => return Err(format!("Failed to restructure the IP address: {}", e))
                        };

                        let (interface_list, interfaces_list) = match get_available_int() {
                            Ok(result) => result,
                            Err(e) => return Err(e),
                        };

                        if !interface_list.iter().any(|i| i == exit_int) {
                            return Err(format!("Invalid exit interface: {}. Available interfaces: {}", exit_int, interfaces_list));
                        }
                        
                        println!("Adding route to {} via {} on interface {}", &cidr_notation, nxt_hop, exit_int);

                        match execute_spawn_process("sudo", &["ip", "route", "add", &cidr_notation, "via", nxt_hop, "dev", exit_int]) {
                            Ok(_) => {
                                println!("Route added successfully");
                                return Ok(());
                            },
                            Err(e) => Err::<(), String>(format!("Failed to add route: {}", e).to_string()),
                        }
                    } else{
                        Err("The 'ip route' command is only available in Global Configuration mode.".into())
                    };
                } else {
                    return Err("Invalid command format. Use: 'ip address <IP address> <subnet_mask>' or to check interface detials use 'ip a' command".into());
                }
                return Ok(());
            },
        }
    );

    commands.insert(
        "shutdown",
        Command {
            name: "shutdown",
            description: "Disable the selected network interface.",
            suggestions: None,
            suggestions1: None,
            suggestions2: None,
            options: None,
            execute: |_, context, _| {
                if matches!(context.current_mode, Mode::InterfaceMode) {
                    let selected_interface = SELECTED_INTERFACE.lock().unwrap();
                    if let Some(ref interface) = *selected_interface {
                        execute_spawn_process("sudo", &["ip", "link", "set", interface, "down"])?;
                        println!("interface {} is set to down", interface);
                        Ok(())
                    } else {
                        Err("No interface selected. Use the 'interface' command first.".into())
                    }
                } else {
                    Err("The 'shutdown' command is only available in Interface Configuration mode.".into())
                }
            },
        },
    );
    
    commands.insert(
        "no",
        Command {
            name: "no",
            description: "Enable the selected network interface.",
            suggestions: Some(vec!["shutdown", "ntp", "ip"]),
            suggestions1: Some(vec!["shutdown", "ntp", "ip"]),
            suggestions2: Some(vec!["server", "route"]),
            options: None,
            execute: |args, context, _| {
                if args.len() == 1 && args[0] == "shutdown" {
                    if matches!(context.current_mode, Mode::InterfaceMode) {
                        let selected_interface = SELECTED_INTERFACE.lock().unwrap();
                        if let Some(ref interface) = *selected_interface {
                            execute_spawn_process("sudo", &["ip", "link", "set", interface, "up"])?;
                            execute_spawn_process("sudo", &["netplan", "apply"])?;
                            println!("interface {} is set to up", interface);
                            Ok(())
                        } else {
                            Err("No interface selected. Use the 'interface' command first.".into())
                        }
                    } else {
                        Err("The 'shutdown' command is only available in Interface Configuration mode.".into())
                    }
                } else if args.len() == 3 && args[0] == "ntp" && args[1] == "server" {
                    if matches!(context.current_mode, Mode::ConfigMode) {
                        let ip_address = args[2].to_string();
                        if context.ntp_servers.remove(&ip_address) {
                            // Remove from the associations list as well
                            context.ntp_associations.retain(|assoc| assoc.address != ip_address);
                            println!("NTP server {} removed.", ip_address);
                            Ok(())
                        } else {
                            Err("NTP server not found.".into())
                        }
                    } else {
                        Err("The 'no ntp server' command is only available in configuration mode.".into())
                    }
                } else if args[0] == "ip" && args[1] == "route"{
                    if matches!(context.current_mode, Mode::ConfigMode) {

                        if args.len() < 6 {
                            return Err("Usage: ip route <ip_address> <netmask> <exit_interface> <next_hop>".into());
                        }

                        let ip_addr = &args[2];
                        let netmask = &args[3];
                        let exit_int = &args[4];
                        let nxt_hop = &args[5];

                        let cidr_notation = match ip_with_cidr(ip_addr, netmask) {
                            Ok(result) => result,
                            Err(e) => return Err(format!("Failed to restructure the IP address: {}", e))
                        };

                        let (interface_list, interfaces_list) = match get_available_int() {
                            Ok(result) => result,
                            Err(e) => return Err(e),
                        };

                        if !interface_list.iter().any(|i| i == exit_int) {
                            return Err(format!("Invalid exit interface: {}. Available interfaces: {}", exit_int, interfaces_list));
                        }
                        
                        println!("Deleting route to {} via {} on interface {}", &cidr_notation, nxt_hop, exit_int);

                        match execute_spawn_process("sudo", &["ip", "route", "del", &cidr_notation, "via", nxt_hop, "dev", exit_int]) {
                            Ok(_) => {
                                println!("Route deleted successfully");
                                return Ok(());
                            },
                            Err(e) => Err::<(), String>(format!("Failed to delete route: {}", e).to_string()),
                        }
                    } else {
                        Err("The 'no ip route' command is only available in configuration mode.".into())
                    }
                } else if args[0] == "ip" && args[1] == "address"{
                    if matches!(context.current_mode, Mode::InterfaceMode) {
                        let ip_address = &args[2];
                        let subnet_mask = &args[3];
                        let selected_interface = SELECTED_INTERFACE.lock().unwrap();
                        if let Some(ref interface) = *selected_interface {
                            match ip_with_cidr(ip_address, subnet_mask) {
                                Ok(result) => {
                                    // Fixed: Use Ok() and ? to handle the Result returned by execute_spawn_process
                                    //println!("IP_address = {}, Interface = {}", &result, interface);
                                    execute_spawn_process("sudo", &["ip", "addr", "del", &result, "dev", interface])?;
                                    println!("IP address {} is removed from the interface {}", &result, interface);
                                    return Ok(());
                                }, 
                                Err(e) => return Err(format!("Failed to remove the IP address: {}", e))
                            }
                        } else {
                            return Err("No interface selected. Use the 'interface' command first.".into());
                        }
                    } else {
                        Err("The 'no ip address' command is only available in Interface Configuration mode.".into())
                    }
                    
                } 
                
                else {
                    Err("Invalid arguments provided to 'no'.".into())
                }
                
            },
        },
    );

    commands.insert("clear", Command {
        name: "clear",
        description: "Clear processes",
        suggestions: Some(vec!["ntp associations"]),
        suggestions1: None,
        suggestions2: None,
        options: None,
        execute: |args, context, _| {
            match args.get(0) {
                None => {
                    ProcessCommand::new("clear")
                        .status()
                        .unwrap();
                    Ok(())
                },
                Some(&"ntp") => {
                    if !matches!(context.current_mode, Mode::PrivilegedMode) {
                        return Err("The 'clear ntp associations' command is only available in privileged EXEC mode.".into());
                    }
    
                    match args.get(1) {
                        Some(&"associations") => {
                            context.ntp_associations.clear();
                            // Reinitialize associations for configured servers
                            println!("NTP associations cleared and reinitialized.");
                            Ok(())
                        },
                        _ => Err("Invalid command. Usage: clear ntp associations".into())
                    }
                },
                _ => Err("Invalid command. Available commands: clear, clear ntp associations".into())
            }
        },
    });

    
    commands.insert("ntp", Command {
        name: "ntp",
        description: "NTP configuration commands",
        suggestions: Some(vec!["source", "server", "master", "authenticate", "authentication-key", "trusted-key"]),
        suggestions1: Some(vec!["source", "server", "master", "authenticate", "authentication-key", "trusted-key"]),
        suggestions2: None,
        options: None,
        execute: |args, context, _| {
            if !matches!(context.current_mode, Mode::ConfigMode) {
                return Err("NTP commands are only available in configuration mode.".into());
            }
    
            if args.is_empty() {
                return Err("Subcommand required. Available subcommands: server, master, authenticate, authentication-key, trusted-key".into());
            }
    
            match &args[0][..] {
                "server" => {
                    if args.len() == 2 {
                        let ip_address = args[1].to_string();
                        if ip_address.parse::<Ipv4Addr>().is_ok() {
                            context.ntp_servers.insert(ip_address.clone());
                            // Assuming once the server is configured, we add it to NTP associations
                            let association = NtpAssociation {
                                address: ip_address.clone(),
                                ref_clock: ".INIT.".to_string(),
                                st: 16,
                                when: "-".to_string(),
                                poll: 64,
                                reach: 0,
                                delay: 0.0,
                                offset: 0.0,
                                disp: 0.01,
                            };
                            context.ntp_associations.push(association); 
                            println!("NTP server {} configured.", ip_address);
                            Ok(())
                        } else {
                            Err("Invalid IP address format.".into())
                        }
                    } else {
                        Err("Invalid arguments. Usage: ntp server {ip-address}".into())
                    }
                },
                "source" => {
                    if args.len() >= 2 {
                        if args.len() == 2 {
                            let interface_name = args[1].to_string();
                            context.ntp_source_interface = Some(interface_name.clone());
                            println!("NTP source interface set to {}", interface_name);
                            Ok(())
                        } else {
                            Err("Invalid arguments. Usage: ntp source interface {interface-name}".into())
                        }
                    } else {
                        Err("Invalid arguments. Usage: ntp source interface {interface-name}".into())
                    }
                },
                "master" => {
                    context.ntp_master = true;
                    println!("Device configured as NTP master.");
                    Ok(())
                },
                "authenticate" => {
                    if args.len() == 1 {
                        context.ntp_authentication_enabled = !context.ntp_authentication_enabled;
                        let status = if context.ntp_authentication_enabled {
                            "enabled"
                        } else {
                            "disabled"
                        };
                        println!("NTP authentication {}", status);
                        Ok(())
                    } else {
                        Err("Invalid arguments. Use 'ntp authenticate'.".into())
                    }
                },
                "authentication-key" => {
                    if args.len() == 4 && args[2] == "md5" {
                        if let Ok(key_number) = args[1].parse::<u32>() {
                            let md5_key = args[3].to_string();
                            context.ntp_authentication_keys.insert(key_number, md5_key.clone());
                            println!("NTP authentication key {} configured with MD5 key: {}", key_number, md5_key);
                            Ok(())
                        } else {
                            Err("Invalid key number. Must be a positive integer.".into())
                        }
                    } else {
                        Err("Invalid arguments. Use 'ntp authentication-key <key-number> md5 <key-value>'.".into())
                    }
                },
                "trusted-key" => {
                    if args.len() == 2 {
                        if let Ok(key_number) = args[1].parse::<u32>() {
                            context.ntp_trusted_keys.insert(key_number);
                            println!("NTP trusted key {} configured.", key_number);
                            Ok(())
                        } else {
                            Err("Invalid key number. Must be a positive integer.".into())
                        }
                    } else {
                        Err("Invalid arguments. Use 'ntp trusted-key <key-number>'.".into())
                    }
                },
                _ => Err("Invalid NTP subcommand. Available subcommands: server, master, authenticate, authentication-key, trusted-key".into())
            }
        }
    });
  
    commands.insert("service", Command {
        name: "service password-encryption",
        description: "Enable password encryption",
        suggestions: Some(vec!["password-encryption"]),
        suggestions1: Some(vec!["password-encryption"]),
        suggestions2: None,
        options: None,
        execute: |args, context, _| {
            if matches!(context.current_mode, Mode::ConfigMode) {
                if args.len() == 1 && args[0] == "password-encryption" {
                    let storage = PASSWORD_STORAGE.lock().unwrap();
                    
                    let stored_password = storage.enable_password.clone();
                    let stored_secret = storage.enable_secret.clone();
                    drop(storage);
                    
                    if let Some(password) = stored_password {
                        let encrypted_password = encrypt_password(&password);
                        context.config.encrypted_password = Some(encrypted_password);
                    }
                    
                    if let Some(secret) = stored_secret {
                        let encrypted_secret = encrypt_password(&secret);
                        context.config.encrypted_secret = Some(encrypted_secret);  // Update encrypted secret
                    }
        
                    context.config.password_encryption = true;
                    println!("Password encryption enabled.");
                    Ok(())
                } else {
                    Err("Invalid arguments provided to 'service password-encryption'. This command does not accept additional arguments.".into())
                }
            } else {
                Err("The 'service password-encryption' command is only available in Privileged EXEC mode.".into())
            }
        },
    });

    commands.insert(
        "ssh",
        Command {
            name: "ssh",
            description: "Establish SSH connection to a remote host",
            suggestions: Some(vec![
                "-v",
                "-l",
                "-h",
                "--help"
            ]),
            suggestions1: Some(vec![
                "-v",
                "-l",
                "-h",
                "--help"
            ]),
            suggestions2: None,
            options: Some(vec![
                "<login-name>       - Enter the hostname",
                "<ip_address>       - Enter the host IP address"
            ]),
            execute: |args, context, _| {
                if matches!(context.current_mode, Mode::PrivilegedMode) {
                    match args.get(0) {
                        Some(&"-v") => {
                            println!("OpenSSH_8.9p1 Ubuntu-3ubuntu0.1, OpenSSL 3.0.2 15 Mar 2022");
                            Ok(())
                        },
                        Some(&"-l") => {
                            if args.len() < 2 {
                                println!("Usage: ssh -l <username>@<ip-address>");
                                return Ok(());
                            }
    
                            let connection_string = args[1];
                            
                            // Split the connection string into username and ip
                            match connection_string.split_once('@') {
                                Some((username, ip)) => {
                                    connect_via_ssh(username, ip)?; 
                                    println!("Connected successfully!");
                                    Ok(())
                                },
                                None => {
                                    println!("Invalid format. Use: ssh -l username@ip-address");
                                    println!("Example: ssh -l admin@192.168.1.1");
                                    Ok(())
                                }
                            }
                        },
                        Some(&help) if help == "-h" || help == "--help" => {
                            println!("SSH Command Usage:");
                            println!("  ssh -v                     Display SSH version");
                            println!("  ssh -l username@ip-address Login to remote server");
                            println!("\nExamples:");
                            println!("  ssh -l admin@192.168.1.1");
                            Ok(())
                        },
                        Some(cmd) => {
                            println!("Invalid SSH option: {}", cmd);
                            println!("Use 'ssh -h' for help");
                            Ok(())
                        },
                        None => {
                            println!("Missing parameters. Use 'ssh -h' for help");
                            Ok(())
                        }
                    }
                } else {
                    Err("SSH command is only available in User EXEC mode and Privileged EXEC mode.".to_string())
                }
            },
        }
    );

    commands.insert("ping", Command {
        name: "ping",
        description: "Ping a specific IP address to check reachability",
        suggestions: None,
        suggestions1: None,
        suggestions2: None,
        options: Some(vec!["<ip-address>    - Enter the ip-address"]),
        execute: |args, _context, _| {
            if args.len() == 1 {
                let ip = args[0].to_string();
                
                println!("Pinging {} with 32 bytes of data:", ip);
                
                execute_spawn_process("ping", &["-c", "4", "-s", "32", &ip]);
                Ok(())

            } else {
                Err("Invalid syntax. Usage: ping <ip>".into())
            }
        },
    });
    
    commands.insert("traceroute", Command {
        name: "traceroute",
        description: "Trace the route to a specific IP address or hostname",
        suggestions: None,
        suggestions1: None,
        suggestions2: None,
        options: Some(vec!["<ip-address/hostname>    - Enter the IP address or hostname"]),
        execute: |args, _context, _| {
            if args.len() == 1 {
                let target = args[0].to_string();
    
                println!("Tracing route to {} over a maximum of 30 hops", target);
    
                execute_spawn_process("traceroute", &["-n", "-m", "30", &target]);
                println!("Trace Completed.");
                Ok(())

            } else {
                Err("Invalid syntax. Usage: traceroute <ip/hostname>".into())
            }
        },
    });


    commands
}

