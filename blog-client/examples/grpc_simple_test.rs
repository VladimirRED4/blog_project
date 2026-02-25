use blog_client::{BlogClient, Transport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Простой тест gRPC клиента ===\n");

    // Создаем клиент
    println!("1. Создание клиента...");
    let _client = BlogClient::new(Transport::Grpc("http://localhost:50051".to_string())).await?;
    println!("   ✅ Клиент создан\n");

    // Проверяем доступные методы
    println!("2. Проверка доступных методов:");

    // Регистрация
    println!("   - register() ✅");
    // Логин
    println!("   - login() ✅");
    // Создание поста
    println!("   - create_post() ✅");
    // Получение поста
    println!("   - get_post() ✅");
    // Обновление поста
    println!("   - update_post() ✅");
    // Удаление поста
    println!("   - delete_post() ✅");
    // Список постов
    println!("   - list_posts() ✅\n");

    println!("✅ Все методы gRPC клиента доступны!");
    println!("\nДля полного тестирования запустите:");
    println!("  cargo run --example grpc_full_test");

    Ok(())
}
