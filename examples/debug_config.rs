use obsidian_mcp_server::config::Settings;

fn main() {
    println!("Config directory: {:?}", dirs::config_dir());
    
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("obsidian-vault-mcp.toml");
        println!("Expected config path: {:?}", config_path);
        println!("Config file exists: {}", config_path.exists());
        
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    println!("Config file content:");
                    println!("Raw bytes: {:?}", content.as_bytes());
                    println!("Content: {}", content);
                }
                Err(e) => {
                    println!("Failed to read config: {}", e);
                }
            }
        }
    }
    
    // 設定を読み込んでみる
    match Settings::load() {
        Ok(settings) => {
            println!("Settings loaded successfully: {:?}", settings);
        }
        Err(e) => {
            println!("Failed to load settings: {}", e);
        }
    }
}
