use colored::*;
use copypasta::{ClipboardContext, ClipboardProvider};
use rusqlite::{params, Connection, Result};
use std::io::Error;
use std::net::IpAddr;
use std::process::Command;
use std::thread;
use std::{io::stdin, u8};
use tabled::{Style, Table, Tabled};

pub const TABLE: &str = "server_connections";
const ALL: &str = "all";
const SSH: &str = "ssh";
const DEFAULT_SSH_COMMAND: &str = "hostname";

#[derive(Debug, Tabled)]
pub struct ServerConnection {
    id: u8,
    user: String,
    ip: String,
    key_path: String,
    port: String,
    alias: String,
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

///Delete single database record
pub fn delete_record(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(&format!("DELETE FROM {TABLE} WHERE id = ?1"), params![id])?;
    Ok(())
}

///Delete all database records
pub fn purge_database(conn: &Connection) -> Result<()> {
    conn.execute(&format!("DELETE FROM {TABLE}"), params![])?;
    Ok(())
}

///Create server connection on database
pub fn create_server_connection(conn: &Connection) -> Result<()> {
    let user = get_input("User", env!("USER"));
    let ip = get_input("IP", "0.0.0.0");

    match ip.parse::<IpAddr>() {
        Ok(_) => {
            let key_path = get_input("Public key path", ".");
            let port = get_input("Port", "22");
            let alias = get_input("Alias", "sample");

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

///Get user input
pub fn get_input(text: &str, default: &str) -> String {
    let mut input = String::new();
    println!("{}: {} (default)", text, default);

    stdin().read_line(&mut input).unwrap();
    let mut input = String::from(input.trim());
    if input.len() == 0 {
        input = default.to_string();
    }
    input
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
    let id_list = get_input("Connection ID's (separated by spaces)", &ALL);
    let mut command_list: Vec<String> = Vec::new();
    let ssh_command = get_input("SSH command", &DEFAULT_SSH_COMMAND);

    if id_list == ALL {
        for i in records {
            command_list.push(create_command(&ssh_command, &i));
        }
        run_commands(command_list);
        return Ok(());
    }

    let id_list: Vec<u8> = id_list
        .split_whitespace()
        .map(|x| x.parse::<u8>().unwrap_or_default())
        .collect();

    for i in id_list {
        for r in records {
            if r.id == i {
                command_list.push(create_command(&ssh_command, &r));
            }
        }
    }
    run_commands(command_list);
    Ok(())
}

///Start SSH/SFTP connection
pub fn start_connection(records: &Vec<ServerConnection>) -> Result<()> {
    let id = get_input("Connection ID", "1");

    match id.parse::<u8>() {
        Ok(x) => {
            let mut msg = format!("[WARNING] ID not found: {}", &x).color("yellow");

            for i in records {
                if i.id == x {
                    let connection_type = match get_input(
                        "Connection type (1 for SFTP)",
                        SSH.to_uppercase().as_str(),
                    )
                    .as_str()
                    {
                        "1" => "sftp",
                        _ => SSH,
                    };
                    let command = &i.get_command(connection_type);
                    set_clipboard(&command);
                    msg = format!("[OK] Sent to clipboad: {command}").color("green");
                    break;
                }
            }
            println!("{msg}");
        }
        Err(_) => display_message("ERROR", "Type a number to get an ID", "red"),
    };
    Ok(())
}

///Display message
pub fn display_message(message_type: &str, message: &str, color: &str) {
    let msg = format!("[{}] {}", message_type.to_uppercase(), message).color(color);
    println!("{msg}");
}

///Create SSH key on current directory
pub fn create_key() -> Result<()> {
    let name = get_input("SSH key name", "sample");

    if name.len() < 2 {
        display_message("ERROR", "name is required", "red");
        return Ok(());
    }

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
    let id_list = get_input("Connection ID's (separated by spaces)", &ALL);
    let mut online_servers: Vec<&str> = Vec::new();
    let mut offline_servers: Vec<&str> = Vec::new();

    if &id_list.as_str() == &ALL {
        for i in records {
            let server_is_online = ping_server(&i.ip);
            match server_is_online {
                true => online_servers.push(&i.alias),
                false => offline_servers.push(&i.alias),
            }
        }
    } else {
        let id_list: Vec<u8> = id_list
            .split_whitespace()
            .map(|x| x.parse::<u8>().unwrap_or_default())
            .collect();

        for i in id_list {
            for r in records {
                if r.id == i {
                    let server_is_online = ping_server(&r.ip);
                    match server_is_online {
                        true => online_servers.push(&r.alias),
                        false => offline_servers.push(&r.alias),
                    }
                }
            }
        }
    }
    let online_msg = format!(
        "Online servers: {:?}. Quantity: {}",
        online_servers,
        online_servers.len(),
    );

    let offline_msg = format!(
        "Offline servers: {:?}. Quantity: {}",
        offline_servers,
        offline_servers.len(),
    );

    display_message("✅", &online_msg, "green");
    display_message("❌", &offline_msg, "red");

    Ok(())
}
