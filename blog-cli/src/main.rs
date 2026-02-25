use anyhow::{Context, Result};
use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

/// Blog CLI - интерфейс командной строки для управления блогом
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Адрес сервера (по умолчанию: http://localhost:3000 для HTTP, http://localhost:50051 для gRPC)
    #[arg(short, long)]
    server: Option<String>,

    /// Использовать gRPC вместо HTTP
    #[arg(long)]
    grpc: bool,

    /// Путь к файлу с токеном (по умолчанию: .blog_token в домашней директории)
    #[arg(long)]
    token_file: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Регистрация нового пользователя
    Register {
        /// Имя пользователя
        #[arg(short, long)]
        username: String,

        /// Email пользователя
        #[arg(short, long)]
        email: String,

        /// Пароль
        #[arg(short, long)]
        password: String,

        /// Полное имя (опционально)
        #[arg(short, long)]
        full_name: Option<String>,
    },

    /// Вход в систему
    Login {
        /// Имя пользователя
        #[arg(short, long)]
        username: String,

        /// Пароль
        #[arg(short, long)]
        password: String,
    },

    /// Показать информацию о текущем токене
    Status,

    /// Создание нового поста
    Create {
        /// Заголовок поста
        #[arg(short, long)]
        title: String,

        /// Содержание поста
        #[arg(short, long)]
        content: String,
    },

    /// Получение поста по ID
    Get {
        /// ID поста
        #[arg(short, long)]
        id: i64,
    },

    /// Обновление поста
    Update {
        /// ID поста
        #[arg(short, long)]
        id: i64,

        /// Новый заголовок (опционально)
        #[arg(short, long)]
        title: Option<String>,

        /// Новое содержание (опционально)
        #[arg(short, long)]
        content: Option<String>,
    },

    /// Удаление поста
    Delete {
        /// ID поста
        #[arg(short, long)]
        id: i64,
    },

    /// Список постов с пагинацией
    List {
        /// Количество постов на странице (по умолчанию: 10)
        #[arg(short, long, default_value_t = 10)]
        limit: i64,

        /// Смещение от начала (по умолчанию: 0)
        #[arg(short, long, default_value_t = 0)]
        offset: i64,
    },
}

struct TokenManager {
    token_path: PathBuf,
}

impl TokenManager {
    fn new(custom_path: Option<PathBuf>) -> Result<Self> {
        let token_path = match custom_path {
            Some(path) => path,
            None => {
                let home = dirs::home_dir().context("Failed to get home directory")?;
                home.join(".blog_token")
            }
        };

        Ok(Self { token_path })
    }

