-- Числа
print(to_string(42))        -- "42"
print(to_string(3.14))      -- "3.14"

-- Булевы значения
print(to_string(true))      -- "true"
print(to_string(false))     -- "false"

-- Nil
print(to_string(nil))       -- "nil"

-- Строки (остаются строками)
print(to_string("hello"))   -- "hello"

-- Комплексные типы
local t = {a = 1, b = 2}
print(to_string(t))         -- "[table]"

-- Конкатенация
local num = 42
local message = "Number is: " .. to_string(num)
print(message)              -- "Number is: 42"