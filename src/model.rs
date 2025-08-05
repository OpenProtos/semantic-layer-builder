use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::fs;
use toml_edit::DocumentMut;

pub struct Model {
    pub conn: Connection,               // sqlite connection having all data needed
    pub layer: DocumentMut,             // layer datas
    pub layer_path: std::path::PathBuf, // path of the file for saving it - Placeholder
}

impl Model {
    pub fn new(db_path: &std::path::PathBuf, layer_path: std::path::PathBuf) -> Result<Self> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("Failing to connect to `{:?}`", &db_path))?;

        let contents = fs::read_to_string(&layer_path)
            .with_context(|| format!("Could not read file `{:?}`", &layer_path))?;

        let layer: DocumentMut = contents
            .parse::<DocumentMut>()
            .with_context(|| format!("Unable to parse TOML from `{:?}`", &layer_path))?;

        Ok(Model {
            conn,
            layer,
            layer_path,
        })
    }

    pub fn query_protos(&self) -> Result<Vec<(usize, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT rowid, proto FROM tcp_proto_messages")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        Ok(rows
            .filter_map(Result::ok)
            .collect::<Vec<(usize, String)>>())
    }

    pub fn query_data(&self, proto_id: &usize) -> Result<String> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM tcp_proto_messages WHERE rowid = ?")?;
        let rows = stmt.query_one(&[(1, proto_id)], |row| Ok(row.get(0)?))?;

        Ok(rows)
    }

    pub fn save_layer(&self) -> Result<()> {
        std::fs::write(&self.layer_path, self.layer.to_string())?;
        Ok(())
    }
}
