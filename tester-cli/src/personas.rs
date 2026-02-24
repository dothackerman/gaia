use anyhow::Result;
use core::str::FromStr;
use subxt::utils::AccountId32;
use subxt_signer::sr25519::Keypair;
use subxt_signer::SecretUri;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Persona {
    Alice,
    Bob,
    Charlie,
    Dave,
    Eve,
    Ferdie,
}

impl Persona {
    pub const ALL: [Persona; 6] = [
        Persona::Alice,
        Persona::Bob,
        Persona::Charlie,
        Persona::Dave,
        Persona::Eve,
        Persona::Ferdie,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Persona::Alice => "Alice",
            Persona::Bob => "Bob",
            Persona::Charlie => "Charlie",
            Persona::Dave => "Dave",
            Persona::Eve => "Eve",
            Persona::Ferdie => "Ferdie",
        }
    }

    pub fn keypair(self) -> Result<Keypair> {
        let uri = SecretUri::from_str(&format!("//{}", self.label()))?;
        Keypair::from_uri(&uri).map_err(Into::into)
    }

    pub fn account_id(self) -> Result<AccountId32> {
        let public_key = self.keypair()?.public_key();
        Ok(AccountId32(public_key.0))
    }
}
