//! Public-API contract of `engine`: the codes and every declared failure a client translates by.

#[cfg(test)]
mod tests {

    use engine::error::{Code, Fault, Faults};
    use std::collections::HashSet;
    use engine::Contract;

    #[test]
    fn every_declared_key_resolves_to_one_fault () {

        let faults = Contract::faults();
        let keys: HashSet<&str> = faults.iter().map(|fault| fault.key()).collect();

        assert_eq!(keys.len(), faults.len());
        assert!(faults.iter().all(|fault| Contract::fault(fault.key()) == Some(*fault) && !fault.detail().is_empty()));

    }

    #[test]
    fn keys_carry_their_code_and_codes_are_the_error_vocabulary () {

        assert_eq!(Contract::fault("not_uri").map(Fault::code), Some(Code::Invalid));
        assert_eq!(Contract::fault("no_tls").map(Fault::code), Some(Code::Unsupported));
        assert_eq!(Contract::fault("command_timeout").map(Fault::code), Some(Code::Timeout));
        assert_eq!(Contract::fault(Faults::POISONED.key()), Some(Faults::POISONED));
        assert_eq!(Contract::fault("nothing_like_this"), None);
        assert_eq!(Contract::codes(), Code::ALL);

    }

}
