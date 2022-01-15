use std::io::{BufRead, BufReader};
use std::env;
use std::fs::File;
use std::collections::HashMap;

/*
ANSI 256color to true-color
===========================
 NAME       FG/BG     Rgb
---------------------------
white      37m/47m   c5c8c6
red        31m/41m   cc6666
green      32m/42m   b5bd68
yellow     33m/43m   f0c674
blue       34m/44m   81a2be
magenta    35m/45m   b294bb
cyan       36m/46m   8abe87
gray       90m/100m  666666
Br white   97m/107m  eaeaea
Br red     91m/101m  d54e53
Br green   92m/102m  b9ca4a
Br yellow  93m/103m  e7c547
Br blue    94m/104m  7aa6da
Br magenta 95m/105m  c397d8
Br cyan    96m/106m  70c0b1
===========================
*/

#[derive(Debug, Copy, Clone)]
struct Rgb {
    r: u32,
    g: u32,
    b: u32
}

fn get_grad(start: &Rgb, end: &Rgb, steps: u32) -> Vec<Rgb> {
    // The number of colors to compute
    let len = steps;

    // Alpha blending amount
    let mut alpha = 0.0;

    let mut grad: Vec<Rgb> = Vec::new();

    for _i in 0..len {
        let red: f32;
        let green: f32;
        let blue: f32;
        alpha = alpha + (1.0 / len as f32);

        red = end.r as f32 * alpha + (1.0 - alpha) * start.r as f32;
        green = end.g as f32 * alpha + (1.0 - alpha) * start.g as f32;
        blue = end.b as f32 * alpha + (1.0 - alpha) * start.b as f32;

        let rgb = Rgb {
            r: red as u32,
            g: green as u32,
            b: blue as u32
        };
        grad.push(rgb)
    }
    return grad;
}

fn main() {
    // Table for true-color gradation
    // (back_start_color.r, back_start_color.g, back_start_color.b, back_end_color.r, back_end_color.g, back_end_color.b, fore_color.r, fore_color.g, fore_color.b)
    let grad_table =
        [(0x70, 0xc0, 0xb1, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xc3, 0x97, 0xd8, 0xc5, 0xc8, 0xc6 ,0x3c, 0x3e, 0x3f),
         (0x7a, 0xa6, 0xda, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xe7, 0xc5, 0x47, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xb9, 0xca, 0x4a, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xd5, 0x4e, 0x53, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0x8a, 0xbe, 0x87, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f)];

    let mut grad_idx = 0;

    let mut exec_name: String = String::from("");
    let mut revlist_filename: String = String::from("");
    let mut blame_filename: String = String::from("");
    let mut bat_filename: String = String::from("");

    // parse argument
    let mut idx_mode = false;
    for arg in env::args() {
        if idx_mode {
            match arg.parse::<usize>() {
                Ok(i) => grad_idx = i,
                Err(_) => grad_idx = 0
            }
            if grad_idx >= grad_table.len() {
                grad_idx = 0;
            }
            idx_mode = false;
            continue;
        }
        if arg == "-g" || arg == "--g" {
            idx_mode = true
        } else {
            if exec_name == "" {
                exec_name = arg.clone();
            } else if revlist_filename == "" {
                revlist_filename = arg.clone();
            } else if blame_filename == "" {
                blame_filename = arg.clone();
            } else if bat_filename == "" {
                bat_filename = arg.clone();
            }
        }
    }

    if revlist_filename == "" || blame_filename == "" || bat_filename == "" {
        return;
    }
    let revlist_file = File::open(revlist_filename);
    let blame_file = File::open(blame_filename);
    let bat_file = File::open(bat_filename);

    // load revlist file
    let mut revlist_map = HashMap::new();
    match revlist_file {
        Err(_) => return,
        Ok(file) => {
            let reader = BufReader::new(file);
            for (index, ln) in reader.lines().enumerate() {
                let line;
                match ln {
                    Ok(data) => line = data,
                    Err(_) => continue
                }
                revlist_map.insert(line, index as usize);
            }
        }
    }

    let fore_color = Rgb {
        r: grad_table[grad_idx].6,
        g: grad_table[grad_idx].7,
        b: grad_table[grad_idx].8
    };
    let back_start_color = Rgb {
        r: grad_table[grad_idx].0,
        g: grad_table[grad_idx].1,
        b: grad_table[grad_idx].2
    };
    let back_end_color = Rgb {
        r: grad_table[grad_idx].3,
        g: grad_table[grad_idx].4,
        b: grad_table[grad_idx].5
    };
    let grad = get_grad(&back_start_color, &back_end_color, revlist_map.len() as u32);

    // load bat file
    let mut bat_lines = vec![];
    match bat_file {
        Err(_) => return,
        Ok(file) => {
            let reader = BufReader::new(file);
            for ln in reader.lines() {
                let line;
                match ln {
                    Ok(data) => line = data,
                    Err(_) => continue
                }
                bat_lines.push(line);
            }
        }
    }

    let line_number_digits = ((bat_lines.len() as f64).log10() + 1.0) as usize;

    // load blame file and process each line
    match blame_file {
        Err(_) => return,
        Ok(file) => {
            let reader = BufReader::new(file);
            for (index, ln) in reader.lines().enumerate() {
                let line;
                match ln {
                    Ok(data) => line = data,
                    Err(_) => continue
                }

                // split change-number and hash-value
                let change_number;
                let hash;
                match line.rfind(' ') {
                    Some(found) => {change_number = &line[..found]; hash = &line[found+1..]},
                    None => {change_number = ""; hash = ""}
                }

                // get matching index from hash value
                let matching_idx;
                match revlist_map.get(hash) {
                    Some(found) => matching_idx = *found,
                    None => matching_idx = revlist_map.len() - 1
                }

                // get current gradation color from matching index. default is back_end_color
                let back_color;
                match grad.get(matching_idx) {
                    Some(color) => back_color = color,
                    None => back_color = &back_end_color
                }

                let line_number = format!("{:>width$}", index + 1, width = line_number_digits);
                if hash == "" {
                    println!("│\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{} {}\x1b[0m│{}", fore_color.r, fore_color.g, fore_color.b, back_color.r, back_color.g, back_color.b, line, line_number, bat_lines[index]);
                } else {
                    println!("│\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}:{} {}\x1b[0m│{}", fore_color.r, fore_color.g, fore_color.b, back_color.r, back_color.g, back_color.b, change_number, hash, line_number, bat_lines[index]);
                }
            }
        }
    }
}
