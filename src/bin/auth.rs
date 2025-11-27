use std::error::Error;


use std::fs::File;

use rand::rngs::OsRng;
use argon2::{ Argon2, PasswordHasher, PasswordVerifier, };
use password_hash::{ PasswordHasher, SaltString };
// use rand_core::OsRng;

// pub fn gen_password_hash(password: &str) -> Result<String, AppError> {
//     // let salt = SaltString::generate(&mut OsRng);
//     // let argon2 = Argon2::default();
//     // let pwd_hash_res = argon2.hash_password(password.as_bytes(), &salt);
//
//     match pwd_hash_res {
//         Ok(hash) => Ok(hash.to_string()),
//         Err(e) => Err(AppError::PasswordHashError(format!("{}", e))),
//     }
// }

static AUTH_PATH: &str = "auth.json";

struct UserAuth {
    username: String,
    password: String,
}

fn hash(raw_password: String) -> Result<String, Box<dyn Error>> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let hash = argon2.hash_password(raw_password.as_bytes(), &salt)?;

    Ok(hash)
}

struct Database(Vec<UserAuth>);

pub struct Auth {
    db: Database,
}

impl Auth {
    pub fn create() -> Self {
        let auth = Self {
            db: Database(Vec::new()),
        };

        Self
    }

    fn load_auth_json(path: &str) -> Result<Vec<UserAuth>, Box<dyn Error>> {
        let file   = File::open(path)?;
        let reader = std::io::BufReader::new(file);

        let data: Vec<UserAuth> = serde_json::from_reader(reader)?;

        Ok(data)
    }

    pub fn load_from(path: &str) {
        let data = load_auth_json(path).unwrap();

        Self {
            db: data,
        }
    }

    pub fn sign_up(self, username: String, raw_psw: String) -> Result<UserAuth, Box<dyn Error>> {
        for user in self.db {
            if user.username == username { Err }
        }


    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let auth = Auth::load_from(AUTH_PATH);
}
