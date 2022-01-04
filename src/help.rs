struct Help<'a> {
    text: &'a str,
    description: &'a str,
}

fn print_help(helps: &[Help]) {
    let text_width = helps
        .iter()
        .max_by_key(|c| c.text.len())
        .unwrap()
        .text
        .len();
    let desc_width = helps
        .iter()
        .max_by_key(|c| c.description.len())
        .unwrap()
        .description
        .len();
    helps.iter().for_each(|c| {
        println!(
            "{:<tw$}  {:<dw$}",
            c.text,
            c.description,
            tw = text_width,
            dw = desc_width
        )
    });
}

pub fn help() {
    let commands: Vec<Help> = vec![
        Help {
            text: "exit",
            description: "Exit from the launcher",
        },
        Help {
            text: "quit",
            description: "Exit from the launcher",
        },
        Help {
            text: "list",
            description: "Show profiles, configs and statuses",
        },
        Help {
            text: "set <profile> <mode>",
            description: "Set mode",
        },
        Help {
            text: "shutdown",
            description: "Shutdown all firefoxes",
        },
    ];
    let modes = vec![
        Help {
            text: "auto",
            description: "Auto mode",
        },
        Help {
            text: "on",
            description: "Always on",
        },
        Help {
            text: "off",
            description: "Always off",
        },
        Help {
            text: "suspend",
            description: "Always suspend",
        },
        Help {
            text: "asis",
            description: "Leave it as is",
        },
    ];
    print_help(&commands);
    println!();
    println!("modes");
    println!("----------------------");
    print_help(&modes);
}
