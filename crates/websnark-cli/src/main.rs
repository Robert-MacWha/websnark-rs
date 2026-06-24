use std::fs;

use clap::{Parser, Subcommand};
use websnark_rs::{proving_key::ProvingKey, verifying_key::VerifyingKey};

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a proving key from snarkjs JSON to binary format
    ConvertProvingKey {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
    },
    /// Convert a verifying key from snarkjs JSON to binary format
    ConvertVerifyingKey {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ConvertProvingKey { input, output } => {
            let proving_key_json = fs::read_to_string(&input).expect("Failed to read proving key");
            let proving_key: ProvingKey =
                serde_json::from_str(&proving_key_json).expect("Failed to parse proving key");
            let proving_key_bytes =
                postcard::to_stdvec(&proving_key).expect("Failed to serialize proving key");
            fs::write(&output, &proving_key_bytes).expect("Failed to write proving key");

            println!("Converted proving key, testing deserialization...");
            let proving_key_bytes = fs::read(&output).expect("Failed to read proving key");
            let proving_key_deserialized: ProvingKey = postcard::from_bytes(&proving_key_bytes)
                .expect("Failed to deserialize proving key");

            assert_eq!(proving_key, proving_key_deserialized);
            println!("Deserialization successful!");
        }
        Commands::ConvertVerifyingKey { input, output } => {
            let verifying_key_json =
                fs::read_to_string(&input).expect("Failed to read verifying key");
            let verifying_key: VerifyingKey =
                serde_json::from_str(&verifying_key_json).expect("Failed to parse verifying key");
            let verifying_key_bytes =
                postcard::to_stdvec(&verifying_key).expect("Failed to serialize verifying key");
            fs::write(&output, &verifying_key_bytes).expect("Failed to write verifying key");
        }
    }
}
