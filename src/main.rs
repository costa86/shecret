use colored::*;
use home::home_dir;
use rusqlite::{Connection, Result};
use shecret::*;
use std::{env, io::stdin};

fn main() -> Result<()> {
    let sql_file = format!(
        "{}/{}.db3",
        home_dir().unwrap().display(),
        env!("CARGO_PKG_NAME")
    );

    let conn = Connection::open(&sql_file)?;
    create_database(&conn)?;

    let mut input = String::new();

    let title = format!(
        "{} - {}\nAuthors: {}\nVersion: {}\nLicense: {}\nDatabase path: {}\nCrafted with ❤️ using Rust language\n",
        env!("CARGO_PKG_NAME").to_uppercase(),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_LICENSE"),
        &sql_file
    )
    .color("yellow");
    println!("{title}");

    let options_menu = "Available options (case insensitive):
    CC: Create a server connection
    LC: List all server connections
    SC: Start a connection
    DC: Delete a server connection
    PD: Purge database (delete all server connections)
    CK: Create SSH key
    IC: Issue SSH command to multiple servers
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
            "IC" => issue_command(&get_connections(&conn).unwrap())?,
            "Q" => break,
            _ => {
                display_message("ERROR", "Invalid option", "red");
            }
        }
        input.clear();
    }
    Ok(())
}
