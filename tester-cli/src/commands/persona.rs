use crate::personas::Persona;

pub fn list() {
    println!("Available seeded personas:");
    for persona in Persona::ALL {
        println!("- {}", persona.label());
    }
}

pub fn preview(persona: Persona) {
    match persona.account_id() {
        Ok(account) => {
            println!("Persona: {}", persona.label());
            println!("AccountId32: {account}");
        }
        Err(error) => {
            eprintln!("Failed to derive account for {}: {error}", persona.label());
        }
    }
}
