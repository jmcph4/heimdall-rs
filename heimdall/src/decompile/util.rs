use std::{collections::HashMap};

use ethers::{prelude::rand::{self, Rng}, abi::AbiEncode};
use heimdall_common::{
    ether::{evm::vm::VM, signatures::{ResolvedFunction, resolve_signature}}
};


// Find all function selectors in the given EVM.
pub fn find_function_selectors(evm: &VM, assembly: String) -> Vec<String> {
    let mut function_selectors = Vec::new();

    let mut vm = evm.clone();

    // find a selector not present in the assembly
    let selector;
    loop {
        let num = rand::thread_rng().gen_range(286331153..2147483647);
        if !vm.bytecode.contains(&format!("63{}", num.encode_hex()[58..].to_string())) {
            selector = num.encode_hex()[58..].to_string();
            break;
        }
    }

    // execute the EVM call to find the dispatcher revert
    let dispatcher_revert = vm.call(selector, 0).instruction - 1;

    // search through assembly for PUSH4 instructions up until the dispatcher revert
    let assembly: Vec<String> = assembly.split("\n").map(|line| line.trim().to_string()).collect();
    for line in assembly.iter() {
        let instruction_args: Vec<String> = line.split(" ").map(|arg| arg.to_string()).collect();
        let program_counter: u128 = instruction_args[0].clone().parse().unwrap();
        let instruction = instruction_args[1].clone();

        if program_counter < dispatcher_revert {
            if instruction == "PUSH4" {
                let function_selector = instruction_args[2].clone();
                function_selectors.push(function_selector);
            }
        }
        else {
            break;
        }
    }

    function_selectors
}

pub fn resolve_function_selectors(selectors: Vec<String>) -> HashMap<String, Vec<ResolvedFunction>> {
    
    let mut resolved_functions: HashMap<String, Vec<ResolvedFunction>> = HashMap::new();

    for selector in selectors {
        match resolve_signature(&selector) {
            Some(function) => {
                resolved_functions.insert(selector, function);
            },
            None => continue
        }
    }

    resolved_functions
}