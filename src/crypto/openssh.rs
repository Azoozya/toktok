use rand_core::OsRng;
use ssh_key::LineEnding;
use ssh_key::private::PrivateKey;
use ssh_key::public::PublicKey;
use signature::{Verifier,Signer};
use crate::crypto::asymetric::{KeyPair,KeyPairError};

fn from_openssh_public(filename: &String) -> Result<PublicKey,ssh_key::Error> {
    PublicKey::read_openssh_file(std::path::Path::new(&filename))
}

fn from_openssh_private(filename: &String) -> Result<PrivateKey,ssh_key::Error> {
    PrivateKey::read_openssh_file(std::path::Path::new(&filename))
}

pub fn from(filename: String, passphrase: Option<String>) -> Result<KeyPair,KeyPairError> {

    match from_openssh_public(&filename) {
        Ok(public) => Ok( KeyPair::from(  public ) ),
            Err(_) => {
                
            let mut private = match from_openssh_private(&filename) {
                Err(_) => {
                    return Err( KeyPairError::ParsingError );
                },
                Ok(maybe_encrypted) => maybe_encrypted,
            };
            
            if private.is_encrypted() {
                match passphrase {
                    None => { return Err( KeyPairError::NoPasshpraseProvided ); },
                    Some(passphrase) => {
                        private = match private.decrypt(&passphrase) {
                            Err(_) => {
                                return Err( KeyPairError::WrongPasshpraseProvided );
                            },
                            Ok(decrypted) => decrypted,
                        };
                    }
                }
            }
            
            KeyPair::try_from( private )
        }
    }
}

fn into_openssh_private(filename: &String, key: PrivateKey) -> Result<(),ssh_key::Error> {
    key.write_openssh_file(std::path::Path::new(&filename),LineEnding::LF)
}

fn into_openssh_public(filename: &String, key: PublicKey) -> Result<(),ssh_key::Error> {
    key.write_openssh_file(std::path::Path::new(&filename))
}

// as openssh: public filename is postfixed by .pub
pub fn into(filename: String, keypair: KeyPair, passphrase: Option<String>) -> Result<(),KeyPairError> {
    // /*
    let public: PublicKey = KeyPair::into(keypair.clone() );
    if let Err(_) = into_openssh_public (
        &format!("{}.pub",filename),
        public
    ) {
        return Err( KeyPairError::WritingError );
    }

    // */
    
    // /*
    // try_from would fail if it's an already encrypted key
    let mut private: PrivateKey = match KeyPair::try_into(keypair.clone() ) {
        Err(err) => { match err {
            KeyPairError::NoPrivateKey => { return Ok(()); },
            _ => { return Err(err); },
            } 
        },
        Ok(private) => private,
    };
    
    // trying to encrypt it if a passphrase was submitted
    if let Some(passphrase) = passphrase {
        if passphrase.len() > 0 {
            private = private.encrypt(&mut OsRng, passphrase).unwrap();
        }
    }

    // trying to write it
    if let Err(_) = into_openssh_private (
        &filename,
        private
    ) {
        return Err( KeyPairError::WritingError );
    }

    // */
    Ok(())
}

#[test]
pub fn test_from_into_openssh() {
    let public = crate::crypto::openssh::from("toktok.pub".to_string(),None);
    let err = crate::crypto::openssh::from(
        "toktok".to_string(),
        None,
    );
    let private = crate::crypto::openssh::from(
        "toktok".to_string(),
        Some(
            "lama".to_string()
        )
    );
    assert!(public.is_ok());
    assert!(err.is_err());
    assert!(private.is_ok());

    assert!(crate::crypto::openssh::into("kot".to_string(), public.unwrap(), None).is_ok());
    assert!(crate::crypto::openssh::into("kotkot".to_string(), private.unwrap(), None).is_ok());
    assert!(crate::crypto::openssh::from("kot.pub".to_string(), Some( "lama".to_string() )).is_ok() );
    assert!(crate::crypto::openssh::from("kotkot".to_string(), Some( "lama".to_string() )).is_ok() );
    assert!(crate::crypto::openssh::from("kotkot.pub".to_string(), Some( "lama".to_string() )).is_ok() );
    
    //assert!(false);
}

#[test]
pub fn test_sign_verify() {
    let public = crate::crypto::openssh::from("toktok.pub".to_string(),None);
    let private = crate::crypto::openssh::from(
        "toktok".to_string(),
        Some(
            "lama".to_string()
        )
    );

    assert!(public.is_ok());
    assert!(private.is_ok());
    let public = public.unwrap();
    let private = private.unwrap();

    let data = Vec::from("Huître");
    let other_data = Vec::from("8tre");

    let signature = private.try_sign(&data);
    assert!(signature.is_ok());
    let signature = signature.unwrap();

    assert!( public.verify(&data, &signature).is_ok() );
    assert!( public.verify(&other_data, &signature).is_err() );

    //println!("{:#?}\n{:#?}",signature,signature.as_bytes());
    //assert!(false);
}