use colored::*;
use copypasta::{ClipboardContext, ClipboardProvider};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use rusqlite::{params, Connection, Result};
use std::io::Error;
use std::net::IpAddr;
use std::process::Command;
use std::thread;
use tabled::{Style, Table, Tabled};

pub const TABLE: &str = "server_connections";
const SSH: &str = "ssh";
const DEFAULT_SSH_COMMAND: &str = "hostname";
pub const CHOICES: [&str; 9] = [
    "Display all server connections",
    "Start a connection",
    "Create a server connection",
    "Delete a server connection",
    "Purge database (delete all server connections)",
    "Create SSH key",
    "Issue SSH command to multiple servers",
    "Check server status (online/offline)",
    "Exit",
];

#[derive(Debug, Tabled)]
pub struct ServerConnection {
    id: u8,
    user: String,
    ip: String,
    key_path: String,
    port: String,
    pub alias: String,
}

impl ServerConnection {
    pub fn get_command(&self, cmd: &str) -> String {
        format!(
            "{} -i {} -p {} {}@{}",
            cmd, self.key_path, self.port, self.user, self.ip
        )
    }
}

///Set clipboard (control + v)
fn set_clipboard(content: &str) {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(content.to_string().to_owned()).unwrap();
    ctx.get_contents().unwrap();
}

///Create database table if not exists
pub fn create_database(conn: &Connection) -> Result<()> {
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {TABLE} (
                  id              INTEGER PRIMARY KEY,
                  user           VARCHAR(255) NOT NULL,
                  ip          VARCHAR(255) NOT NULL,
                  key_path      VARCHAR(255) NOT NULL,
                  port      VARCHAR(4) NOT NULL,
                  alias      VARCHAR(255) NOT NULL
                  )"
        ),
        [],
    )?;
    Ok(())
}

///Delete database records
pub fn delete_records_by_alias(conn: &Connection, alias: &str) -> Result<()> {
    conn.execute(
        &format!("DELETE FROM {TABLE} WHERE alias = ?1"),
        params![alias],
    )?;
    Ok(())
}

///Delete all database records
pub fn purge_database(conn: &Connection) -> Result<()> {
    if get_user_confirmation("Are you sure you want to delete all server connections") {
        conn.execute(&format!("DELETE FROM {TABLE}"), params![])?;
    }
    Ok(())
}

///Create server connection on database
pub fn create_server_connection(conn: &Connection) -> Result<()> {
    let user = get_user_input("User", env!("USER"));
    let ip = get_user_input("IP", "0.0.0.0");

    match ip.parse::<IpAddr>() {
        Ok(_) => {
            let key_path = get_user_input("Public key path", ".");
            let port = get_user_input("Port", "22");
            let alias = get_user_input("Alias", "sample");

            let record = ServerConnection {
                id: 0,
                user,
                ip,
                key_path,
                port,
                alias,
            };

            conn.execute(
                &format!("INSERT INTO {TABLE} (user, ip, key_path, port, alias) VALUES (?1, ?2, ?3, ?4, ?5)"),
                params![record.user, record.ip, record.key_path, record.port, record.alias],
            )?;
            let msg = format!("Server Connection created: {}", &record.alias);
            display_message("OK", &msg, "green")
        }
        Err(_) => {
            let msg = format!("Invalid IP: {ip}");
            display_message("ERROR", &msg, "red")
        }
    }
    Ok(())
}

///Display all database connections as a table
pub fn display_connections(records: &Vec<ServerConnection>) {
    let table = Table::new(records).with(Style::modern()).to_string();
    println!("{table}\nQuantity: {}\n", records.len());
}

///Get all database connections from database
pub fn get_connections(conn: &Connection) -> Result<Vec<ServerConnection>> {
    let mut records: Vec<ServerConnection> = Vec::new();
    let query = format!("SELECT * FROM {TABLE}");
    let mut stmt = conn.prepare(&query)?;

    let result_iter = stmt.query_map([], |row| {
        Ok(ServerConnection {
            id: row.get(0)?,
            user: row.get(1)?,
            ip: row.get(2)?,
            key_path: row.get(3)?,
            port: row.get(4)?,
            alias: row.get(5)?,
        })
    })?;

    for i in result_iter {
        records.push(i?);
    }
    Ok(records)
}

fn run_cmd(cmd: &str, arguments: &[&str]) -> Result<(), Error> {
    Command::new(cmd).args(arguments).spawn()?;
    Ok(())
}

///Run SSH commands
fn run_commands(commands: Vec<String>) {
    let handle = thread::spawn(|| {
        for i in commands {
            let ssh_command = i.split_once("ssh ").unwrap().1;
            let alias = i.split_once("alias").unwrap().0;
            let ssh_command: Vec<&str> = ssh_command.split_whitespace().collect();

            match run_cmd("ssh", &ssh_command) {
                Ok(_) => display_message(
                    "ok",
                    format!("SSH command sent to {alias}").as_str(),
                    "green",
                ),
                Err(e) => display_message("error", &e.to_string(), "red"),
            };
        }
    });

    handle.join().unwrap();
}

