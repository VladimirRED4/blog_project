# Blog Project - Многокомпонентный блог на Rust

[![CI](https://github.com/yourusername/blog-project/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/blog-project/actions/workflows/ci.yml)

## 📋 Описание проекта

Проект представляет собой полнофункциональную блог-платформу, реализованную на языке Rust с использованием современного стека технологий. Система состоит из нескольких взаимодействующих компонентов:

## 🏗 Архитектура проекта

```text
blog-project/
├── blog-server/          # Бэкенд-сервер (REST + gRPC API)
├── blog-client/          # Клиентская библиотека для взаимодействия с API
├── blog-cli/             # Интерфейс командной строки
└── blog-wasm/            # WebAssembly фронтенд
```

## 🔧 Компоненты

### blog-server (Порт 3000 HTTP / 50051 gRPC)

* REST API на Actix-web для браузерных клиентов

* gRPC API на Tonic для высокопроизводительных клиентов

* Аутентификация через JWT токены

* PostgreSQL база данных с миграциями

* Полное CRUD для постов с проверкой прав доступа

### blog-client (Клиентская библиотека)

* Унифицированный интерфейс для HTTP и gRPC транспортов

* Автоматическое управление JWT токенами

* Типизированные методы для всех операций

* Обработка ошибок через thiserror

### blog-cli (Командная строка)

* Удобный интерфейс для управления блогом

* Сохранение токена в файл ~/.blog_token

* Поддержка HTTP и gRPC транспортов

### blog-wasm (Веб-интерфейс)

* Фронтенд на Yew (React-подобный Rust фреймворк)

* Компиляция в WebAssembly

* Адаптивный дизайн

* Работа в браузере через HTTP API

## 🚀 Быстрый старт

### Настройка базы данных PostgreSQL

```bash
# Создание базы данных
createdb -U postgres blog_db

# Или через psql
psql -U postgres -c "CREATE DATABASE blog_db;"
```

### Настройка переменных окружения

* Создайте файл blog-server/.env:

```bash
# Database configuration
DATABASE_URL=postgres://postgres:postgres@localhost/blog_db

# JWT configuration (минимум 32 символа)
JWT_SECRET=your-very-long-secret-key-min-32-chars-here-change-it

# Server ports
HTTP_PORT=3000
GRPC_PORT=50051

# Database connection pool
DATABASE_MAX_CONNECTIONS=5

# Logging
RUST_LOG=debug,blog_server=debug,sqlx=warn

# CORS allowed origins (for WASM frontend)
CORS_ALLOWED_ORIGINS=http://localhost:8000,http://127.0.0.1:8000
```

## 📦 Сборка и запуск компонентов

### Запуск сервера

```bash
cd blog-server

# Сборка
cargo build --release

# Запуск
cargo run

# Или с подробным логированием
RUST_LOG=debug cargo run
```

* Ожидаемый вывод

```bash
Starting blog server...
HTTP server will listen on 0.0.0.0:3000
gRPC server will listen on 0.0.0.0:50051
Connecting to database...
Database connection pool created
Running database migrations...
Migrations completed successfully
Services initialized successfully
HTTP server running on 0.0.0.0:3000
gRPC server running on 0.0.0.0:50051
```

### Использование CLI

```bash
cd blog-cli

# Справка
cargo run -- --help

# Регистрация нового пользователя
cargo run -- register --username "ivan" --email "ivan@example.com" --password "secret123"

# Вход в систему
cargo run -- login --username "ivan" --password "secret123"

# Создание поста
cargo run -- create --title "Мой первый пост" --content "Привет, мир!"

# Список постов
cargo run -- list

# Получение поста по ID
cargo run -- get --id 1

# Обновление поста
cargo run -- update --id 1 --title "Новый заголовок" --content "Новое содержание"

# Удаление поста
cargo run -- delete --id 1

# Использование gRPC
cargo run -- --grpc list

# Проверка статуса токена
cargo run -- status
```

### Сборка и запуск WASM-фронтенда

```bash
cd blog-wasm

# Сборка WASM модуля
wasm-pack build --target web --dev

# Запуск HTTP сервера
python3 -m http.server 8000
```

* Откройте браузер по адресу <http://localhost:8000>

## 🧪 Тестирование

### Тестирование API через curl

```bash
# Переменные
BASE_URL="http://localhost:3000"

# Регистрация
curl -X POST $BASE_URL/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"test","email":"test@example.com","password":"password123","full_name":"Test User"}'

# Логин
curl -X POST $BASE_URL/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"test","password":"password123"}'

# Сохраняем токен (предполагаем, что получили его из ответа)
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

# Создание поста
curl -X POST $BASE_URL/api/protected/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Test Post","content":"Test Content"}'

# Список постов
curl "$BASE_URL/api/posts?limit=10&offset=0"

# Получение поста
curl "$BASE_URL/api/posts/1"

# Обновление поста
curl -X PUT $BASE_URL/api/protected/posts/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Updated Title","content":"Updated Content"}'

# Удаление поста
curl -X DELETE $BASE_URL/api/protected/posts/1 \
  -H "Authorization: Bearer $TOKEN"
```

### Тестирование gRPC

```bash
cd blog-server
cargo run

# На другом терминале
cd blog-client
cargo run --example grpc_test_runner
cargo run --example grpc_full_test
```
