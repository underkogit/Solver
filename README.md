# LuaBuild 🚀

**Modern Build Automation System powered by Rust + Lua**

LuaBuild - это современная система автоматизации сборки, которая сочетает производительность Rust с гибкостью и простотой Lua. Создавайте мощные скрипты сборки, используя богатый набор встроенных функций для работы с файлами, процессами, текстом и многим другим.

## ✨ Возможности

- 🔥 **Высокая производительность** - движок на Rust
- 📝 **Простой синтаксис** - скрипты на Lua
- 🛠️ **Богатый API** - более 80 встроенных функций
- 🌐 **Кроссплатформенность** - Windows, macOS, Linux
- ⚡ **Async/await поддержка** - неблокирующие операции
- 🎯 **Система целей** - организация сложных сценариев сборки
- 📊 **Real-time вывод** - отслеживание прогресса выполнения

## 🚀 Быстрый старт

### Установка

```bash
# Клонируем репозиторий
git clone https://github.com/your-username/luabuild.git
cd luabuild

# Собираем проект
cargo build --release

# Устанавливаем в систему (опционально)
cargo install --path .
```

### Первый скрипт

Создайте файл `build.lua`:

```lua
print_success("🚀 Начинаем сборку проекта!")

-- Проверяем существование исходников
if not dir_exists("src") then
    print_error("Папка src не найдена!")
    return
end

-- Создаем папку для сборки
create_dir("build")

-- Компилируем Rust проект
local success = task_run("cargo build --release", function(line)
    if contains(line, "Finished") then
        print_success("✓ " .. line)
    else
        println(line)
    end
end)

if success then
    print_success("🎉 Сборка завершена успешно!")
else
    print_error("❌ Ошибка при сборке")
end
```

Запустите скрипт:

```bash
luabuild build.lua
```

## 📚 Документация API

### 🎨 Базовые функции

#### Переменные окружения
```lua
-- Глобальные переменные
verbose          -- boolean: режим подробного вывода
target           -- string: текущая цель сборки  
lua_script_path  -- string: путь к выполняемому скрипту
```

#### Функции вывода
```lua
-- Цветной вывод
print_success("Операция успешна")  -- зеленый текст
print_error("Произошла ошибка")    -- красный текст в stderr
println("Обычное сообщение")       -- обычный вывод
```

### 📁 Работа с файлами и папками

#### Проверка существования
```lua
-- Проверка файлов и папок
local exists = file_exists("config.toml")  -- boolean
local is_dir = dir_exists("src")           -- boolean
```

#### Чтение и запись файлов
```lua
-- Чтение файла
local content = read_file("Cargo.toml")    -- string | nil
if content then
    println("Размер файла: " .. #content .. " байт")
end

-- Запись в файл
write_file("output.txt", "Hello, World!")

-- Работа с директориями
create_dir("build/release")        -- создает все промежуточные папки
delete_file("temp.txt")            -- удаляет файл
delete_dir("old_build")            -- рекурсивно удаляет папку
```

#### Копирование
```lua
-- Копирование файлов и папок
copy_file("source.txt", "destination.txt")
copy_dir("src_folder", "backup_folder")   -- рекурсивное копирование
```

#### Работа с путями
```lua
-- Абсолютные пути
local abs_path = get_full_path("../config")
local script_relative = resolve_path("data/input.json")  -- относительно скрипта
```

### ⚙️ Выполнение команд и процессов

#### Базовое выполнение команд
```lua
-- Простое выполнение с callback
local success = task_run("npm install", function(line)
    if starts_with(line, "[ERROR]") then
        print_error(trim(line))
    else
        println(line)
    end
end)
```

#### Выполнение с прогрессом
```lua
-- Детальная информация о прогрессе
task_with_progress("cargo test", function(progress)
    if progress.line then
        println(string.format("[%d] %s", progress.processed_lines, progress.line))
    elseif progress.error then
        print_error(progress.error)
    elseif progress.success ~= nil then
        print_success(string.format("Завершено за %d сек, обработано %d строк", 
                                   progress.total_time, progress.total_lines))
    end
end)
```

