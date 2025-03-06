/// External crates for the CLI application
use crate::cliconfig::{CliConfig, CliContext};
use crate::execute::Mode;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
//use crate::network_config::{STATUS_MAP, IP_ADDRESS_STATE, ROUTE_TABLE, OSPF_CONFIG, ACL_STORE};


/// Saves the given `CliConfig` to a file named `startup-config.json`.
/// 
/// This function serializes the provided configuration into JSON format and writes it
/// to a file. If the file already exists, it will be overwritten. If the file does
/// not exist, it will be created. The JSON is formatted for readability (pretty-printed).
/// 
/// # Parameters
/// - `config`: The `CliConfig` object that contains the configuration to be saved.
/// 
/// # Returns
/// This function returns a `Result<(), std::io::Error>`. It will return `Ok(())` if the
/// file is successfully written, or an error if something goes wrong (e.g., file write failure).
/// 
/// # Example
/// ```
/// use crate::cliconfig::CliConfig;
/// let config = CliConfig::default(); // Example config
/// if let Err(e) = save_config(&config) {
///     eprintln!("Failed to save config: {}", e);
/// }
/// ```
pub fn save_config(config: &CliConfig) -> std::io::Result<()> {
    let serialized = serde_json::to_string_pretty(config)?;
    let mut file = OpenOptions::new()
        .create(true) 
        .write(true)  
        .truncate(true) 
        .open("startup-config.json")?;
    file.write_all(serialized.as_bytes())
}


/// Loads the configuration from the `startup-config.json` file.
/// 
/// This function attempts to read the `startup-config.json` file and deserialize its
/// contents into a `CliConfig` object. If the file cannot be opened, read, or parsed,
/// a default configuration will be returned.
/// 
/// # Returns
/// The function returns a `CliConfig` object. If loading the configuration fails, it
/// will return the default configuration as defined by `CliConfig::default()`.
/// 
/// # Example
/// ```
/// let config = load_config();
/// println!("Loaded config: {:?}", config);
/// ```
pub fn load_config() -> CliConfig {
    if let Ok(mut file) = File::open("startup-config.json") {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(config) = serde_json::from_str::<CliConfig>(&contents) {
                return config;
            }
        }
    }
    CliConfig::default()
}


/// Retrieves the current running configuration of the device.
/// 
/// The running configuration is a volatile piece of information that reflects 
/// the current state of the device, including any changes made to it. This 
/// configuration is stored in memory rather than NVRAM, meaning it will be lost 
/// when the device loses power.
/// 
/// # Returns
/// A `String` representing the current running configuration of the device.
/// 
/// # Example
/// ```rust
/// let config = get_running_config();
/// println!("Running Configuration: {}", config);
/// ``` 
pub fn get_running_config(context: &CliContext) -> String {
    let hostname = &context.config.hostname;
    let encrypted_password = context.config.encrypted_password.clone().unwrap_or_default();
    let encrypted_secret = context.config.encrypted_secret.clone().unwrap_or_default();

    
    format!(
        r#"version 15.1
no service timestamps log datetime msec
{}
!
hostname {}
!
enable password 5 {}
enable secret 5 {}
!
interface 
 ip address 
 duplex auto
 speed auto
 
!
interface Vlan1
 no ip address
 shutdown
!
ip classes

!
router ospf 
 log-adjacency-changes
 passive-interface 
 
!

!
!
end
"#,
        if context.config.password_encryption {
            "service password-encryption"
        } else {
            "no service password-encryption"
        },
        hostname,
        encrypted_password,
        encrypted_secret,
        
    )
}


