use colored::*;
use copypasta::{ClipboardContext, ClipboardProvider};
use rusqlite::{params, Connection, Result};
use std::net::IpAddr;
use std::process::Command;
use std::{io::stdin, u8};
use tabled::{Style, Table, Tabled};

pub const TABLE: &str = "server_connections";
pub const SQL_FILE: &str = "shecret.db3";

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
    let user = get_input("User:");
    let ip = get_input("IP:");

    match ip.parse::<IpAddr>() {
        Ok(_) => {
            let key_path = get_input("Public key path:");
            let mut port = get_input("Port: 22 (default)");
            let alias = get_input("Alias:");

            if port.len() == 0 {
                port = "22".to_string();
            }

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
pub fn get_input(text: &str) -> String {
    let mut input = String::new();
    println!("{}", text);
    stdin().read_line(&mut input).unwrap();
    String::from(input.trim())
}

///Display all database connections as a table
pub fn display_connections(records: &Vec<ServerConnection>) {
    let table = Table::new(records).with(Style::modern()).to_string();
    println!("{table}");
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

///Start SSH/SFTP connection
pub fn start_connection(records: &Vec<ServerConnection>) -> Result<()> {
    let id = get_input("Connection ID:");

    match id.parse::<u8>() {
        Ok(x) => {
            let mut msg = format!("[WARNING] ID not found: {}", &x).color("yellow");

            for i in records {
                if i.id == x {
                    let connection_type =
                        match get_input("Type: SSH (default) or type 1 for SFTP").as_str() {
                            "1" => "sftp",
                            _ => "ssh",
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
    let msg = format!("[{}] {}", message_type, message).color(color);
    println!("{msg}");
}

///Create SSH key on current directory
pub fn create_key() -> Result<()> {
    let name = get_input("SSH key name");

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