#### Real-time команды
```lua
-- Для команд с real-time выводом (ping, tail, etc.)
task_realtime("ping google.com -c 4", function(data)
    if data.line then
        println("PING: " .. data.line)
    elseif data.error then
        print_error("ERROR: " .. data.error)
    end
end)
```

#### Переменные окружения и система
```lua
-- Переменные окружения
local path = get_env("PATH")        -- string | nil
set_env("RUST_LOG", "debug")

-- Рабочая директория
local current = get_cwd()           -- string | nil
set_cwd("/home/user/project")       -- boolean

-- Определение платформы
local platform = get_platform()    -- "windows", "macos", "linux", "unknown"
if is_windows() then
    println("Работаем на Windows")
end
if is_unix() then
    println("Unix-система")
end
```

#### Cargo интеграция
```lua
-- Специальная функция для Cargo
local result = cargo_build(true)    -- true для release режима
println("Exit code: " .. result.exit_code)
println("Output: " .. result.stdout)
if result.success == 1 then
    print_success("Cargo сборка успешна!")
end
```

### 📝 Работа с текстом

#### Форматирование строк
```lua
-- Обрезка пробелов
local clean = trim("  hello world  ")        -- "hello world"
local left = trim_start("  hello")           -- "hello"
local right = trim_end("hello  ")            -- "hello"

-- Регистр
local upper = to_upper("Hello World")        -- "HELLO WORLD"
local lower = to_lower("Hello World")        -- "hello world"
local cap = capitalize("hello world")        -- "Hello world"

-- Разделение и объединение
local parts = split("a,b,c", ",")            -- {"a", "b", "c"}
local joined = join({"a", "b", "c"}, "-")    -- "a-b-c"

-- Выравнивание
local padded = pad_left("42", 5, "0")        -- "00042"
local right_pad = pad_right("42", 5, "0")    -- "42000"
```

#### Регулярные выражения
```lua
-- Поиск и проверка
local matches = regex_match("hello123", "\\d+")                    -- true
local found = regex_find("hello123world", "\\d+")                  -- "123"
local all = regex_find_all("hello123world456", "\\d+")             -- {"123", "456"}

-- Замена
local result = regex_replace("hello123world456", "\\d+", "XXX")    -- "helloXXXworldXXX"

-- Группы захвата
local captures = regex_capture("name: John, age: 25", "name: (\\w+), age: (\\d+)")
-- {["0"] = "name: John, age: 25", ["1"] = "John", ["2"] = "25"}
```

#### Анализ текста
```lua
-- Подсчеты
local words = word_count("Hello world from Rust")    -- 4
local lines = line_count("line1\nline2\nline3")      -- 3
local chars = char_count("привет мир")               -- 10

-- Проверки
local has = contains("hello world", "world")         -- true
local starts = starts_with("hello world", "hello")   -- true
local ends = ends_with("hello world", "world")       -- true
```

#### Преобразования
```lua
-- Различные преобразования
local reversed = reverse("hello")                                    -- "olleh"
local repeated = repeat_text("abc", 3)                              -- "abcabcabc"
local no_spaces = remove_whitespace("h e l l o")                    -- "hello"
local normalized = normalize_whitespace("hello    world  \n\t test") -- "hello world test"

-- Подстроки и поиск
local substr = substring("hello world", 6, 5)        -- "world"
local index = find_index("hello world", "world")     -- 6
local replaced = replace_text("hello world world", "world", "Rust") -- "hello Rust Rust"
```

### 🛠️ Утилиты

#### Преобразование типов
```lua
-- Конвертация в строку
local str = to_string(42)           -- "42"
local str2 = to_string(true)        -- "true"
local str3 = to_string({a = 1})     -- "[table]"

-- Определение типа
local t = get_type(42)              -- "integer"
local t2 = get_type("hello")        -- "string"
local t3 = get_type({})             -- "table"
```

