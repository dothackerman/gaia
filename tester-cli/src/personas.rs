use anyhow::{bail, Result};
use subxt::utils::AccountId32;
use subxt_signer::sr25519::Keypair;

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
        let uri = format!("//{}", self.label());
        Keypair::from_uri(&uri).map_err(Into::into)
    }

    pub fn account_id(self) -> Result<AccountId32> {
        let public_key = self.keypair()?.public_key();
        Ok(AccountId32(public_key.0))
    }

    pub fn from_name(name: &str) -> Result<Self> {
        let lowered = name.to_ascii_lowercase();
        for persona in Self::ALL {
            if lowered == persona.label().to_ascii_lowercase() {
                return Ok(persona);
            }
        }
        bail!("unknown persona: {name}")
    }
}
