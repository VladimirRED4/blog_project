#!/bin/bash

# Перед запуском теста необходимо очистить базу данных
# TRUNCATE posts CASCADE;
# TRUNCATE users CASCADE;

# Цвета для вывода
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Счетчик тестов
TESTS_PASSED=0
TESTS_FAILED=0

# Функция для запуска теста
run_test() {
    local test_name="$1"
    local command="$2"
    local expected_status="$3"
    local expected_output="$4"
    local expected_count="$5"  # Опционально: ожидаемое количество постов

    echo -e "${YELLOW}▶ Тест: ${test_name}${NC}"
    echo "   Команда: $command"

    # Запускаем команду и сохраняем вывод и статус
    output=$(eval "$command" 2>&1)
    status=$?

    # Проверяем статус
    if [ $status -eq $expected_status ]; then
        echo -e "   ${GREEN}✓ Статус: ожидаемый ($expected_status)${NC}"
    else
        echo -e "   ${RED}✗ Статус: ожидаемый $expected_status, получен $status${NC}"
        echo "$output" | sed 's/^/     /'
        ((TESTS_FAILED++))
        return 1
    fi

    # Проверяем наличие ожидаемого текста в выводе
    if [[ -n "$expected_output" ]] && [[ "$output" == *"$expected_output"* ]]; then
        echo -e "   ${GREEN}✓ Вывод содержит: \"$expected_output\"${NC}"
    elif [[ -z "$expected_output" ]]; then
        # OK - нет проверки вывода
        :
    else
        echo -e "   ${RED}✗ Вывод не содержит: \"$expected_output\"${NC}"
        echo "   Полный вывод:"
        echo "$output" | sed 's/^/     /'
        ((TESTS_FAILED++))
        return 1
    fi

    # Проверяем количество найденных постов (если указано)
    if [[ -n "$expected_count" ]]; then
        # Извлекаем количество постов из вывода
        found_count=$(echo "$output" | grep -o "Found [0-9]* posts" | grep -o "[0-9]*")
        if [[ "$found_count" == "$expected_count" ]]; then
            echo -e "   ${GREEN}✓ Количество постов: $found_count (ожидалось $expected_count)${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "   ${RED}✗ Количество постов: $found_count, ожидалось $expected_count${NC}"
            ((TESTS_FAILED++))
            return 1
        fi
    else
        ((TESTS_PASSED++))
    fi

    echo -e "   ${BLUE}---${NC}"
    return 0
}

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  Тестирование Blog CLI${NC}"
echo -e "${BLUE}================================${NC}\n"

# Убедимся, что сервер запущен
echo -e "${YELLOW}Проверка сервера...${NC}"
if ! curl -s http://localhost:3000/api/posts > /dev/null; then
    echo -e "${RED}❌ Сервер не запущен!${NC}"
    echo "Запустите сервер: cd blog-server && cargo run"
    exit 1
fi
echo -e "${GREEN}✓ Сервер работает${NC}\n"

# Очищаем старый токен
rm -f ~/.blog_token

# Переходим в директорию CLI
cd "$(dirname "$0")"

# Генерируем уникальные имена
TIMESTAMP=$(date +%s)
USERNAME="testuser_$TIMESTAMP"
EMAIL="test_$TIMESTAMP@example.com"
PASSWORD="password123"

# Тест 1: Регистрация нового пользователя
run_test "Регистрация пользователя" \
    "cargo run -- register --username \"$USERNAME\" --email \"$EMAIL\" --password \"$PASSWORD\"" \
    0 "Registration successful"

# Тест 2: Попытка повторной регистрации
run_test "Повторная регистрация" \
    "cargo run -- register --username \"$USERNAME\" --email \"$EMAIL\" --password \"$PASSWORD\"" \
    1 "Registration failed"

# Тест 3: Логин с правильным паролем
run_test "Логин" \
    "cargo run -- login --username \"$USERNAME\" --password \"$PASSWORD\"" \
    0 "Login successful"

# Тест 4: Логин с неправильным паролем
run_test "Логин с неверным паролем" \
    "cargo run -- login --username \"$USERNAME\" --password \"wrongpassword\"" \
    1 "Login failed"

# Тест 5: Проверка статуса токена
run_test "Проверка токена" \
    "cargo run -- status" \
    0 "Token file:"

# Тест 6: Создание первого поста
run_test "Создание первого поста" \
    "cargo run -- create --title \"Первый тестовый пост\" --content \"Содержание первого тестового поста\"" \
    0 "Post created successfully"

# Сохраняем ID первого поста
FIRST_POST_ID=$(cargo run -- list 2>/dev/null | grep -o '\[[0-9]*\]' | head -1 | tr -d '[]')
echo -e "${BLUE}   ID первого поста: $FIRST_POST_ID${NC}"

# Тест 7: Создание второго поста
run_test "Создание второго поста" \
    "cargo run -- create --title \"Второй тестовый пост\" --content \"Содержание второго тестового поста\"" \
    0 "Post created successfully"

# Тест 8: Создание третьего поста
run_test "Создание третьего поста" \
    "cargo run -- create --title \"Третий тестовый пост\" --content \"Содержание третьего тестового поста\"" \
    0 "Post created successfully"

# Тест 9: Список постов (должно быть 3)
run_test "Список постов (проверка количества)" \
    "cargo run -- list --limit 10 --offset 0" \
    0 "" "3"

# Тест 10: Получение поста по ID
if [ ! -z "$FIRST_POST_ID" ]; then
    run_test "Получение первого поста" \
        "cargo run -- get --id $FIRST_POST_ID" \
        0 "Post retrieved"
fi

# Тест 11: Получение несуществующего поста
run_test "Получение несуществующего поста" \
    "cargo run -- get --id 99999" \
    1 "not found"

# Тест 12: Обновление первого поста
if [ ! -z "$FIRST_POST_ID" ]; then
    run_test "Обновление первого поста" \
        "cargo run -- update --id $FIRST_POST_ID --title \"Обновленный заголовок\" --content \"Обновленное содержание\"" \
        0 "Post updated successfully"
fi

# Тест 13: Список после обновления (все еще должно быть 3)
run_test "Список после обновления" \
    "cargo run -- list --limit 10 --offset 0" \
    0 "" "3"

# Тест 14: Удаление первого поста
if [ ! -z "$FIRST_POST_ID" ]; then
    run_test "Удаление первого поста" \
        "cargo run -- delete --id $FIRST_POST_ID" \
        0 "Post deleted successfully"
fi

# Тест 15: Список после удаления (должно быть 2)
run_test "Список после удаления" \
    "cargo run -- list --limit 10 --offset 0" \
    0 "" "2"

# Тест 16: Список через gRPC
run_test "Список через gRPC" \
    "cargo run -- --grpc list --limit 5 --offset 0" \
    0 "Found"

# Тест 17: Справка
run_test "Справка" \
    "cargo run -- --help" \
    0 "Blog CLI"

# Тест 18: Выход из системы (удаление токена)
run_test "Выход из системы" \
    "rm -f ~/.blog_token && echo 'Token removed'" \
    0 "Token removed"

echo -e "\n${BLUE}================================${NC}"
echo -e "${BLUE}  Результаты тестирования${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "${GREEN}✅ Пройдено: $TESTS_PASSED${NC}"
echo -e "${RED}❌ Провалено: $TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 Все тесты пройдены успешно!${NC}"
else
    echo -e "\n${RED}⚠ Некоторые тесты не пройдены${NC}"
    exit 1
fi