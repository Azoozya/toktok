use ssh_key::private::{PrivateKey, KeypairData as Private};
use ssh_key::public::{PublicKey, KeyData as Public};
use ssh_key::{Signature,Algorithm};
use signature::{Verifier,Signer,Error};

// Should change into std::io::Error for parsing error
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum KeyPairError {
    ParsingError,
    WritingError,
    NoPrivateKey,
    NoPublicKey,
    NoPasshpraseProvided,
    WrongPasshpraseProvided,
    SigningError,
}

// Is potentially usefull to define a Type<Q: Signer, P: Verifier>
#[derive(Debug,Clone)]
pub struct KeyPair {
    private: Option<Private>,
    public: Public,
}

impl KeyPair {
    pub fn algorithm(&self) -> Algorithm {
        if let Some(private) = &self.private {
            private.algorithm().unwrap()
        }
        else {
            self.public.algorithm()
        }
    }
}

// Compatible ssh_key crate /*
impl Signer<Signature> for KeyPair {
    fn try_sign(&self, message: &[u8]) -> Result<Signature, Error> {
        match self.private.clone() {
            None =>  Err( Error::new() ),
            Some(keypair) => {
                let algo = keypair.algorithm()?;
                let signature = keypair.sign(message);
                
                match Signature::new(
                    algo,
                    signature.as_ref().to_vec(),
                ) {
                    Ok(signature) => Ok( signature ),
                    Err(_) => Err( Error::new() ),
                }
            },
        }
    }
}

impl Verifier<Signature> for KeyPair {
    fn verify(&self, message: &[u8], signature: &Signature) -> Result<(),Error> {
        self.public.clone().verify(message, &signature)
    }
}


impl TryFrom<Private> for KeyPair {
    type Error = KeyPairError;
    fn try_from(keydata: Private) -> Result<Self,Self::Error> {
        let private: PrivateKey = match PrivateKey::try_from(keydata) {
            Err(_) => { return Err( KeyPairError::NoPasshpraseProvided); },
            Ok(private) => private,
        };

        let public = Public::from( private.clone() );
        let private = Some( private.key_data().clone() );
        Ok( KeyPair { private, public } )
    }
}

impl From<Public> for KeyPair {
    fn from(keydata: Public) -> Self {
        let public = keydata;
        let private = None;
        KeyPair { private, public }
    }
}
// Compatible ssh_key crate */

// Used by openssh module /*
impl TryFrom<PrivateKey> for KeyPair {
    type Error = KeyPairError;
    fn try_from(keydata: PrivateKey) -> Result<Self,Self::Error> {
        let public = Public::from( keydata.clone() );
        let private = Some( keydata.key_data().clone() );
        Ok( KeyPair { private, public } )
    }
}

impl TryInto<PrivateKey> for KeyPair {
    type Error = KeyPairError;
    fn try_into(self) -> Result<PrivateKey,Self::Error> {
        
        match self.private {
            None => Err( KeyPairError::NoPrivateKey ),
            Some(keydata) => {
                match PrivateKey::try_from(keydata) {
                    Err(_) => Err( KeyPairError::NoPasshpraseProvided),
                    Ok(private) => Ok( private ),
                }
            }
        }
    }
}

impl From<PublicKey> for KeyPair {
    fn from(keydata: PublicKey) -> Self {
        let public = keydata.key_data().clone();
        let private = None;
        KeyPair { private, public }
    }
}

impl Into<PublicKey> for KeyPair {
    fn into(self) -> PublicKey {
        PublicKey::from( self.public )
    }
}
// Used by openssh module */

