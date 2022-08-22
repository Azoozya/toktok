use crate::network::host::Host;
use metrohash::MetroHash64;

use std::hash::Hasher;

use sqlite::Connection;

/*



*/

pub struct SqliteCore {
    client: Host,
    client_hash: u64,
    openssh_id: Option<Vec<u8>>,
    openssh_pub: Option<Vec<u8>>,
    active: Option<()>,
    last_activity: Option<String>,
}

impl SqliteCore {
    pub fn new(client: &Host) -> Self {
        let client = client.clone();
        let mut hasher = MetroHash64::new();
        hasher.write(client.local_addr().as_bytes());
        
        SqliteCore { client: client.clone(), client_hash: hasher.finish(), openssh_id: None, openssh_pub: None, active: None, last_activity: None }
    }

    fn hash(&self) -> u64 {
        self.client_hash
    }

    pub fn init(filename: &str) -> Option<Connection> {
        if let Ok(co) = Connection::open( &std::path::Path::new(&filename) ) {
            co.execute("
                CREATE TABLE Core (
                    Addr UNSIGNED BIG INT PRIMARY KEY NOT NULL,
                    OpensshID BLOB,
                    OpensshPub BLOB,
                    Active BOOLEAN,
                    LastActivity DATETIME
                );
            ").ok();
            Some(co)
        }
        else { None }
    }

    pub fn read(&mut self, conn: &Connection) {

    }
}

#[test]
pub fn test_sqlite_init() {
    let co = SqliteCore::init("test.db").unwrap();
}