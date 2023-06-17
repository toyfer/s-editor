use std::env;
use std::fs;
use std::io::{self, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).map_or("", String::as_str);

    let mut contents = String::new();
    if !filename.is_empty() {
        contents = fs::read_to_string(filename).expect("Unable to read file");
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    let mut cursor_x = 1;
    let mut cursor_y = 1;
    let mut lines = contents.lines().map(|s| s.to_string()).collect::<Vec<String>>();
    let mut dirty = false;

    write!(stdout, "{}", termion::clear::All).unwrap();

    for line in &lines {
        write!(stdout, "{}{}\r\n", line, termion::cursor::Down(1)).unwrap();
    }

    write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('\n') => {
                let line = lines.get_mut(cursor_y - 1).unwrap();
                let (left, right) = line.split_at_mut(cursor_x - 1);
                *line = format!("{}{}\r", left, right);
                lines.insert(cursor_y, String::from(&line[cursor_x - 1..]));
                *line = String::from(&line[..cursor_x - 1]);
                cursor_y += 1;
                cursor_x = 1;
                dirty = true;
            }
            Key::Char(c) => {
                let line = lines.get_mut(cursor_y - 1).unwrap();
                let (left, right) = line.split_at_mut(cursor_x - 1);
                *line = format!("{}{}{}{}", left, c, right, termion::cursor::Right(1));
                cursor_x += 1;
                dirty = true;
            }
            Key::Backspace => {
                if cursor_x > 1 {
                    let line = lines.get_mut(cursor_y - 1).unwrap();
                    let (left, right) = line.split_at_mut(cursor_x - 2);
                    *line = format!("{}{}", left, &right[1..]);
                    cursor_x -= 1;
                    dirty = true;
                } else if cursor_y > 1 {
                    let current_line = lines.remove(cursor_y - 1);
                    let prev_line = lines.get_mut(cursor_y - 2).unwrap();
                    *prev_line = format!("{}{}", prev_line, current_line);
                    cursor_y -= 1;
                    cursor_x = prev_line.len() + 1;
                    dirty = true;
                }
            }
            Key::Left => {
                if cursor_x > 1 {
                    cursor_x -= 1;
                    write!(stdout, "{}", termion::cursor::Left(1)).unwrap();
                }
            }
            Key::Right => {
                if cursor_x <= lines[cursor_y - 1].len() {
                    cursor_x += 1;
                    write!(stdout, "{}", termion::cursor::Right(1)).unwrap();
                }
            }
            Key::Up => {
                if cursor_y > 1 {
                    cursor_y -= 1;
                    let line = &lines[cursor_y - 1];
                    if cursor_x > line.len() + 1 {
                        cursor_x = line.len() + 1;
                    }
                    write!(stdout, "{}", termion::cursor::Up(1)).unwrap();
                }
            }
            Key::Down => {
                if cursor_y < lines.len() {
                    cursor_y += 1;
                    let line = &lines[cursor_y - 1];
                    if cursor_x > line.len() + 1 {
                        cursor_x = line.len() + 1;
                    }
                    write!(stdout, "{}", termion::cursor::Down(1)).unwrap();
                }
            }
            Key::Ctrl('s') => {
                if dirty {
                    let mut f = if filename.is_empty() {
                        let mut filename = String::new();
                        print!("Enter filename: ");
                        io::stdout().flush().unwrap();
                        io::stdin().read_line(&mut filename).unwrap();
                        let filename = filename.trim();
                        fs::File::create(filename).expect("Unable to create file")
                    } else {
                        fs::File::create(filename).expect("Unable to create file")
                    };

                    for line in &lines {
                        writeln!(f, "{}", line).expect("Unable to write to file");
                    }

                    dirty = false;
                }
            }
            Key::Ctrl('q') => {
                if dirty {
                    let mut input = String::new();
                    print!("File has unsaved changes. Do you want to save before quitting? (y/n): ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim().to_lowercase();
                    if input == "y" || input == "yes" {
                        let mut f = if filename.is_empty() {
                            let mut filename = String::new();
                            print!("Enter filename: ");
                            io::stdout().flush().unwrap();
                            io::stdin().read_line(&mut filename).unwrap();
                            let filename = filename.trim();
                            fs::File::create(filename).expect("Unable to create file")
                        } else {
                            fs::File::create(filename).expect("Unable to create file")
                        };

                        for line in &lines {
                            writeln!(f, "{}", line).expect("Unable to write to file");
                        }
                    }
                }
                break;
            }
            _ => {}
        }

        write!(
            stdout,
            "{}",
            termion::cursor::Goto(cursor_x as u16, cursor_y as u16)
        )
        .unwrap();
        stdout.flush().unwrap();
    }
}