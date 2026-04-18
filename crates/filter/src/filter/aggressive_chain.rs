use parser::core::encounter::Encounter;

use crate::filter::filter_chain::FilterChain;
use crate::filter::Filter;

pub struct AggressiveChain {
    chain: FilterChain,
    action: Box<dyn Fn(&[Encounter])>,
}

impl AggressiveChain {
    pub fn new(chain: FilterChain, action: impl Fn(&[Encounter]) + 'static) -> Self {
        AggressiveChain {
            chain,
            action: Box::new(action),
        }
    }

    pub fn run(&self, encounters: &Vec<Encounter>) {
        let result = self.chain.apply(encounters);
        if !result.is_empty() {
            (self.action)(&result);
        }
    }
}
