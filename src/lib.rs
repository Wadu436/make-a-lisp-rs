use std::{
    cell::{Ref, RefCell},
    collections::hash_map,
    rc::Rc,
};

pub use env::Env;
use error::MalError;
use types::{MalData, MalNativeFunction};

mod env;
pub mod error;
mod reader;
mod types;

pub fn load_builtins(env: &mut Env) {
    env.set(
        "+".to_owned(),
        MalData::MalNativeFunction(MalNativeFunction(Rc::new(Box::new(
            |args: &[MalData]| {
                if let MalData::Integer(arg0) = &args[0] {
                    if let MalData::Integer(arg1) = &args[1] {
                        Ok(MalData::Integer(arg0 + arg1))
                    } else {
                        Err(MalError::TypeError(args[1].clone()))
                    }
                } else {
                    Err(MalError::TypeError(args[0].clone()))
                }
            },
        )))),
    );

    env.set(
        "-".to_owned(),
        MalData::MalNativeFunction(MalNativeFunction(Rc::new(Box::new(
            |args: &[MalData]| {
                if let MalData::Integer(arg0) = &args[0] {
                    if let MalData::Integer(arg1) = &args[1] {
                        Ok(MalData::Integer(arg0 - arg1))
                    } else {
                        Err(MalError::TypeError(args[1].clone()))
                    }
                } else {
                    Err(MalError::TypeError(args[0].clone()))
                }
            },
        )))),
    );

    env.set(
        "*".to_owned(),
        MalData::MalNativeFunction(MalNativeFunction(Rc::new(Box::new(
            |args: &[MalData]| {
                if let MalData::Integer(arg0) = &args[0] {
                    if let MalData::Integer(arg1) = &args[1] {
                        Ok(MalData::Integer(arg0 * arg1))
                    } else {
                        Err(MalError::TypeError(args[1].clone()))
                    }
                } else {
                    Err(MalError::TypeError(args[0].clone()))
                }
            },
        )))),
    );

    env.set(
        "/".to_owned(),
        MalData::MalNativeFunction(MalNativeFunction(Rc::new(Box::new(
            |args: &[MalData]| {
                if let MalData::Integer(arg0) = &args[0] {
                    if let MalData::Integer(arg1) = &args[1] {
                        Ok(MalData::Integer(arg0 / arg1))
                    } else {
                        Err(MalError::TypeError(args[1].clone()))
                    }
                } else {
                    Err(MalError::TypeError(args[0].clone()))
                }
            },
        )))),
    );
}

pub fn read(input: String) -> Result<MalData, MalError> {
    let mut reader = reader::Reader::new(input);
    reader.read_input()
}

pub fn eval(input: MalData, env: Rc<RefCell<Env>>) -> Result<MalData, MalError> {
    if let Some(debug_eval) = env.borrow().get("DEBUG-EVAL") {
        match debug_eval {
            MalData::Nil | MalData::False => {}
            _ => println!("EVAL: {}", input),
        }
    }
    use MalData::*;
    match &input {
        Symbol(s) => {
            if let Some(value) = env.borrow().get(s) {
                Ok(value.clone())
            } else {
                Err(MalError::SymbolNotFound(s.clone()))
            }
        }
        List(list) => {
            if !list.is_empty() {
                if let MalData::Symbol(s) = &list[0] {
                    match s.as_str() {
                        "def!" => {
                            // Check if the list is the right length
                            if list.len() != 3 {
                                return Err(MalError::TypeError(input.clone()));
                            }
                            // Get the key and value
                            if let MalData::Symbol(key) = &list[1] {
                                let value = eval(list[2].clone(), env.clone())?;
                                env.borrow_mut().set(key.clone(), value.clone());
                                return Ok(value);
                            } else {
                                // Error
                                return Err(MalError::TypeError(list[1].clone()));
                            }
                        }
                        "let*" => {
                            // Check if the list has an even number of elements
                            if list.len() != 3 {
                                return Err(MalError::TypeError(input.clone()));
                            }
                            // Evaluate the bindings list
                            if let MalData::List(bindings) | MalData::Vector(bindings) = &list[1] {
                                let pairs = bindings.chunks_exact(2);
                                if pairs.remainder().len() != 0 {
                                    return Err(MalError::TypeError(list[1].clone()));
                                }
                                let new_env = Env::new(Some(env.clone()));
                                // Load up the new environment with new bindings
                                for pair in pairs {
                                    let key = if let MalData::Symbol(key) = &pair[0] {
                                        key.clone()
                                    } else {
                                        return Err(MalError::TypeError(pair[0].clone()));
                                    };
                                    let value = eval(pair[1].clone(), new_env.clone())?;
                                    new_env.borrow_mut().set(key, value);
                                }
                                // Evaluate the body
                                return eval(list[2].clone(), new_env);
                            } else {
                                // Error
                                return Err(MalError::TypeError(list[1].clone()));
                            }
                        }
                        _ => {}
                    }
                }
            }

            // "Apply phase"
            let evaluated_list = list
                .iter()
                .map(|el| eval(el.clone(), env.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            if !evaluated_list.is_empty() {
                if let MalData::MalNativeFunction(f) = &evaluated_list[0] {
                    // Apply the function
                    Ok(f.0.as_ref()(&evaluated_list[1..])?)
                } else {
                    Err(MalError::TypeError(evaluated_list[0].clone()))
                }
            } else {
                Ok(MalData::List(evaluated_list))
            }
        }
        Vector(vector) => Ok(MalData::Vector(
            vector
                .iter()
                .map(|el| eval(el.clone(), env.clone()))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        HashMap(hash_map) => {
            // Evaluate the hashmap
            let evaluated_hash_map: Result<
                hash_map::HashMap<types::MalHashMapKey, MalData>,
                MalError,
            > = hash_map
                .iter()
                .map(|(key, value)| {
                    eval(value.clone(), env.clone()).map(|value| (key.clone(), value))
                })
                .collect();
            Ok(MalData::HashMap(evaluated_hash_map?))
        }
        _ => Ok(input),
    }
}

pub fn print(input: MalData) -> String {
    format!("{}", input)
}

pub fn rep(input: String, env: Rc<RefCell<Env>>) -> Result<String, MalError> {
    read(input)
        .and_then(|ast| eval(ast, env))
        .and_then(|evaluated| Ok(print(evaluated)))
}
