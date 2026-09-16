# Разбор аргументов командной строки env::args() и env::args_os()

## Общая картина

Программа читает аргументы командной строки, разбирает их на **флаги** (`-l`, `-w`, `-p`) и **пути к файлам**, а затем считает статистику (количество строк и слов) для каждого файла. Демонстрируются три способа получения аргументов: `env::args()`, `env::args().skip(1)` и `env::args_os()`.

## Три способа получить аргументы в `main`

### 1. `env::args()` — все аргументы

```rust
let args = env::args().collect::<Vec<String>>();
println!("args: {args:?}");
// Out: args: ["target\\debug\\c_0010_cli.exe", "-l", "-w", "-p", "Cargo.toml", "Cargo.lock"]
```

- `env::args()` возвращает **итератор** `Args` с элементами `String`.
- **Первый элемент** — имя программы (путь к исполняемому файлу).
- `.collect::<Vec<String>>()` собирает все аргументы в вектор.
- **Паникует**, если аргумент содержит невалидный UTF-8.

### 2. `env::args().skip(1)` — без имени программы

```rust
println!("skip(1) for args: {:?}",
    env::args().skip(1).collect::<Vec<String>>()
);
// Out: skip(1) for args: ["-l", "-w", "-p", "Cargo.toml", "Cargo.lock"]
```

- `.skip(1)` пропускает **первый элемент** — имя программы.
- Остаются только пользовательские аргументы.

### 3. `env::args_os()` — безопасно для невалидного UTF-8

```rust
let os_args = env::args_os().collect::<Vec<OsString>>();
println!("os_args: {os_args:?}");
// Out: os_args: ["target\\debug\\c_0010_cli.exe", "-l", "-w", "-p", "Cargo.toml", "Cargo.lock"]
```

- `env::args_os()` возвращает итератор `ArgsOs` с элементами `OsString`.
- **Не паникует** на невалидном UTF-8.
- Используется, когда аргументы могут быть путями файлов с не-UTF8 символами.
- Именно этот вариант используется в `CliArgs::parse()`.

## Разбор `CliArgs::parse()`

### Структура `CliArgs`

```rust
#[derive(Debug)]
struct CliArgs {
    lines: bool,           // -l / --lines
    words: bool,           // -w / --words
    paths: Vec<PathBuf>,   // -p / --paths, затем список файлов
}
```

Хранит разобранные флаги и пути к файлам.

### Итератор аргументов

```rust
let mut args = env::args_os().skip(1);
```

- `env::args_os()` — итератор по `OsString`.
- `.skip(1)` — пропускаем имя программы.
- `mut` нужен, потому что мы будем вызывать `.next()` в цикле.

### Цикл разбора

```rust
while let Some(arg) = args.next() {
    match arg.to_str().unwrap_or_default() {
        "-l" | "--lines" => lines = true,
        "-w" | "--words" => words = true,
        "-p" | "--paths" => {
            paths.extend(args.by_ref().map(PathBuf::from));
            break;
        }
        _ => {},
    }
}
```

Разберём по веткам:

#### Ветка `-l` / `--lines`

```rust
"-l" | "--lines" => lines = true,
```

- `arg.to_str()` — пытается преобразовать `OsString` в `&str`.
- `.unwrap_or_default()` — если невалидный UTF-8, возвращает `""`.
- `match` сравнивает строку с литералами.
- `|` — альтернатива (или короткий, или длинный флаг).
- Устанавливает `lines = true`.

#### Ветка `-w` / `--words`

```rust
"-w" | "--words" => words = true,
```

Аналогично: `words = true`.

#### Ветка `-p` / `--paths`

```rust
"-p" | "--paths" => {
    paths.extend(args.by_ref().map(PathBuf::from));
    break;
}
```

Ключевая ветка:

- **`args.by_ref()`** — берёт **изменяемую ссылку** на итератор `args`, чтобы **не потребить его**. Без `by_ref()` итератор был бы перемещён в `map`, и его нельзя было бы использовать в следующей итерации `while let`.
- **`.map(PathBuf::from)`** — преобразует каждый оставшийся `OsString` в `PathBuf`. `PathBuf::from` принимает `OsString` напрямую.
- **`.extend(...)`** — «высасывает» весь итератор в `paths` (тип `Vec<PathBuf>`).
- **`break`** — выходит из цикла `while let`. Это означает: **после `-p` все оставшиеся аргументы считаются путями**, никакие другие флаги не разбираются.

#### Ветка `_ => {}`

```rust
_ => {},
```

- Игнорирует всё, что не совпало с флагами (например, `--help`, `-x`, случайные строки).
- Обязательна для **исчерпывающего** `match`.

### Возврат результата

```rust
Self { lines, words, paths }
```

Собирает структуру `CliArgs` из разобранных значений.

## Пример вызова

```
cargo run -- -l -w -p Cargo.toml Cargo.lock
```

Что происходит:

1. `env::args_os().skip(1)` → `["-l", "-w", "-p", "Cargo.toml", "Cargo.lock"]`.
2. `-l` → `lines = true`.
3. `-w` → `words = true`.
4. `-p` → `paths.extend(["Cargo.toml", "Cargo.lock"])`, затем `break`.
5. Результат:

