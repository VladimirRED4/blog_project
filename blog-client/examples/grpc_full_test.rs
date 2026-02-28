use blog_client::{BlogClient, Transport};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Тестирование gRPC клиента");
    println!("============================\n");

    // Создаем клиент с gRPC транспортом
    println!("📡 Создание gRPC клиента...");
    let client = BlogClient::new(Transport::Grpc("http://localhost:50051".to_string())).await?;
    println!("✅ Клиент создан\n");

    // Генерируем уникальные имена для тестов
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let username = format!("grpc_user_{}", timestamp);
    let email = format!("grpc_{}@example.com", timestamp);
    let password = "testpassword123";

    // Тест 1: Регистрация
    println!("📝 Тест 1: Регистрация пользователя");
    println!("   Username: {}", username);
    println!("   Email: {}", email);

    match client
        .register(
            username.clone(),
            email.clone(),
            password.to_string(),
            // "Test User".to_string(),
        )
        .await
    {
        Ok(response) => {
            println!("   ✅ Регистрация успешна!");
            println!("   📊 User ID: {}", response.user.id);
            println!("   👤 Username: {}", response.user.username);
            println!("   📧 Email: {}", response.user.email);
            println!(
                "   🔐 Токен при регистрации: {}",
                if response.token.is_empty() {
                    "❌ НЕ ПОЛУЧЕН (ошибка!)"
                } else {
                    "✅ получен"
                }
            );
            if response.token.is_empty() {
                println!("   ❌ ОШИБКА: gRPC регистрация должна возвращать токен!");
            }
        }
        Err(e) => println!("   ❌ Ошибка регистрации: {}", e),
    }
    println!("");

    // Тест 2: Логин
    println!("🔑 Тест 2: Логин пользователя");
    match client.login(username.clone(), password.to_string()).await {
        Ok(response) => {
            println!("   ✅ Логин успешен!");
            println!("   📊 User ID: {}", response.user.id);
            println!("   👤 Username: {}", response.user.username);
            println!("   📧 Email: {}", response.user.email);
            println!("   🔐 Токен получен: {}...", &response.token[..20]);

            // Сохраняем токен для следующих тестов
            client.set_token(response.token.clone()).await;
            println!("   💾 Токен сохранен\n");
        }
        Err(e) => {
            println!("   ❌ Ошибка логина: {}", e);
            return Ok(());
        }
    }

    // Небольшая пауза
    sleep(Duration::from_millis(500)).await;

    // Тест 3: Создание поста
    println!("📝 Тест 3: Создание поста");
    match client
        .create_post(
            "Мой первый gRPC пост".to_string(),
            "Это тестовый пост, созданный через gRPC клиент".to_string(),
        )
        .await
    {
        Ok(post) => {
            println!("   ✅ Пост создан успешно!");
            println!("   📊 ID: {}", post.id);
            println!("   📌 Заголовок: {}", post.title);
            println!("   📄 Содержание: {}", post.content);
            println!("   👤 Автор ID: {}", post.author_id);
            println!("   📅 Создан: {}", post.created_at);

            // Сохраняем ID поста для следующих тестов
            let post_id = post.id;
            println!("");

            // Тест 4: Получение поста по ID
            println!("🔍 Тест 4: Получение поста #{}", post_id);
            match client.get_post(post_id).await {
                Ok(post) => {
                    println!("   ✅ Пост получен!");
                    println!("   📌 Заголовок: {}", post.title);
                    println!("   📄 Содержание: {}", post.content);
                }
                Err(e) => println!("   ❌ Ошибка получения поста: {}", e),
            }
            println!("");

            // Тест 5: Обновление поста
            println!("✏️ Тест 5: Обновление поста #{}", post_id);
            match client
                .update_post(
                    post_id,
                    Some("Обновленный заголовок".to_string()),
                    Some("Это обновленное содержание поста".to_string()),
                )
                .await
            {
                Ok(post) => {
                    println!("   ✅ Пост обновлен!");
                    println!("   📌 Новый заголовок: {}", post.title);
                    println!("   📄 Новое содержание: {}", post.content);
                }
                Err(e) => println!("   ❌ Ошибка обновления поста: {}", e),
            }
            println!("");

            // Тест 6: Список постов
            println!("📋 Тест 6: Список постов");
            match client.list_posts(Some(10), Some(0)).await {
                Ok(response) => {
                    println!("   ✅ Всего постов: {}", response.total);
                    println!("   📊 Показано: {}", response.posts.len());
                    for (i, post) in response.posts.iter().enumerate() {
                        println!("   {}. [{}] {}", i + 1, post.id, post.title);
                    }
                }
                Err(e) => println!("   ❌ Ошибка получения списка: {}", e),
            }
            println!("");

            // Тест 7: Удаление поста
            println!("🗑️ Тест 7: Удаление поста #{}", post_id);
            match client.delete_post(post_id).await {
                Ok(()) => println!("   ✅ Пост успешно удален!"),
                Err(e) => println!("   ❌ Ошибка удаления поста: {}", e),
            }
        }
        Err(e) => println!("   ❌ Ошибка создания поста: {}", e),
    }
    println!("");

    // Тест 8: Проверка токена
    println!("🔐 Тест 8: Проверка токена");
    match client.get_token().await {
        Some(token) => println!("   ✅ Токен в клиенте: {}...", &token[..20]),
        None => println!("   ❌ Токен не найден"),
    }

    println!("\n✅ Все тесты завершены!");
    Ok(())
}
