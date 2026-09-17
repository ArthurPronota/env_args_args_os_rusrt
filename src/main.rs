/*
Запуск:
cargo run -- -l -w -p Cargo.toml Cargo.lock
*/
use std::env ;
use std::ffi::OsString;
use std::path::PathBuf ;
use std::fs ;

#[derive(Debug)]
// аргументы командной строки
struct CliArgs {
    lines:  bool,   // нужны данные по количеству строк
    words:  bool,   // нужны данные по количеству слов
    paths:  Vec<PathBuf> // пути к файлам для анализа
}

impl CliArgs {
    fn parse() ->Self {

        let mut lines = false ;
        let mut words = false ;
        let mut paths = Vec::<PathBuf>::new() ;

        let mut args = 
                env::args_os()
                    .skip(1) // Создаёт итератор что пропускает первые n элементов.
                    ;
        
        while let Some(arg) = args.next() {
            match arg.to_str().unwrap_or_default() {
                "-l" | "--lines" => lines = true,
                "-w" | "--words" => words = true,
                "-p" | "--paths" => {
                    //let v = 
                    paths
                        .extend( // Расширяет коллекцию с содержимым итератора.
                        args
                            // Создает адаптер «по ссылке» для данного экземпляра Iterator.
                            .by_ref()
                            .map(
                                PathBuf::from   // Конвертирует OsString в PathBuf.
                            )
                        ) ;
                    break;
                }
                _ => {},
            }
        }

        Self {
            lines,
            words,
            paths,
        }
    }
}

// статистика для файла
struct Stats<'a> {
    filepath:   &'a PathBuf,    // &'a Path,
    lines:      Option<usize>,
    words:      Option<usize>,
}

fn stat_file(cli_arg: &CliArgs) -> Vec<Stats<'_>> {
    let mut stat_collection = Vec::<Stats>::new() ;

    if !cli_arg.lines && !cli_arg.words { // не нужныданные по lines и words
        return stat_collection ;
    }

    for filepath in &cli_arg.paths {

        if !filepath.is_file() {    // это не файл
            continue ;
        }

        let Ok(content) = fs::read_to_string(filepath) else {
            continue ;
        } ;

        // определение lines
        let lines = match cli_arg.lines {
            true => Some(content.lines().count()),
            false => None,
        } ;

        let words = match cli_arg.words {
            true => Some(
                content
                    .lines()
                    .map(|line| line.split_whitespace().count())
                    //.count()  // 20260917 неверне значение с исхожнике !!!
                    .sum()  // 20260917 верное значение !!!
            ),
            false => None,
        } ;

        stat_collection.push(
            Stats { filepath, lines, words }
        );

    }

    stat_collection
}

fn main() {

    let args = 
            env::args() // Args Implements notable traits: Iterator<Item = String>
                .collect::<Vec<String>>() ;

    println!("args: {args:?}") ; // Out: args: ["target\\debug\\c_0010_cli.exe", "-l", "-w", "-p", "Cargo.toml", "Cargo.lock"]

    // Skip the program name, it's usually the first argument
    println!("skip(1) for args: {:?}", 
            env::args().skip(1).collect::<Vec<String>>()
        ) ;  // Out: skip(1) for args:  ["-l", "-w", "-p", "Cargo.toml", "Cargo.lock"]

    // Некоторые документы, например, пути файлов, могут быть 
    // недействительными UTF-8. Используйте args_os для безопасного 
    // обращения с ними:
    let os_args = 
            env::args_os() // Implements notable traits: Iterator<Item = OsString>
                .collect::<Vec<OsString>>() ;

    println!("os_args: {os_args:?}") ; // Out: os_args: ["target\\debug\\c_0010_cli.exe", "-l", "-w", "-p", "Cargo.toml", "Cargo.lock"]

    // Расчет статистики файлов

    let cli_args = CliArgs::parse() ;
    let stats = stat_file(&cli_args) ;

    println!("cli_args: {:?}", cli_args) ;  // Out: cli_args: CliArgs { lines: true, words: true, paths: ["Cargo.toml", "Cargo.lock"] }

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
        ) ; // Out: Path: "Cargo.toml" Lines: 6 Words: 6
            //      Path: "Cargo.lock" Lines: 7 Words: 7
    }
}