/// Retrieves the startup configuration of the device.
/// 
/// The startup configuration is a non-volatile piece of information that is 
/// stored in NVRAM. This configuration persists across device reboots and 
/// represents the settings that the device will use upon startup.
/// 
/// # Returns
/// A `String` representing the startup configuration of the device.
/// 
/// # Example
/// ```rust
/// let startup_config = default_startup_config();
/// println!("Startup Configuration: {}", startup_config);
/// ```
pub fn default_startup_config() -> String {
    
    let startup_config = (
        
        r#"
Building configuration...

Current configuration : 0 bytes

version 15.1
no service timestamps log datetime msec
no service password-encryption
!
hostname Router
!
enable password 5 
enable secret 5 
!
interface FastEthernet0/0
no ip address
shutdown
!
!
end
"#
        .to_string()
    
);
    startup_config
}


pub fn help_command(context: &CliContext){
    println!("\n ");
                println!(r#"Help may be requested at any point in a command by entering
a question mark '?'. If nothing matches, the help list will
be empty and you must backup until entering a '?' shows the
available options.
Two styles of help are provided:
1. Full help is available when you are ready to enter a
   command argument (e.g. 'show ?') and describes each possible
   argument.
2. Partial help is provided when an abbreviated argument is entered
   and you want to know what arguments match the input
   (e.g. 'show pr?'.
"#);
                println!("\nAvailable commands");
                println!("\n ");
                
                if matches!(context.current_mode, Mode::UserMode) {
                    println!("enable            - Enter privileged mode");
                    println!("exit              - Exit current mode");
                    println!("ping              - Send ICMP echo request");
                    println!("traceroute        - Display the packet transfer path");
                    println!("help              - Display available commands");
                    println!("reload            - Reload the system");
                    println!("clear             - Clear the terminal");
                    println!("show              - Some available show commands are present. To view enter 'show ?'");
                    println!("write             - Save the configuration");
                    println!("ifconfig          - Display interface configuration");
                    println!("connect           - Connect the Network Processor or the SEM");
                }
                else if matches!(context.current_mode, Mode::PrivilegedMode) {
                    println!("configure         - Enter configuration mode");
                    println!("exit              - Exit to user mode");
                    println!("help              - Display available commands");
                    println!("write             - Save the configuration");
                    println!("copy              - Copy configuration files");
                    println!("clock             - Manage system clock");
                    println!("ping              - Send ICMP echo request");
                    println!("traceroute        - Display the packet transfer path");
                    println!("show              - Some available show commands are present. To view enter 'show ?'");
                    println!("ifconfig          - Display interface configuration");
                    println!("reload            - Reload the system");
                    println!("clear             - Clear the terminal");
                    println!("debug             - Debug the availbale processes");
                    println!("undebug           - Undebug the availbale processes");
                    println!("connect           - Connect the Network Processor or the SEM");
                    println!("ssh               - Connect via SSH or show ssh version");
                    println!("disable           - Exit the Privileged EXEC Mode and enter the USER EXEC Mode");
                }
                else if matches!(context.current_mode, Mode::ConfigMode) {
                    println!("hostname          - Set system hostname");
                    println!("exit              - Exit to privileged mode");
                    println!("help              - Display available commands");
                    println!("write             - Save the configuration");
                    println!("ping              - Send ICMP echo request");
                    println!("traceroute        - Display the packet transfer path");
                    println!("enable            - Enter privileged mode");
                    println!("service password encryption - Encrypt passwords defined for the device");
                    println!("ifconfig          - Configure interface");
                    println!("ntp               - Configure NTP");
                    println!("no ntp            - Remove NTP configurations");
                    println!("reload            - Reload the system");
                    println!("interface         - Select another interface");
                    println!("clear             - Clear the terminal");
                }
                else if matches!(context.current_mode, Mode::InterfaceMode) {
                    println!("exit              - Exit to config mode");
                    println!("shutdown          - Shutdown interface");
                    println!("no                - Negate a command");
                    println!("help              - Display available commands");
                    println!("write             - Save the configuration");
                    println!("interface         - Select another interface");
                    println!("ip address        - Set IP address");
                    println!("reload            - Reload the system");
                    println!("clear             - Clear the terminal");
                }
                
                println!("\n ");
}