#### Включение других скриптов
```lua
-- Абсолютный путь
include("/path/to/script.lua")

-- Относительно текущего скрипта
include_local("modules/helper.lua")
include_local("../common/utils.lua")
```

#### Отладка
```lua
-- Подробный вывод структуры данных
debug_print({
    name = "test",
    values = {1, 2, 3},
    config = {debug = true}
})
```

#### Измерение времени
```lua
-- Создание и использование таймера
local timer = create_timer()
timer:start()

-- Выполняем операции
task_run("sleep 2", function(line) println(line) end)

local elapsed = timer:stop()
println("Операция заняла: " .. elapsed .. " секунд")

-- Можно проверить время без остановки
local current = timer:elapsed()
```

#### Работа с таблицами
```lua
-- Утилиты для таблиц
local count = table_length({a = 1, b = 2, c = 3})    -- 3
local empty = table_is_empty({})                      -- true

-- Объединение таблиц
local merged = table_merge({a = 1}, {b = 2, a = 3})  -- {a = 3, b = 2}
```

#### Математика
```lua
-- Ограничение диапазона
local clamped = clamp(15, 0, 10)      -- 10

-- Округление
local rounded = round(3.14159, 2)     -- 3.14
local int_round = round(3.7)          -- 4
```

#### Случайные данные
```lua
-- Генерация случайных значений
local random_str = random_string(8)        -- "aB3xY9Qm"
local random_num = random_number(1, 100)   -- число от 1 до 100
```

#### JSON поддержка
```lua
-- Парсинг JSON
local data = json_parse('{"name": "test", "value": 42}')
if data then
    println("Name: " .. data.name)      -- "test"
    println("Value: " .. data.value)    -- 42
end

-- Сериализация в JSON
local json_str = json_stringify({
    name = "project",
    version = "1.0.0",
    dependencies = {"rust", "lua"}
})
-- '{"name":"project","version":"1.0.0","dependencies":["rust","lua"]}'
```

## 📖 Примеры использования

### 1. Простая сборка Rust проекта

```lua
-- build.lua
print_success("🚀 Сборка Rust проекта")

if not file_exists("Cargo.toml") then
    print_error("Cargo.toml не найден!")
    return
end

-- Очищаем старую сборку
if dir_exists("target") then
    delete_dir("target")
    println("Очищена папка target")
end

-- Запускаем сборку
local success = task_run("cargo build --release", function(line)
    if contains(line, "Compiling") then
        print_success("📦 " .. line)
    elseif contains(line, "Finished") then
        print_success("✅ " .. line)
    elseif contains(line, "error") then
        print_error("❌ " .. line)
    else
        println(line)
    end
end)

if success then
    print_success("🎉 Сборка завершена!")
    
    -- Копируем бинарник
    if file_exists("target/release/myapp") then
        create_dir("dist")
        copy_file("target/release/myapp", "dist/myapp")
        print_success("📦 Бинарник скопирован в dist/")
    end
else
    print_error("💥 Сборка провалена!")
end
```

### 2. Веб-проект с тестами

