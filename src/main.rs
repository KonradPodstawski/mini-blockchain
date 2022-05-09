use anyhow::Error;
use wasmer::{Store, Module, Instance};
use core::fmt::Debug;
use std::time::SystemTime;
use chrono::DateTime;
use chrono::offset::Utc;
use sha256::digest;
use wasmer_wasi::WasiState;

#[derive(Clone, Debug)]
struct Block {
    time_stamp: String,
    data: String,
    hash: String,
    prev_hash: String,
    contract: Option<Instance>,
}
 
impl Block {
    pub fn run_conrtact(&mut self) -> Result<(), Error> {
        if let Some(contract) = &self.contract {
            let start = contract.exports.get_function("_start")?;
            start.call(&[])?;
        } else {
            println!("No contract found");
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Blockchain {
    chain: Vec<Block>
}

fn calc_hash(time_stamp: String, data_to_hash: String, prev_hash: String) -> String {
    digest(time_stamp + &data_to_hash + &prev_hash)
}

pub fn get_time() -> String {
    let system_time = SystemTime::now();
    let time_stamp: DateTime<Utc> = system_time.into();
    time_stamp.format("%d/%m/%Y %T").to_string()
}

impl Blockchain {

    fn add_block_with_data(&mut self, my_data: String) {
        let time_stamp = get_time();
        let data_to_hash = my_data.clone();
        let hash = calc_hash(time_stamp.clone(), data_to_hash, self.last_hash());
        let prev_hash = self.last_hash();
        let contract = None;

        let new_block = Block { time_stamp , data: my_data, hash, prev_hash, contract };
        self.chain.push(new_block);
    }

    fn add_block_with_contract(&mut self, name_contract_file: String) -> Result<(), Error> {
        let time_stamp = get_time();
        let data_to_hash = name_contract_file.clone();
        let hash = calc_hash(time_stamp.clone(), data_to_hash, self.last_hash());
        let prev_hash = self.last_hash();

        match Blockchain::load_contract(name_contract_file.clone()) {
            Ok(contract) => {
                let new_block = Block { time_stamp , data: name_contract_file, hash, prev_hash, contract };
                self.chain.push(new_block);
                Ok(())
            }
            Err(e) => {
                println!("{:?}", e);
                Err(e)
            }
        }
    }

    pub fn load_contract(name_contract_file: String) -> Result<Option<Instance>, Error> {
        let store = Store::default();
        let module = Module::from_file(&store, name_contract_file.clone() + ".wasm")?;
    
        let mut wasi_env = WasiState::new("")
        .finalize()?;
    
        let import_object = wasi_env.import_object(&module)?;
        Ok(Some(Instance::new(&module, &import_object)?))
    }

    fn last_hash(&self) -> String {
        match self.chain.last() {
            Some(block) => block.hash.clone(),
            None => String::from("000")
        }
    }

    fn get_block_by_index(&self, index: usize) -> Block {
        self.chain.get(index).unwrap().clone()
    }

    fn new() -> Self {
        let mut blockchain = Self {
            chain: Vec::new()
        };

        blockchain.add_block_with_data(String::from("Genresis Block"));
        blockchain
    }
}

fn main() -> anyhow::Result<()> {
    let mut chain = Blockchain::new();
    
    chain.add_block_with_data(String::from("new data"));
    chain.add_block_with_contract(String::from("hello"))?;

    let mut block_err = chain.get_block_by_index(1);
    let mut block = chain.get_block_by_index(2);

    println!("{:#?}", chain);
    println!("results of operations of contracts: \n");

    block_err.run_conrtact()?;
    block.run_conrtact()?;

    Ok(())
}
