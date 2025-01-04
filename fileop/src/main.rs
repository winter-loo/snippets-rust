use std::collections::{BTreeMap, HashMap};

fn main() {
    // step 1: read types_default
    let var_init: String = std::fs::read_to_string("./var_init").expect("file var_init");

    let types_default: std::collections::HashMap<&str, &str> = var_init
        .split('\n')
        .filter(|s| s.len() != 0)
        .map(|s| {
            let parts: Vec<&str> = s.split_whitespace().collect();
            (parts[0], parts[1])
        })
        .collect();

    // println!("{types_default:#?}");

    // step 2: read var
    let deal = std::fs::read_to_string("./var").expect("file var");

    let new_deal: HashMap<_, Vec<(_, _)>> = deal
        .split('\n')
        .filter(|s| s.len() != 0)
        .map(|s| {
            let parts: Vec<_> = s.split(":").collect();
            let line = parts[2];
            let new_line = if line.contains("*") {
                // pointer
                Some(line.replace(";", " = NULL;"))
            } else if line.contains("[") {
                // array
                Some(line.replace(";", " = {};"))
            } else {
                // others
                let key = line.split_whitespace().collect::<Vec<_>>()[0];
                if let Some(value) = types_default.get(&key) {
                    let value = format!(" = {};", value);
                    Some(line.replace(";", &value))
                } else {
                    None
                }
            };
            (parts[0], (parts[1].parse::<usize>().unwrap(), new_line))
        })
        .fold(HashMap::new(), |mut acc, (key, value)| {
            acc.entry(key).or_insert_with(Vec::new).push(value);
            acc
        });

    // println!("{new_deal:#?}");

    // step 3: modify file content
    new_deal.iter().for_each(|(file, updates)| {
        update_file(&file, &updates);
    });
}

fn update_file(filename: &str, updates: &Vec<(usize, Option<String>)>) {
    let file_content = std::fs::read_to_string(filename).expect(&format!("file {filename}"));
    // println!("{filename} -> {file_content:#?}");

    let mut file_content: BTreeMap<_, _> = file_content
        .split('\n')
        .enumerate()
        .map(|(idx, s)| (idx + 1, s))
        .collect();

    // println!("{file_content:#?}");

    let mut changed = false;
    updates.iter().for_each(|(linu, new_line)| {
        file_content.entry(*linu).and_modify(|s| {
            if !s.contains("=") {
                if let Some(new_line) = new_line {
                    *s = new_line;
                    changed = true;
                }
            }
        });
    });

    if changed {
        let mut new_file_content: String = file_content
            .iter()
            .map(|(_, line)| line.to_string() + "\n")
            .collect();
        new_file_content.remove(new_file_content.len() - 1);
        // println!("new {filename} -> {new_file_content:#?}");

        std::fs::write(filename, new_file_content).expect(&format!("write file {filename}"));
    }
}