```lua
-- web-build.lua
local timer = create_timer()
timer:start()

print_success("🌐 Сборка веб-проекта")

-- Проверяем зависимости
if not file_exists("package.json") then
    print_error("package.json не найден!")
    return
end

-- Устанавливаем зависимости
print_success("📦 Установка зависимостей...")
local npm_success = task_run("npm install", function(line)
    if contains(line, "added") then
        print_success("➕ " .. line)
    elseif contains(line, "warn") then
        println("⚠️  " .. line)
    end
end)

if not npm_success then
    print_error("Ошибка установки зависимостей")
    return
end

-- Запускаем тесты
print_success("🧪 Запуск тестов...")
local test_success = task_with_progress("npm test", function(progress)
    if progress.line then
        if contains(progress.line, "PASS") then
            print_success("✅ " .. progress.line)
        elseif contains(progress.line, "FAIL") then
            print_error("❌ " .. progress.line)
        else
            println(progress.line)
        end
    elseif progress.success ~= nil then
        if progress.success then
            print_success("🎉 Все тесты прошли!")
        else
            print_error("💥 Тесты провалены!")
        end
    end
end)

if not test_success then
    print_error("Тесты не прошли, сборка остановлена")
    return
end

-- Сборка для продакшена
print_success("🏗️  Сборка для продакшена...")
local build_success = task_run("npm run build", function(line)
    if contains(line, "Built at") then
        print_success("🚀 " .. line)
    elseif contains(line, "ERROR") then
        print_error("❌ " .. line)
    end
end)

if build_success then
    local elapsed = timer:elapsed()
    print_success(string.format("🎉 Сборка завершена за %.2f секунд!", elapsed))
    
    -- Архивируем результат
    if dir_exists("dist") then
        task_run("tar -czf release.tar.gz dist/", function(line)
            println(line)
        end)
        print_success("📦 Создан архив release.tar.gz")
    end
else
    print_error("💥 Ошибка сборки!")
end
```

### 3. Развертывание с бэкапом

```lua
-- deploy.lua
local config = json_parse(read_file("deploy-config.json"))
if not config then
    print_error("Не удалось загрузить конфигурацию развертывания")
    return
end

print_success("🚀 Начинаем развертывание на " .. config.server)

-- Создаем бэкап
local timestamp = os.date("%Y%m%d-%H%M%S")
local backup_name = "backup-" .. timestamp .. ".tar.gz"

print_success("💾 Создаем бэкап...")
local backup_success = task_run("tar -czf " .. backup_name .. " " .. config.deploy_path, 
function(line)
    println(line)
end)

if not backup_success then
    print_error("Не удалось создать бэкап!")
    return
end

-- Загружаем новую версию
print_success("📤 Загружаем новую версию...")
local upload_cmd = string.format("rsync -avz --delete dist/ %s@%s:%s", 
                                 config.user, config.server, config.deploy_path)

local upload_success = task_with_progress(upload_cmd, function(progress)
    if progress.line then
        if contains(progress.line, "sent") then
            print_success("📊 " .. progress.line)
        else
            println(progress.line)
        end
    end
end)

if upload_success then
    print_success("🎉 Развертывание завершено!")
    
    -- Перезапускаем сервис
    if config.restart_command then
        print_success("🔄 Перезапускаем сервис...")
        local restart_cmd = string.format("ssh %s@%s '%s'", 
                                         config.user, config.server, config.restart_command)
        task_run(restart_cmd, function(line) println(line) end)
    end
else
    print_error("💥 Ошибка развертывания!")
    
    -- Восстанавливаем из бэкапа
    print_success("🔄 Восстанавливаем из бэкапа...")
    local restore_cmd = string.format("ssh %s@%s 'cd %s && tar -xzf %s'", 
                                     config.user, config.server, config.deploy_path, backup_name)
    task_run(restore_cmd, function(line) println(line) end)
end
```

### 4. Кроссплатформенная сборка

