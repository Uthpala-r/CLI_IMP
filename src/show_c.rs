use crate::clock_settings::{Clock, handle_show_clock, handle_show_uptime};
use crate::cliconfig::CliContext;
use crate::network_config::{read_lines, IP_ADDRESS_STATE, STATUS_MAP};
use crate::run_config::{get_running_config, default_startup_config};

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

pub fn show_sessions() {
    //Use 'w' command to access the system Telnet sessions
    println!("% No connections open");
}

pub fn show_controllers(arg1: Option<&str>, arg2: Option<&str>) -> Result<(), String> {
    
    //Triggers the command ‘lspci’ or ‘sudo lshw -class network’ and extract the relevant details.

    let interface_type = match arg1 {
        Some(t) => t,
        None => return Err("Error: Interface type required. Usage: show controllers <interface-type> <interface-number>".to_string()),
    };

    let interface_number = arg2.filter(|s| !s.is_empty()).unwrap_or("0/0");
                                
    let valid_interfaces = vec![
        "GigabitEthernet", "FastEthernet", "Ethernet", "Serial"
    ];
                                
    if !valid_interfaces.contains(&interface_type) {
        return Err(format!("Invalid interface type. Valid types are: {}", 
            valid_interfaces.join(", ")).into());
    }
    
    println!("Interface {}{}", interface_type, interface_number);
    println!("Hardware is PQUICC MPC860P ADDR: 80C95180, FASTSEND: 80011BA4");
    println!("DIST ROUTE ENABLED: 0");
    println!("Route Cache Flag: 0");

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

pub fn show_start_conf(context: &CliContext) -> Result<(), String>{
    println!("Building configuration...\n");
    if let Some(last_written) = &context.config.last_written {
        println!("Startup configuration (last saved: {}):\n", last_written);
        let startup_config = get_running_config(&context);
        println!("{}", startup_config);
    } else {
        println!("Startup configuration (default):\n");
        println!("{}", default_startup_config());
    }
    Ok(())
}

pub fn show_interfaces(context: &CliContext) -> Result<(), String> {
    let ip_address_state = IP_ADDRESS_STATE.lock().unwrap();

    //println!("DEBUG: Selected interface: {:?}", context.selected_interface);
    //println!("DEBUG: IP_ADDRESS_STATE contains {} entries.", ip_address_state.len());

    let Some(interface_name) = &context.selected_interface else {
        return Err("No interface selected. Use the 'interface' command first.".into());
    };

    if ip_address_state.is_empty() {
        println!("No interfaces found.");
        return Ok(());

    } else {
        for (interface_name, (ip_address, _)) in ip_address_state.iter() {
            println!("{} is up, line protocol is up", interface_name);
            println!("  Internet address is {}, subnet mask 255.255.255.0", ip_address);
            println!("  MTU 1500 bytes, BW 10000 Kbit, DLY 100000 usec");
            println!("  Encapsulation ARPA, loopback not set, keepalive set (10 sec)");
            println!("  Last clearing of \"show interface\" counters: never");
            println!("  Input queue: 0/2000/0/0 (size/max/drops/flushes); Total output drops: 0");
            println!("  5 minute input rate 1000 bits/sec, 10 packets/sec");
            println!("  5 minute output rate 500 bits/sec, 5 packets/sec");
            println!("  100 packets input, 1000 bytes, 10 no buffer");
            println!("  50 packets output, 500 bytes, 0 underruns");
            
        }
    }
    Ok(())
                    
}

pub fn show_ip_int_br() -> Result<(), String> {
    let ip_address_state = IP_ADDRESS_STATE.lock().unwrap();
    let status_map = STATUS_MAP.lock().unwrap();

    println!(
        "{:<22} {:<15} {:<8} {:<20} {:<20} {:<10}",
        "Interface", "IP-Address", "OK?", "Method", "Status", "Protocol"
    );

    for (interface_name, (ip_address, _)) in ip_address_state.iter() {
        let is_up = status_map.get(interface_name).copied().unwrap_or(false);
        let status = if is_up {
            "up"
        } else {
            "administratively down"
        };
        let protocol = if is_up {
            "up"
        } else {
            "down"
        };

        println!(
            "{:<22} {:<15} YES     unset/manual        {}         {}",
            interface_name, ip_address, status, protocol
        );
    }
    Ok(())
}

pub fn show_login() {
    
    //Triggers the system ‘last’ and ‘faillog’ commands.
    
    println!("A default login delay of 1 seconds is applied.");
    println!("No Quiet-Mode access list has been configured.");
    println!(" ");
    println!("Router NOT enabled to watch for login Attacks");
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

pub fn show_proc(){
    //Triggers the system commands (eg. Top, lscpu) and display the output 
    println!("CPU utilization for five seconds: 0%/0%; one minute: 0%; five minutes: 0%");
    println!(
        " PID Q  Ty       PC  Runtime(uS)    Invoked   uSecs    Stacks TTY Process\n\
        1 C  sp 602F3AF0            0       1627       0 2600/3000   0 Load Meter\n\
        2 L  we 60C5BE00            4        136      29 5572/6000   0 CEF Scanner\n\
        3 L  st 602D90F8         1676        837    2002 5740/6000   0 Check heaps\n\
        4 C  we 602D08F8            0          1       0 5568/6000   0 Chunk Manager\n\
        5 C  we 602DF0E8            0          1       0 5592/6000   0 Pool Manager"
    ); 
}

pub fn show_proc_cpu(){
    //Triggers the system commands (eg. Top, lscpu) and display the output 
    println!("CPU utilization for five seconds: 8%/4%; one minute: 6%; five minutes: 5%");
    println!(
        " PID Runtime(uS)   Invoked  uSecs    5Sec   1Min   5Min TTY Process\n\
        1         384     32789     11   0.00%  0.00%  0.00%   0 Load Meter\n\
        2        2752      1179   2334   0.73%  1.06%  0.29%   0 Exec\n\
        3      318592      5273  60419   0.00%  0.15%  0.17%   0 Check heaps\n\
        4           4         1   4000   0.00%  0.00%  0.00%   0 Pool Manager\n\
        5        6472      6568    985   0.00%  0.00%  0.00%   0 ARP Input"
    );
}

pub fn show_proc_cpu_his(){
    //Triggers the system commands (eg. Top, lscpu) and display the output 
    println!(
        "CPU% per minute (last 60 minutes)\n\
        100\n 90\n 80         *  *                     * *     *  * *  *\n\
        70  * * ***** *  ** ***** ***  **** ******  *  *******     * *\n\
        60  #***##*##*#***#####*#*###*****#*###*#*#*##*#*##*#*##*****#\n\
        50  ##########################################################\n\
        40  ##########################################################\n\
        30  ##########################################################\n\
        20  ##########################################################\n\
        10  ##########################################################\n\
            0....5....1....1....2....2....3....3....4....4....5....5....\n\
                    0    5    0    5    0    5    0    5    0    5"
    );
}

pub fn show_proc_mem(){
    //Triggers the system commands (eg. Top, lscpu) and display the output 
    println!(
        "Total: 106206400, Used: 7479116, Free: 98727284\n\
        PID TTY  Allocated      Freed    Holding    Getbufs    Retbufs Process\n\
        0   0      81648       1808    6577644          0          0 *Init*\n\
        0   0        572     123196        572          0          0 *Sched*\n\
        0   0   10750692    3442000       5812    2813524          0 *Dead*\n\
        1   0        276        276       3804          0          0 Load Meter"
    );
}