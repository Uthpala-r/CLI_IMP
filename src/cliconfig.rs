/// External crates for the CLI application
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::execute::Mode;
use crate::network_config::NtpAssociation;


/// Represents the configuration for the CLI application.
///
/// This structure holds the following configuration details:
/// - `running_config`: A map containing the currently active configuration settings.
/// - `startup_config`: A map containing the startup configuration settings loaded at initialization.
/// - `hostname`: The hostname of the system.
/// etc.
///
/// # Examples
/// ```
/// let config = CliConfig::default();
/// assert_eq!(config.hostname, "PNF");
/// ```
/// 
#[derive(Serialize, Deserialize, Clone)]
pub struct CliConfig {
    pub running_config: Option<String>,
    pub startup_config: Option<String>,
    pub hostname: String,  
    pub enable_password: Option<String>,          
    pub enable_secret: Option<String>,  
    pub encrypted_password: Option<String>,          
    pub encrypted_secret: Option<String>,          
    pub password_encryption: bool,
    pub domain_name: Option<String>,
    pub last_written: Option<String>,     
  
}


impl Default for CliConfig {
    
    /// Provides the default values for `CliConfig`.
    ///
    /// - `running_config`: An empty `HashMap`.
    /// - `startup_config`: An empty `HashMap`.
    /// - `hostname`: `"PNF"`.
    /// - `crypto_ipsec_profile: None`,
    /// - `transform_sets: None`,
    /// - `tunnel_mode: None`,
    /// - `tunnel_source: None`,
    /// - `tunnel_destination: None`,
    /// - `tunnel_protection_profile: None`,
    /// - `virtual_template: None`,
    /// - `enable_password: None`,          
    /// - `enable_secret: None`,            
    /// - `password_encryption: false`, 
    /// - `domain_name: None`,
    fn default() -> Self {
        Self {
            running_config: None,
            startup_config: None,
            hostname: "PNF".to_string(),
            enable_password: None,          
            enable_secret: None,   
            encrypted_password: None,          
            encrypted_secret: None,         
            password_encryption: false, 
            domain_name: None,
            last_written: None,
            
        }
    }
}


/// Represents the current context of the CLI application.
///
/// The `CliContext` maintains the state of the CLI, including the current operational mode,
/// the system prompt, the configuration, and the currently selected interface (if any).
///
/// # Examples
/// ```
/// let context = CliContext::default();
/// assert_eq!(context.prompt, "PNF>");
/// ```
/// 
pub struct CliContext {
    pub current_mode: Mode,
    pub prompt: String,
    pub config: CliConfig,
    pub selected_interface: Option<String>,
    pub ntp_source_interface: Option<String>,
    pub ntp_servers: HashSet<String>,  
    pub ntp_associations: Vec<NtpAssociation>,  
    pub ntp_authentication_enabled: bool,   
    pub ntp_authentication_keys: HashMap<u32, String>, 
    pub ntp_trusted_keys: HashSet<u32>,     
    pub ntp_master: bool, 
}


impl Default for CliContext {

    /// Provides the default values for `CliContext`.
    ///
    /// - `current_mode`: `Mode::UserMode`.
    /// - `prompt`: `"PNF>"`.
    /// - `config`: The default configuration provided by `CliConfig::default()`.
    /// - `selected_interface`: `None`.
    /// - `selected_vlan`: `None`.
    /// - `vlan_names`: `None`,
    /// - `vlan_states`: `None`,
    /// - `switchport_mode`: `None`,
    /// - `trunk_encapsulation`: `None`,
    /// - `native_vlan`: `None`,
    /// - `allowed_vlans`: `HashSet::new()`,
    /// - `ntp_servers: HashSet::new()`, 
    /// - `ntp_associations: Vec::new()`,
    /// - `ntp_authentication_enabled: false`,   
    /// - `ntp_authentication_keys: HashMap::new()`, 
    /// - `ntp_trusted_keys: HashSet::new()`,     
    /// - `ntp_master: false,
    fn default() -> Self {
        Self {
            current_mode: Mode::UserMode,
            prompt: "PNF>".into(),
            config: CliConfig::default(),
            selected_interface: None,
            ntp_source_interface: None,
            ntp_servers: HashSet::new(), 
            ntp_associations: Vec::new(),
            ntp_authentication_enabled: false,   
            ntp_authentication_keys: HashMap::new(), 
            ntp_trusted_keys: HashSet::new(),     
            ntp_master: false,
        }
    }
}