```lua
-- cross-build.lua
print_success("🌍 Кроссплатформенная сборка")

local targets = {"x86_64-pc-windows-gnu", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu"}
local platform = get_platform()
local build_dir = "builds"

create_dir(build_dir)

for i, target in ipairs(targets) do
    print_success(string.format("🔨 Сборка для %s (%d/%d)", target, i, #targets))
    
    local timer = create_timer()
    timer:start()
    
    local cmd = "cargo build --release --target " .. target
    local success = task_run(cmd, function(line)
        if contains(line, "Compiling") then
            println("📦 " .. line)
        elseif contains(line, "Finished") then
            print_success("✅ " .. line)
        elseif contains(line, "error") then
            print_error("❌ " .. line)
        end
    end)
    
    local elapsed = timer:stop()
    
    if success then
        print_success(string.format("✅ %s собран за %.2f сек", target, elapsed))
        
        -- Копируем бинарник
        local binary_name = "myapp"
        if contains(target, "windows") then
            binary_name = binary_name .. ".exe"
        end
        
        local source_path = string.format("target/%s/release/%s", target, binary_name)
        local dest_path = string.format("%s/%s-%s", build_dir, binary_name, target)
        
        if file_exists(source_path) then
            copy_file(source_path, dest_path)
            print_success("📦 " .. dest_path)
        end
    else
        print_error(string.format("💥 Ошибка сборки для %s", target))
    end
    
    println()
end

print_success("🎉 Кроссплатформенная сборка завершена!")
println("📁 Результаты в папке: " .. build_dir)
```

## 🎯 Система целей

LuaBuild поддерживает систему целей для организации сложных сценариев:

```lua
-- build-targets.lua

-- Определяем цели
local targets = {
    clean = function()
        print_success("🧹 Очистка...")
        if dir_exists("target") then delete_dir("target") end
        if dir_exists("dist") then delete_dir("dist") end
    end,
    
    build = function()
        print_success("🔨 Сборка...")
        return task_run("cargo build --release", function(line) println(line) end)
    end,
    
    test = function()
        print_success("🧪 Тестирование...")
        return task_run("cargo test", function(line) println(line) end)
    end,
    
    package = function()
        print_success("📦 Упаковка...")
        create_dir("dist")
        copy_file("target/release/myapp", "dist/myapp")
        return true
    end
}

-- Выполняем цель
if target and targets[target] then
    local success = targets[target]()
    if success then
        print_success("✅ Цель '" .. target .. "' выполнена")
    else
        print_error("❌ Ошибка выполнения цели '" .. target .. "'")
    end
else
    -- По умолчанию выполняем все
    for name, func in pairs(targets) do
        print_success("🎯 Выполняется: " .. name)
        if not func() then
            print_error("💥 Остановка на цели: " .. name)
            break
        end
    end
end
```

Запуск конкретной цели:
```bash
luabuild build-targets.lua --target clean
luabuild build-targets.lua --target build
```

## 🔧 Аргументы командной строки

```bash
luabuild [OPTIONS] <SCRIPT>

Arguments:
  <SCRIPT>  Path to the Lua build script

Options:
  -t, --target <TARGET>     Specify build target
  -l, --list-targets        List available targets
  -v, --verbose             Enable verbose output
  -h, --help                Print help
  -V, --version             Print version
```

Примеры:
```bash
# Обычный запуск
luabuild build.lua

# С подробным выводом
luabuild build.lua --verbose

# Указание цели
luabuild build.lua --target release

# Список доступных целей
luabuild build.lua --list-targets
```

## 🤝 Участие в разработке

Мы приветствуем участие в развитии проекта! 

### Требования для разработки

- Rust 1.70+
- Git

### Зависимости

```toml
[dependencies]
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
colored = "2.0"
mlua = { version = "0.9", features = ["lua54", "async", "send"] }
tokio = { version = "1.0", features = ["full"] }
regex = "1.0"
rand = "0.8"
serde_json = "1.0"
```

### Сборка из исходников

```bash
git clone https://github.com/your-username/luabuild.git
cd luabuild
cargo build --release
```

### Запуск тестов

```bash
cargo test
```

## 📄 Лицензия

Этот проект распространяется под лицензией MIT. См. файл [LICENSE](LICENSE) для подробностей.

## 🙏 Благодарности

- [mlua](https://github.com/khvzak/mlua) - Rust биндинги для Lua
- [tokio](https://tokio.rs/) - асинхронная среда выполнения
- [clap](https://clap.rs/) - парсер аргументов командной строки
- [colored](https://github.com/mackwic/colored) - цветной вывод в терминал

---

**LuaBuild** - делаем автоматизацию сборки простой и мощной! 🚀