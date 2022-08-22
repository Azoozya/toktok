use serde::{Deserialize, Serialize};
use crate::network::host::Host;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    name: String,
    port: u16,
    server: Option<Host>,
}

impl Service {
    pub fn new(name: String, port: u16, server: Option<Host>) -> Self{
        Service { name, port, server }
    }
}

#[test]
pub fn test_serde_service() -> serde_json::Result<()> {
    let s = Service::new("lama".to_string(), 1234, None);
    
    let ss = serde_json::to_string(&s)?;
    println!("{:#?}",ss );
    let ss: Service = serde_json::from_str(&ss)?;
    let sss: Service = serde_json::from_str(r#"{"name":"lama","port":1234,"server":null}"#)?;
    assert_eq!(s,ss);
    assert_eq!(ss,sss);

    Ok(())
}