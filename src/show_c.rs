use crate::clock_settings::{Clock, handle_show_clock, handle_show_uptime};
use crate::cliconfig::CliContext;
use crate::network_config::{read_lines, IP_ADDRESS_STATE, STATUS_MAP, execute_spawn_process};
use crate::run_config::{get_running_config};

pub fn show_clock(clock: &mut Option<Clock>) -> String {
    if let Some(clock) = clock {
        handle_show_clock(clock);
        "Clock displayed successfully.".to_string() 
    } else {
        "Clock functionality is unavailable.".to_string() 
    }
}

pub fn show_uptime(clock: &mut Option<Clock>) -> String {
    if let Some(clock) = clock {
        handle_show_uptime(clock);
        "System Uptime displayed successfully.".to_string()
    } else {
        "Clock functionality is unavailable.".to_string()
    }
}

pub fn show_version() {
    //Acess a version file and show
    println!("Cisco IOS Software, C2900 Software (C2900-UNIVERSALK9-M), Version 15.1(4)M4, RELEASE SOFTWARE (fc2)");
}

pub fn show_sessions() -> Result<(), String> {
    //Use 'w' command to access the system Telnet sessions
    execute_spawn_process("sudo", &["w"]);
    Ok(())
}

pub fn show_controllers() -> Result<(), String> {
    
    //Triggers the command ‘lspci’ or ‘sudo lshw -class network’ and extract the relevant details.
    execute_spawn_process("sudo", &["lshw", "-class", "network"]);
    Ok(())
}


pub fn show_history() -> Result<(), String>{
    // Read from history.txt file
                            
    match read_lines("history.txt") {
        Ok(lines) => {
            for line in lines.flatten() {
                println!("{}", line);
            }
            Ok(())
        },
        Err(e) => Err(format!("Error reading history file: {}", e).into())
    }
}

pub fn show_run_conf(context: &CliContext) -> Result<(), String>{
    println!("Building configuration...\n");
    println!("Current configuration : 0 bytes\n");
    let running_config = get_running_config(&context);
    println!("{}", running_config);
    Ok(())
}

pub fn show_start_conf(context: &CliContext) -> Result<(), String> {
    println!("Reading startup configuration file...\n");
    
    let config_path = "startup-config.conf";
    
    match std::fs::read_to_string(config_path) {
        Ok(contents) => {
            if let Some(last_written) = &context.config.last_written {
                println!("Startup configuration (last saved: {}):\n", last_written);
            } else {
                println!("Startup configuration file contents:\n");
            }
            println!("{}", contents);
        },
        Err(e) => {
            return Err(format!("Error reading startup configuration file: {}", e));
        }
    }
    
    Ok(())
}

pub fn show_interfaces() -> Result<(), String> {
    
    //Use ls /sys/class/net command
    execute_spawn_process("ls", &["/sys/class/net"]);
    Ok(())
                    
}

pub fn show_ip_int_br() -> Result<(), String> {
    execute_spawn_process("ip", &["a"]);
    Ok(())
}

pub fn show_ip_route() -> Result<(), String> {
    execute_spawn_process("ip", &["route"]);
    Ok(())
}

pub fn show_login() -> Result<(), String> {
    
    //Triggers the system ‘last’ and ‘faillog’ commands.
    execute_spawn_process("sudo", &["last"]);
    Ok(())
}

pub fn show_ntp_asso(context: &CliContext) -> Result<(), String>{
    if context.ntp_associations.is_empty() {
        println!("No NTP associations configured.");
    } else {
        println!("address         ref clock       st   when     poll    reach  delay          offset            disp");
        for assoc in &context.ntp_associations {
            println!(" ~{}       {}          {}   {}        {}      {}      {:.2}           {:.2}              {:.2}",
                assoc.address, assoc.ref_clock, assoc.st, assoc.when, assoc.poll,
                assoc.reach, assoc.delay, assoc.offset, assoc.disp);
        }
        println!(" * sys.peer, # selected, + candidate, - outlyer, x falseticker, ~ configured");
    }
    Ok(())
}

pub fn show_ntp(context: &CliContext) -> Result<(), String>{
    println!("NTP Master: {}", if context.ntp_master { "Enabled" } else { "Disabled" });
    println!("NTP Authentication: {}", if context.ntp_authentication_enabled { "Enabled" } else { "Disabled" });
    
    if !context.ntp_authentication_keys.is_empty() {
        println!("NTP Authentication Keys:");
        for (key_number, key) in &context.ntp_authentication_keys {
            println!("Key {}: {}", key_number, key);
        }
    }
    
    if !context.ntp_trusted_keys.is_empty() {
        println!("NTP Trusted Keys:");
        for key_number in &context.ntp_trusted_keys {
            println!("Trusted Key {}", key_number);
        }
    }

    Ok(())
}

pub fn show_proc() -> Result<(), String> {
    //Triggers the system commands (eg. Top, lscpu) and display the output 
    execute_spawn_process("sudo", &["lscpu"]);
    Ok(()) 
}