```rust
CliArgs {
    lines: true,
    words: true,
    paths: ["Cargo.toml", "Cargo.lock"],
}
```

Вывод в `main`:

```
cli_args: CliArgs { lines: true, words: true, paths: ["Cargo.toml", "Cargo.lock"] }
```

## Использование разобранных аргументов в `stat_file`

```rust
fn stat_file(cli_arg: &CliArgs) -> Vec<Stats<'_>> {
    let mut stat_collection = Vec::<Stats>::new();

    if !cli_arg.lines && !cli_arg.words {
        return stat_collection;
    }

    for filepath in &cli_arg.paths {
        if !filepath.is_file() {
            continue;
        }

        let Ok(content) = fs::read_to_string(filepath) else {
            continue;
        };

        let lines = match cli_arg.lines {
            true => Some(content.lines().count()),
            false => None,
        };

        let words = match cli_arg.words {
            true => Some(
                content.lines()
                    .map(|line| line.split_whitespace().count())
                    .count()
            ),
            false => None,
        };

        stat_collection.push(Stats { filepath, lines, words });
    }

    stat_collection
}
```

- Если **ни один** флаг не установлен — возвращает пустой вектор (нечего считать).
- Для каждого пути:
  - проверяет, что это файл (`is_file()`);
  - читает содержимое (`fs::read_to_string`);
  - если `lines == true` — считает строки (`content.lines().count()`);
  - если `words == true` — считает слова (`split_whitespace().count()` по строкам);
  - складывает результат в `Stats`.
- Обратите внимание: `words` считается **неправильно** — `content.lines().map(...).count()` вернёт **количество строк**, а не слов. Правильно было бы `content.split_whitespace().count()`. Это баг в коде.

## Вывод статистики в `main`

```rust
for stat in stats {
    println!("Path: {:?}{}{}",
        stat.filepath,
        if let Some(l) = stat.lines {
            format!(" Lines: {}", l)
        } else {
            "".to_string()
        },
        if let Some(w) = stat.words {
            format!(" Words: {w}")
        } else {
            "".to_string()
        }
    );
}
```

- Для каждого `Stats` печатает путь.
- Если `lines` установлено — добавляет `Lines: N`.
- Если `words` установлено — добавляет `Words: N`.
- Формат: `Path: "Cargo.toml" Lines: 6 Words: 6`.

## Ключевые моменты использования аргументов

| Аспект | Описание |
|---|---|
| `env::args()` | Итератор `String`, паникует на невалидном UTF-8 |
| `env::args_os()` | Итератор `OsString`, безопасен для любых байтов |
| `.skip(1)` | Пропускает имя программы |
| `args.next()` | Получает следующий аргумент |
| `arg.to_str().unwrap_or_default()` | `OsString` → `&str` (или `""`) |
| `match` | Сопоставление с флагами |
| `\|` | Альтернатива (короткий/длинный флаг) |
| `args.by_ref()` | Не потреблять итератор, чтобы использовать его дальше |
| `.map(PathBuf::from)` | `OsString` → `PathBuf` |
| `.extend(...)` | Добавить все элементы итератора в коллекцию |
| `break` | Прекратить разбор флагов после `-p` |
| `_ => {}` | Игнорировать неизвестные аргументы |

## Особенности и потенциальные проблемы

### 1. `-p` завершает разбор

После `-p` все оставшиеся аргументы считаются путями. Это значит, что **нельзя** смешивать флаги и пути: `-p file.txt -l` — `-l` будет воспринят как имя файла.

### 2. `to_str().unwrap_or_default()`

Если аргумент — невалидный UTF-8, он превращается в `""` и **игнорируется** (ветка `_`). Это безопасно, но невалидный флаг не будет обработан.

### 3. `args.by_ref()` критичен

Без `by_ref()` итератор был бы **перемещён** в `map`, и `while let Some(arg) = args.next()` в следующей итерации не скомпилировался бы. `by_ref()` берёт **ссылку**, оставляя владение у `args`.

### 4. Баг в подсчёте слов

```rust
content.lines()
    .map(|line| line.split_whitespace().count())
    .count()
```

Это считает **количество строк**, а не слов. Правильно:

```rust
content.split_whitespace().count()
```

### 5. Нет поддержки `--`

Стандартное соглашение: после `--` все аргументы считаются позиционными. Здесь этого нет — можно добавить:

```rust
"--" => {
    paths.extend(args.by_ref().map(PathBuf::from));
    break;
}
```

## Итог

- Аргументы получаются через `env::args_os()` (безопасно для путей).
- `.skip(1)` пропускает имя программы.
- `while let Some(arg) = args.next()` перебирает аргументы.
- `match` разбирает флаги: `-l`/`--lines`, `-w`/`--words`, `-p`/`--paths`.
- `-p` забирает все оставшиеся аргументы как пути через `args.by_ref().map(PathBuf::from)` и завершает разбор (`break`).
- Неизвестные аргументы игнорируются (`_ => {}`).
- Разобранные флаги и пути используются в `stat_file` для подсчёта строк и слов.
- В коде есть **баг** в подсчёте слов (считает строки, а не слова).
