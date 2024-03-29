mod config;
mod datetime;
pub mod memory;
mod terminal;

use alloc::boxed::Box;

use git_version::git_version;
use indoc::indoc;

use crate::{
    io::{self, Read},
    logger,
};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const REVISION: &'static str = git_version!();
const PROMPT: &'static str = "cli> ";

pub struct FnCommand {
    name: &'static str,
    description: &'static str,
    action: fn(&str),
}

extern "Rust" {
    fn board_name() -> &'static str;
    fn heap_statistics();
    pub fn reboot();
}

macro_rules! __builtin_commands {
    ($(($name:literal, $description: literal, $action:expr)),+) => {
        [$(FnCommand{name: $name, description: $description, action: $action}),+]
    };
}

const BUILTIN_CMDS: [FnCommand; 15] = __builtin_commands!(
    ("date", "Show date", |line| datetime::date(line)),
    ("dump", "Dump memory address", |line| memory::dump(line)),
    ("free", "Show memory allocation statistics", |_| unsafe { heap_statistics() }),
    ("logread", "Read log", |_| print!("{}", logger::get())),
    ("read", "Read memory address", |line| memory::read(line)),
    ("readx", "Read memory address in hex", |line| memory::readx(line)),
    ("writex", "Write memory address in hex", |line| memory::writex(line)),
    ("import", "Import config", |line| config::import(line)),
    ("export", "Export config", |line| config::export(line)),
    ("set", "Set config entry", |line| config::set(line)),
    ("clear", "Clear config entry", |line| config::clear(line)),
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
    action: Box<dyn FnMut(&str) + Send + 'static>,
}

impl Command {
    pub fn new<A>(name: &'static str, desc: &'static str, action: A) -> Self
    where
        A: FnMut(&str) + Send + 'static,
    {
        Self { name, description: desc, action: Box::new(action) }
    }
}

pub const OSD_CMD_USAGE: &str = indoc! {"
Usage:
    osd upload-font: Upload font
"};

#[macro_export]
macro_rules! __command {
    (bootloader,[$persist:ident]) => {
        $crate::cli::Command::new("bootloader", "Reboot to bootloader", move |_| {
            let mut sysinfo: $crate::sysinfo::SystemInfo = $persist.load();
            sysinfo.reboot_reason = RebootReason::Bootloader;
            $persist.save(&sysinfo);
            unsafe { $crate::cli::reboot() };
        })
    };
    (osd,[$event:ident]) => {
        $crate::cli::Command::new("osd", "OSD related command", move |cmd| {
            use $crate::sync::event::{Notifier as _, Subscriber as _};
            if cmd.trim() != "upload-font" {
                println!("{}", $crate::cli::OSD_CMD_USAGE);
                return;
            }
            $event.notify();
            while $event.wait() {
                $crate::sys::time::TickTimer::default().delay_ms(1u32);
            }
        })
    };
    (save,[$nvram:ident]) => {
        $crate::cli::Command::new("save", "Save configuration", move |_| {
            if let Some(err) = $nvram.store(config::get()).err() {
                println!("Save configuration failed: {:?}", err);
                $nvram.reset().ok();
            }
        })
    };
    (telemetry,[]) => {
        $crate::cli::Command::new("telemetry", "Show flight data", move |_| {
            let ds = $crate::datastore::acquire();
            println!("{}", $crate::collection::Collector::new(ds).collect())
        })
    };
}

#[macro_export]
macro_rules! commands {
    ($(($name:ident, $args:tt)),+) => {
        [$(__command!($name, $args)),+]
    };
}

pub struct CLI<CMDS> {
    terminal: terminal::Terminal,
    commands: CMDS,
}

fn prompt() {
    print!("\r{}", PROMPT);
}

impl<CMDS: AsMut<[Command]>> CLI<CMDS> {
    pub fn new(commands: CMDS) -> Self {
        CLI { terminal: terminal::Terminal::new(), commands }
    }

    pub fn run(&mut self) {
        let mut buffer = [0u8; 80];
        let size = io::stdin().read(&mut buffer[..]).ok().unwrap_or(0);
        let line = match self.terminal.receive(&buffer[..size]) {
            Some(line) => line.trim(),
            None => return,
        };
        if line.starts_with('#') || line.trim() == "" {
            return prompt();
        }
        let (cmd, operand) =
            line.split_once(' ').map(|(a, b)| (a, b.trim_start())).unwrap_or((line, ""));
        match cmd {
            "" => return prompt(),
            "help" => {
                for command in BUILTIN_CMDS.iter() {
                    println!("{:<10} {}", command.name, command.description);
                }
                for command in self.commands.as_mut().iter() {
                    println!("{:<10} {}", command.name, command.description);
                }
                return prompt();
            }
            _ => {}
        };
        if let Some(cmd) = BUILTIN_CMDS.iter().find(|c| c.name == cmd) {
            (cmd.action)(operand);
        } else if let Some(cmd) = self.commands.as_mut().iter_mut().find(|c| c.name == cmd) {
            (cmd.action)(operand);
        } else {
            println!("Unknown command: {}", cmd);
        }
        prompt()
    }
}