    fn save_token(&self, token: &str) -> Result<()> {
        fs::write(&self.token_path, token)
            .with_context(|| format!("Failed to save token to {:?}", self.token_path))?;

        // Устанавливаем права только для владельца на Unix системах
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.token_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&self.token_path, perms)?;
        }

        println!("✓ Token saved to {:?}", self.token_path);
        Ok(())
    }

    fn load_token(&self) -> Result<Option<String>> {
        match fs::read_to_string(&self.token_path) {
            Ok(token) => {
                let token = token.trim().to_string();
                if !token.is_empty() {
                    println!("✓ Token loaded from {:?}", self.token_path);
                    Ok(Some(token))
                } else {
                    Ok(None)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).context("Failed to read token file"),
        }
    }

    #[allow(dead_code)]
    fn clear_token(&self) -> Result<()> {
        if self.token_path.exists() {
            fs::remove_file(&self.token_path)
                .with_context(|| format!("Failed to remove token file {:?}", self.token_path))?;
            println!("✓ Token file removed");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Определяем транспорт
    let transport = if cli.grpc {
        let addr = cli
            .server
            .unwrap_or_else(|| "http://localhost:50051".to_string());
        Transport::Grpc(addr)
    } else {
        let addr = cli
            .server
            .unwrap_or_else(|| "http://localhost:3000".to_string());
        Transport::Http(addr)
    };

    println!("🔌 Connecting to: {}", transport_url(&transport));

    // Создаем клиент
    let client = BlogClient::new(transport)
        .await
        .context("Failed to create blog client")?;

    // Загружаем токен
    let token_manager = TokenManager::new(cli.token_file)?;
    if let Some(token) = token_manager.load_token()? {
        client.set_token(token).await;
        println!("🔑 Authenticated with saved token");
    }

    // Выполняем команду
    match &cli.command {
        Commands::Register {
            username,
            email,
            password,
            full_name,
        } => {
            let full_name = full_name.clone().unwrap_or_else(|| username.clone());
            println!("📝 Registering user: {}", username);

            match client.register(username, email, password, full_name).await {
                Ok(response) => {
                    println!("✅ Registration successful!");
                    println!("   User ID: {}", response.user.id);
                    println!("   Username: {}", response.user.username);
                    println!("   Email: {}", response.user.email);

                    token_manager.save_token(&response.token)?;
                }
                Err(e) => {
                    println!("❌ Registration failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Login { username, password } => {
            println!("🔑 Logging in as: {}", username);

            match client.login(username, password).await {
                Ok(response) => {
                    println!("✅ Login successful!");
                    println!("   User ID: {}", response.user.id);
                    println!("   Username: {}", response.user.username);
                    println!("   Email: {}", response.user.email);

                    token_manager.save_token(&response.token)?;
                }
                Err(e) => {
                    println!("❌ Login failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Status => match token_manager.load_token()? {
            Some(token) => {
                println!("🔑 Token file: {:?}", token_manager.token_path);
                println!("   Token: {}...", &token[..20]);
                println!("   Length: {} characters", token.len());
                println!("   Status: ✅ Active");
                println!("\n   To verify token, try: cargo run -- list");
            }
            None => {
                println!("❌ No token found");
                println!("   Please login first: cargo run -- login --username <username> --password <password>");
            }
        },

        Commands::Create { title, content } => {
            println!("📝 Creating new post...");

            match client.create_post(title, content).await {
                Ok(post) => {
                    println!("✅ Post created successfully!");
                    println!("   ID: {}", post.id);
                    println!("   Title: {}", post.title);
                    println!("   Author ID: {}", post.author_id);
                    println!("   Created: {}", post.created_at);
                }
                Err(e) => {
                    if e.is_unauthorized() {
                        println!("❌ Unauthorized. Please login first:");
                        println!(
                            "   cargo run -- login --username <username> --password <password>"
                        );
                    } else {
                        println!("❌ Failed to create post: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::Get { id } => {
            println!("🔍 Getting post #{}", id);

            match client.get_post(*id).await {
                Ok(post) => {
                    println!("✅ Post retrieved:");
                    println!("   ID: {}", post.id);
                    println!("   Title: {}", post.title);
                    println!("   Content: {}", post.content);
                    println!("   Author ID: {}", post.author_id);
                    println!("   Created: {}", post.created_at);
                    println!("   Updated: {}", post.updated_at);
                }
                Err(e) => {
                    if e.is_not_found() {
                        println!("❌ Post #{} not found", id);
                        println!("   Tip: Use 'list' command to see available posts");
                    } else {
                        println!("❌ Error: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::Update { id, title, content } => {
            println!("✏️ Updating post #{}", id);

            match client
                .update_post(*id, title.clone(), content.clone())
                .await
            {
                Ok(post) => {
                    println!("✅ Post updated successfully!");
                    println!("   ID: {}", post.id);
                    println!("   Title: {}", post.title);
                    println!("   Content: {}", post.content);
                    println!("   Author ID: {}", post.author_id);
                    println!("   Updated: {}", post.updated_at);
                }
                Err(e) => {
                    if e.is_not_found() {
                        println!("❌ Post #{} not found", id);
                    } else if e.is_unauthorized() {
                        println!(
                            "❌ Unauthorized. You may not own this post or need to login again"
                        );
                    } else {
                        println!("❌ Failed to update post: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::Delete { id } => {
            println!("🗑️ Deleting post #{}", id);

            match client.delete_post(*id).await {
                Ok(()) => {
                    println!("✅ Post deleted successfully!");
                }
                Err(e) => {
                    if e.is_not_found() {
                        println!("❌ Post #{} not found", id);
                    } else if e.is_unauthorized() {
                        println!(
                            "❌ Unauthorized. You may not own this post or need to login again"
                        );
                    } else {
                        println!("❌ Failed to delete post: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::List { limit, offset } => {
            println!("📋 Listing posts (limit={}, offset={})", limit, offset);

            match client.list_posts(Some(*limit), Some(*offset)).await {
                Ok(response) => {
                    println!(
                        "✅ Found {} posts (total: {})",
                        response.posts.len(),
                        response.total
                    );
                    println!();

                    if response.posts.is_empty() {
                        println!("   No posts found");
                        println!("   Tip: Create your first post: cargo run -- create --title \"My Post\" --content \"Hello\"");
                    } else {
                        for (i, post) in response.posts.iter().enumerate() {
                            println!("   {}. [{}] {}", i + 1, post.id, post.title);
                            println!("      Created: {}", post.created_at);
                            println!("      Content: {}", truncate(&post.content, 50));
                            println!();
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to list posts: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn transport_url(transport: &Transport) -> String {
    match transport {
        Transport::Http(url) => format!("HTTP: {}", url),
        Transport::Grpc(addr) => format!("gRPC: {}", addr),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
