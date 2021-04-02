mod config;
mod datetime;
pub mod memory;
mod terminal;

use alloc::boxed::Box;

use git_version::git_version;

use crate::components::logger;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const REVISION: &'static str = git_version!();
const PROMPT: &'static str = "cli> ";

pub struct FnCommand {
    name: &'static str,
    description: &'static str,
    action: fn(&str),
}

extern "Rust" {
    pub fn reboot();
    fn board_name() -> &'static str;
}

macro_rules! __builtin_commands {
    ($(($name:literal, $description: literal, $action:expr)),+) => {
        [$(FnCommand{name: $name, description: $description, action: $action}),+]
    };
}

const BUILTIN_CMDS: [FnCommand; 13] = __builtin_commands!(
    ("date", "Show date", |line| datetime::date(line)),
    ("dump", "Dump memory address", |line| memory::dump(line)),
    ("logread", "Read log", |_| print!("{}", logger::get())),
    ("read", "Read memory address", |line| memory::read(line)),
    ("readx", "Read memory address in hex", |line| memory::readx(line)),
    ("writex", "Write memory address in hex", |line| memory::writex(line)),
    ("import", "Import config", |line| config::import(line)),
    ("export", "Export config", |line| config::export(line)),
    ("set", "Set config entry", |line| config::set(line)),
    ("reboot", "Reboot", |_| unsafe { reboot() }),
    ("reset", "Reset config", |_| config::reset()),
    ("show", "Show config", |_| config::show()),
    ("version", "Get version", |_| {
        println!("board: {}", unsafe { board_name() });
        println!("version: {}-{}", VERSION, REVISION);
    })
);

pub struct Command {
    name: &'static str,
    description: &'static str,
    action: Box<dyn FnMut(&str)>,
}

impl Command {
    pub fn new(name: &'static str, desc: &'static str, action: impl FnMut(&str) + 'static) -> Self {
        Self { name, description: desc, action: Box::new(action) }
    }
}

#[macro_export]
macro_rules! __command {
    (bootloader, [$persist:ident]) => {
        $crate::components::cli::Command::new("bootloader", "Reboot to bootloader", move |_| {
            let mut sysinfo: $crate::sysinfo::SystemInfo = $persist.load();
            sysinfo.reboot_reason = RebootReason::Bootloader;
            $persist.save(&sysinfo);
            unsafe { $crate::components::cli::reboot() };
        })
    };
    (telemetry, [$reader:ident]) => {
        $crate::components::cli::Command::new("telemetry", "Show flight data", move |_| {
            println!("{}", $reader.read())
        })
    };
    (save, [$nvram:ident]) => {
        $crate::components::cli::Command::new("save", "Save configuration", move |_| {
            if let Some(err) = $nvram.store(config::get()).err() {
                println!("Save configuration failed: {:?}", err);
                $nvram.reset().ok();
            }
        })
    };
}

#[macro_export]
macro_rules! commands {
    ($(($name:ident, $args:tt)),+) => {
        [$(__command!($name, $args)),+]
    };
}

pub struct CLI<'a> {
    terminal: terminal::Terminal,
    commands: &'a mut [Command],
}

fn prompt() {
    print!("\r{}", PROMPT);
}

impl<'a> CLI<'a> {
    pub fn new(commands: &'a mut [Command]) -> Self {
        CLI { terminal: terminal::Terminal::new(), commands }
    }

    pub fn receive(&mut self, bytes: &[u8]) {
        let line = match self.terminal.receive(bytes) {
            Some(line) => line,
            None => return,
        };
        if line.starts_with('#') {
            return prompt();
        }
        let mut split = line.splitn(2, ' ');
        let first_word = match split.next() {
            Some(word) => word,
            None => return prompt(),
        };
        let remain = split.next().unwrap_or("");
        match first_word {
            "" => return prompt(),
            "help" => {
                for command in BUILTIN_CMDS.iter() {
                    println!("{}\t\t{}", command.name, command.description);
                }
                for command in self.commands.iter() {
                    println!("{}\t\t{}", command.name, command.description);
                }
            }
            _ => {}
        };
        if let Some(cmd) = BUILTIN_CMDS.iter().find(|cmd| cmd.name == first_word) {
            (cmd.action)(remain);
        } else if let Some(cmd) = self.commands.iter_mut().find(|cmd| cmd.name == first_word) {
            (cmd.action)(remain);
        } else {
            println!("Unknown command: {}", first_word);
        }
        print!("\r{}", PROMPT);
    }
}
