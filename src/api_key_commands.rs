use crate::config::Config;
use crate::error::{Result, VelnError};
use std::io::Write;

pub fn cmd_api_key_generate(name: String, role: String, auto: bool) -> Result<()> {
    // Validate role
    let valid_roles = ["admin", "operator", "viewer"];
    if !valid_roles.contains(&role.as_str()) {
        return Err(VelnError::Config(format!(
            "Invalid role '{}'. Must be one of: admin, operator, viewer",
            role
        )));
    }

    // Generate or get API key
    let api_key = if auto {
        // Generate secure random key
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    } else {
        // Prompt user for key
        print!("Enter API key (or press Enter to auto-generate): ");
        std::io::stdout().flush().map_err(|e| VelnError::Other(e.to_string()))?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| VelnError::Other(e.to_string()))?;
        let input = input.trim();

        if input.is_empty() {
            // Auto-generate
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
            hex::encode(bytes)
        } else {
            input.to_string()
        }
    };

    // Load current config
    let config_path = std::env::var("VELN_CONFIG")
        .map_or_else(|_| std::path::PathBuf::from("/usr/local/etc/veln/config.toml"), std::path::PathBuf::from);

    let mut config_content = if std::path::Path::new(&config_path).exists() {
        std::fs::read_to_string(&config_path).map_err(|e| VelnError::Config(format!("Failed to read config: {}", e)))?
    } else {
        // Create default config following FreeBSD conventions
        String::from("# Veln Configuration\n\
# FreeBSD paths:\n\
# - Config: /usr/local/etc/veln/\n\
# - VM data: zroot/veln/\n\
# - Logs: /var/log/veln/\n\
# - Runtime: /var/run/veln/\n\
\n\
zfs_pool = \"zroot\"\n\
vm_root = \"veln/vms\"\n\
iso_root = \"veln/isos\"\n\
\n\
[api]\n\
auth_enabled = true\n\
\n\
[api.keys]\n")
    };

    // Check if [api.keys] section exists, add if not
    let mut needs_newline = false;
    if !config_content.contains("[api.keys]") {
        if !config_content.contains("[api]") {
            config_content.push_str("\n[api]\nauth_enabled = true\n");
        }
        config_content.push_str("\n[api.keys]\n");
    } else {
        // Section exists, check if we need a newline before adding key
        needs_newline = !config_content.ends_with('\n');
    }

    // Add the new key with proper newline separation
    if needs_newline {
        config_content.push('\n');
    }
    let key_entry = format!("\"{}\" = {{ name = \"{}\", role = \"{}\" }}\n", api_key, name, role);
    config_content.push_str(&key_entry);

    // Write config back
    std::fs::write(&config_path, config_content).map_err(|e| {
        VelnError::Config(format!("Failed to write config to {}: {}", config_path.display(), e))
    })?;

    println!("✅ API key added successfully!");
    println!();
    println!("Key: {}", api_key);
    println!("Name: {}", name);
    println!("Role: {}", role);
    println!();
    println!("Configuration updated: {}", config_path.display());
    println!();
    println!("Usage:");
    println!("  curl -H \"Authorization: Bearer {}\" http://localhost:8080/api/vms", api_key);
    println!();
    println!("  Web UI: Enter this key on the login page");

    Ok(())
}

pub fn cmd_api_key_list() -> Result<()> {
    let config = Config::load()?;

    if config.api.keys.is_empty() {
        println!("No API keys configured.");
        println!();
        println!("Generate a key with:");
        println!("  veln api-key generate --name \"My Key\" --role admin");
        return Ok(());
    }

    println!("Configured API Keys");
    println!("===================");
    println!();

    for (key, info) in &config.api.keys {
        let key_short = format!("{}...", &key[..8.min(key.len())]);
        println!("Key:    {}", key_short);
        println!("Name:   {}", info.name);
        println!("Role:   {}", info.role);
        println!();
    }

    println!("Total: {} key(s)", config.api.keys.len());

    Ok(())
}

pub fn cmd_api_key_revoke(key_prefix: String, yes: bool) -> Result<()> {
    let config_path = std::env::var("VELN_CONFIG")
        .map_or_else(|_| std::path::PathBuf::from("/usr/local/etc/veln/config.toml"), std::path::PathBuf::from);

    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| VelnError::Config(format!("Failed to read config: {}", e)))?;

    // Find keys matching the prefix
    let config = Config::load()?;
    let matching_keys: Vec<_> = config.api.keys.keys()
        .filter(|k| k.starts_with(&key_prefix))
        .cloned()
        .collect();

    if matching_keys.is_empty() {
        return Err(VelnError::Config(format!("No API key found matching '{}'", key_prefix)));
    }

    if matching_keys.len() > 1 {
        println!("Multiple keys match '{}'", key_prefix);
        for key in &matching_keys {
            let info = config.api.keys.get(key).unwrap();
            println!("  {}... - {} ({})", &key[..8], info.name, info.role);
        }
        println!();
        println!("Please provide more characters to uniquely identify the key.");
        return Ok(());
    }

    let key_to_revoke = &matching_keys[0];
    let key_info = config.api.keys.get(key_to_revoke).unwrap();

    // Confirm revocation
    if !yes {
        println!("Are you sure you want to revoke this API key?");
        println!("  Key:  {}...", &key_to_revoke[..8]);
        println!("  Name: {}", key_info.name);
        println!("  Role: {}", key_info.role);
        println!();
        print!("Type 'yes' to confirm: ");
        std::io::stdout().flush().map_err(|e| VelnError::Other(e.to_string()))?;
        
        let mut confirmation = String::new();
        std::io::stdin().read_line(&mut confirmation).map_err(|e| VelnError::Other(e.to_string()))?;
        
        if confirmation.trim() != "yes" {
            println!("Revocation cancelled.");
            return Ok(());
        }
    }

    // Remove the key from config content
    // This is a simple approach - remove lines containing the key
    let lines: Vec<&str> = config_content.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip_next = false;
    
    for line in lines {
        if skip_next {
            skip_next = false;
            continue;
        }
        if line.contains(key_to_revoke) {
            // Skip this line (it contains the key)
            continue;
        }
        new_lines.push(line);
    }

    let new_config = new_lines.join("\n");

    // Write config back
    std::fs::write(&config_path, new_config).map_err(|e| {
        VelnError::Config(format!("Failed to write config: {}", e))
    })?;

    println!("✅ API key revoked successfully!");
    println!("  Key:  {}...", &key_to_revoke[..8]);
    println!("  Name: {}", key_info.name);

    Ok(())
}