fn create_command(ssh_command: &str, server_connection: &ServerConnection) -> String {
    let mut full_command = String::from(&server_connection.alias);
    full_command.push_str("alias");
    full_command.push_str(server_connection.get_command("ssh").as_str());
    full_command.push_str(" ");
    full_command.push_str(&ssh_command);
    full_command.to_string()
}

///Issue SSH command to multiple servers
pub fn issue_command(records: &Vec<ServerConnection>) -> Result<()> {
    let mut command_list: Vec<String> = Vec::new();
    let ssh_command = get_user_input("SSH command", &DEFAULT_SSH_COMMAND);

    match get_user_confirmation("All servers") {
        true => {
            for i in records {
                command_list.push(create_command(&ssh_command, &i));
            }
            run_commands(command_list);
            return Ok(());
        }
        false => {
            let records_aliases = records.iter().map(|x| x.alias.as_str()).collect();
            let aliases_list = get_user_multi(&records_aliases, "Connections");
            for i in aliases_list {
                for r in records {
                    if r.alias == i {
                        command_list.push(create_command(&ssh_command, &r));
                    }
                }
            }
        }
    }
    run_commands(command_list);
    Ok(())
}

///Start SSH/SFTP connection
pub fn start_connection(records: &Vec<ServerConnection>) -> Result<()> {
    let records_aliases = records.iter().map(|x| x.alias.as_str()).collect();
    let (server_alias, _) = get_user_selection(&records_aliases, "Connection");
    let mut connection_type = get_user_input("Connection type (1 for SFTP)", &SSH);

    if &connection_type != &SSH {
        connection_type = "sftp".to_string();
    }

    for i in records {
        if i.alias == server_alias {
            let command = &i.get_command(&connection_type);
            set_clipboard(&command);
            let msg = format!("[OK] Sent to clipboad: {}", &command).color("green");
            println!("{msg}");
            break;
        }
    }
    Ok(())
}

///Display message
pub fn display_message(message_type: &str, message: &str, color: &str) {
    let msg = format!("[{}] {}", message_type.to_uppercase(), message).color(color);
    println!("\n{msg}\n");
}

///Create SSH key on current directory
pub fn create_key() -> Result<()> {
    let name = get_user_input("SSH key name", "sample");

    match Command::new("ssh-keygen")
        .args([
            "-a", "100", "-t", "ed25519", "-f", &name, "-C", &name, "-N", "''", "-q",
        ])
        .spawn()
    {
        Ok(_) => {
            let msg = format!("SSH key created: {}", &name);
            display_message("OK", &msg, "green");
        }
        Err(_) => display_message("ERROR", "Error creating SSH key", "red"),
    }
    Ok(())
}

fn ping_server(ip: &str) -> bool {
    let cmd = Command::new("ping").args(["-c", "2", ip, "-q"]).spawn();
    cmd.unwrap().wait_with_output().unwrap().status.success()
}

///Ping to check whether server is online
pub fn check_server_status(records: &Vec<ServerConnection>) -> Result<()> {
    let mut online_servers: Vec<&str> = Vec::new();
    let mut offline_servers: Vec<&str> = Vec::new();

    match get_user_confirmation("Check all servers") {
        true => {
            for i in records {
                match ping_server(&i.ip) {
                    true => online_servers.push(&i.alias),
                    false => offline_servers.push(&i.alias),
                }
            }
        }
        false => {
            let records_aliases = records.iter().map(|x| x.alias.as_str()).collect();
            let aliases_list = get_user_multi(&records_aliases, "Servers to check");

            for i in aliases_list {
                for r in records {
                    if r.alias == i {
                        match ping_server(&r.ip) {
                            true => online_servers.push(&r.alias),
                            false => offline_servers.push(&r.alias),
                        }
                    }
                }
            }
        }
    }
    if online_servers.len() > 0 {
        let online_msg = format!(
            "Online servers: {:?}. Quantity: {}",
            online_servers,
            online_servers.len(),
        );
        display_message("✅", &online_msg, "green");
    }
    if offline_servers.len() > 0 {
        let offline_msg = format!(
            "Offline servers: {:?}. Quantity: {}",
            offline_servers,
            offline_servers.len(),
        );
        display_message("❌", &offline_msg, "red");
    }

    Ok(())
}

///Get boolean response
fn get_user_confirmation(question: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(true)
        .interact()
        .unwrap()
}

///Get text response
fn get_user_input(text: &str, default_text: &str) -> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(text)
        .default(default_text.into())
        .interact_text()
        .unwrap()
}

///Get singe response from choices
pub fn get_user_selection(items: &Vec<&str>, title: &str) -> (String, usize) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .with_prompt(title)
        .default(0)
        .interact()
        .unwrap();

    (items.get(selection).unwrap().to_string(), selection)
}

///Get multiple responses
fn get_user_multi(items: &Vec<&str>, title: &str) -> Vec<String> {
    let mut res = Vec::new();

    let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&items)
        .with_prompt(title)
        .interact()
        .unwrap();

    for i in chosen {
        let each = items.get(i);
        if each.is_some() {
            res.push(each.unwrap().to_string());
        }
    }
    res
}
