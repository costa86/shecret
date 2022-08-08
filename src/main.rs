use colored::*;
use rusqlite::{Connection, Result};
use shecret::*;
use std::io::stdin;

fn main() -> Result<()> {
    let conn = Connection::open(SQL_FILE)?;
    create_database(&conn)?;

    let mut input = String::new();

    let title =
        "SHECRET - SSH and SFTP Rust-based client\nAuthor: Lorenzo Costa <costa86@zoho.com>\n"
            .color("green");
    println!("{title}");

    let options_menu = "Available options (case insensitive):
    CC: Create a server connection
    LC: List all server connections
    SC: Start a connection
    DC: Delete a server connection
    PD: Purge database (delete all server connections)
    CK: Create SSH key
    Q:  Quit";

    loop {
        println!("{options_menu}");
        stdin().read_line(&mut input).unwrap();

        match input.to_uppercase().as_str().trim() {
            "CC" => create_server_connection(&conn)?,
            "LC" => display_connections(&get_connections(&conn).unwrap()),
            "SC" => start_connection(&get_connections(&conn).unwrap())?,
            "DC" => delete_record(&conn, &get_input("ID to delete:"))?,
            "PD" => purge_database(&conn)?,
            "CK" => create_key()?,
            "Q" => break,
            _ => {
                display_message("ERROR", "Invalid option", "red");
            }
        }
        input.clear();
    }
    Ok(())
}
