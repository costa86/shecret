use colored::*;
use home::home_dir;
use rusqlite::{Connection, Result};
use shecret::*;

fn main() -> Result<()> {
    let sql_file = format!(
        "{}/{}.db3",
        home_dir().unwrap().display(),
        env!("CARGO_PKG_NAME")
    );

    let conn = Connection::open(&sql_file)?;
    create_database(&conn)?;

    let title = format!(
        "\n{} - {}\nAuthors: {}\nVersion: {}\nLicense: {}\nDatabase path: {}\nDocumentation: {}\nCrafted with ❤️ using Rust language\n",
        env!("CARGO_PKG_NAME").to_uppercase(),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_LICENSE"),
        &sql_file,
        env!("CARGO_PKG_HOMEPAGE")
    )
    .color("white");
    println!("{title}");

    loop {
        let (_, index) = get_user_selection(&CHOICES.to_vec(), "Option");

        match index {
            0 => display_connections(&get_connections(&conn).unwrap()),
            1 => start_connection(&get_connections(&conn).unwrap())?,
            2 => create_server_connection(&conn)?,
            3 => {
                let connections = get_connections(&conn).unwrap();
                let records_aliases: Vec<&str> =
                    connections.iter().map(|x| x.alias.as_str()).collect();
                delete_records_by_alias(
                    &conn,
                    &get_user_selection(&records_aliases, "Server connection to delete").0,
                )?
            }
            4 => purge_database(&conn)?,
            5 => create_key()?,
            6 => issue_command(&get_connections(&conn).unwrap())?,
            7 => check_server_status(&get_connections(&conn).unwrap())?,
            _ => break,
        }
    }
    Ok(())
}
