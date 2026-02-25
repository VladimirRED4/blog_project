use blog_client::{BlogClient, Transport};
use std::time::Duration;
use tokio::time::sleep;

async fn test_registration() -> Result<String, String> {
    println!("📝 Тестирование регистрации...");

    let client = BlogClient::new(Transport::Grpc("http://localhost:50051".to_string()))
        .await
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let username = format!("test_{}", timestamp);
    let email = format!("test_{}@example.com", timestamp);

    let response = client.register(
        username.clone(),
        email.clone(),
        "password123".to_string(),
        "Test User".to_string(),
    ).await.map_err(|e| format!("Registration failed: {}", e))?;

    assert_eq!(response.user.username, username);
    assert_eq!(response.user.email, email);

    Ok(format!("✓ Регистрация: user_id={}", response.user.id))
}

async fn test_login() -> Result<String, String> {
    println!("🔑 Тестирование логина...");

    let client = BlogClient::new(Transport::Grpc("http://localhost:50051".to_string()))
        .await
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let username = format!("login_test_{}", timestamp);
    let password = "testpass123";

    // Сначала регистрируем пользователя
    let _ = client.register(
        username.clone(),
        format!("{}@example.com", username),
        password.to_string(),
        "Login Test User".to_string(),
    ).await.map_err(|e| format!("Pre-registration failed: {}", e))?;

    // Теперь логинимся
    let response = client.login(username, password.to_string())
        .await
        .map_err(|e| format!("Login failed: {}", e))?;

    assert!(!response.token.is_empty());

    Ok(format!("✓ Логин: token получен ({} chars)", response.token.len()))
}

async fn test_crud_operations() -> Result<String, String> {
    println!("📚 Тестирование CRUD операций...");

    let client = BlogClient::new(Transport::Grpc("http://localhost:50051".to_string()))
        .await
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let username = format!("crud_test_{}", timestamp);
    let password = "crudpass123";

    // Регистрация
    let _ = client.register(
        username.clone(),
        format!("{}@example.com", username),
        password.to_string(),
        "CRUD Test User".to_string(),
    ).await.map_err(|e| format!("Registration failed: {}", e))?;

    // Логин
    let login_resp = client.login(username, password.to_string())
        .await
        .map_err(|e| format!("Login failed: {}", e))?;

    client.set_token(login_resp.token.clone()).await;

    // Create
    let post = client.create_post(
        "Test Post".to_string(),
        "Test Content".to_string(),
    ).await.map_err(|e| format!("Create failed: {}", e))?;
    println!("   📌 Создан пост ID: {}", post.id);

    // Read
    let retrieved = client.get_post(post.id)
        .await
        .map_err(|e| format!("Get failed: {}", e))?;
    assert_eq!(retrieved.id, post.id);
    println!("   📖 Пост получен: {}", retrieved.title);

    // Update
    let updated = client.update_post(
        post.id,
        Some("Updated Title".to_string()),
        Some("Updated Content".to_string()),
    ).await.map_err(|e| format!("Update failed: {}", e))?;
    assert_eq!(updated.title, "Updated Title");
    println!("   ✏️ Пост обновлен: {}", updated.title);

    // Delete
    client.delete_post(post.id)
        .await
        .map_err(|e| format!("Delete failed: {}", e))?;
    println!("   🗑️ Пост удален");

    // Verify deletion
    let result = client.get_post(post.id).await;
    assert!(result.is_err());
    println!("   ✅ Пост не найден (ожидаемо)");

    Ok("✓ CRUD: все операции выполнены успешно".to_string())
}

async fn test_pagination() -> Result<String, String> {
    println!("📄 Тестирование пагинации...");

    let client = BlogClient::new(Transport::Grpc("http://localhost:50051".to_string()))
        .await
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let username = format!("pagination_test_{}", timestamp);
    let password = "paginate123";

    // Регистрация
    let _ = client.register(
        username.clone(),
        format!("{}@example.com", username),
        password.to_string(),
        "Pagination Test User".to_string(),
    ).await.map_err(|e| format!("Registration failed: {}", e))?;

    // Логин
    let login_resp = client.login(username, password.to_string())
        .await
        .map_err(|e| format!("Login failed: {}", e))?;

    client.set_token(login_resp.token.clone()).await;

    // Создаем несколько постов
    println!("   Создание 5 тестовых постов...");
    for i in 1..=5 {
        client.create_post(
            format!("Post {}", i),
            format!("Content {}", i),
        ).await.map_err(|e| format!("Failed to create post {}: {}", i, e))?;
    }

    // Тестируем пагинацию
    let page1 = client.list_posts(Some(2), Some(0)).await
        .map_err(|e| format!("Failed to list page1: {}", e))?;
    assert_eq!(page1.posts.len(), 2);
    println!("   Страница 1: {} постов", page1.posts.len());

    let page2 = client.list_posts(Some(2), Some(2)).await
        .map_err(|e| format!("Failed to list page2: {}", e))?;
    assert_eq!(page2.posts.len(), 2);
    println!("   Страница 2: {} постов", page2.posts.len());

    let page3 = client.list_posts(Some(2), Some(4)).await
        .map_err(|e| format!("Failed to list page3: {}", e))?;
    assert_eq!(page3.posts.len(), 1);
    println!("   Страница 3: {} постов", page3.posts.len());

    println!("   Всего постов: {}", page1.total);

    Ok(format!("✓ Пагинация: всего {} постов", page1.total))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Запуск тестов gRPC клиента");
    println!("===============================\n");

    let mut passed = 0;
    let total = 4;

    // Тест 1: Регистрация
    print!("🔄 Регистрация... ");
    match test_registration().await {
        Ok(result) => {
            println!("✅");
            println!("   {}\n", result);
            passed += 1;
        }
        Err(e) => println!("❌\n   Ошибка: {}\n", e),
    }
    sleep(Duration::from_millis(500)).await;

    // Тест 2: Логин
    print!("🔄 Логин... ");
    match test_login().await {
        Ok(result) => {
            println!("✅");
            println!("   {}\n", result);
            passed += 1;
        }
        Err(e) => println!("❌\n   Ошибка: {}\n", e),
    }
    sleep(Duration::from_millis(500)).await;

    // Тест 3: CRUD операции
    print!("🔄 CRUD операции... ");
    match test_crud_operations().await {
        Ok(result) => {
            println!("✅");
            println!("   {}\n", result);
            passed += 1;
        }
        Err(e) => println!("❌\n   Ошибка: {}\n", e),
    }
    sleep(Duration::from_millis(500)).await;

    // Тест 4: Пагинация
    print!("🔄 Пагинация... ");
    match test_pagination().await {
        Ok(result) => {
            println!("✅");
            println!("   {}\n", result);
            passed += 1;
        }
        Err(e) => println!("❌\n   Ошибка: {}\n", e),
    }

    println!("===============================");
    println!("📊 Результаты: {}/{} тестов пройдено", passed, total);

    if passed == total {
        println!("✅ Все тесты успешно пройдены!");
    } else {
        println!("❌ Некоторые тесты не пройдены");
    }

    Ok(())
}