//! The spending and chain context used by the diagnostic evaluator.

use std::collections::BTreeSet;

use miniscript::bitcoin::{absolute, relative};
use miniscript::MiniscriptKey;

/// Holds the information needed to evaluate supported Miniscript fragments.
pub struct DiagnosticContext<Pk: MiniscriptKey> {
    pub available_keys: BTreeSet<Pk>,
    pub sha256_preimages: BTreeSet<Pk::Sha256>,
    pub hash256_preimages: BTreeSet<Pk::Hash256>,
    pub ripemd160_preimages: BTreeSet<Pk::Ripemd160>,
    pub hash160_preimages: BTreeSet<Pk::Hash160>,

    pub chain_height: Option<absolute::Height>,
    pub chain_time: Option<absolute::Time>,

    pub elapsed_blocks: Option<relative::Height>,
    pub elapsed_time: Option<relative::Time>,
}

impl<Pk: MiniscriptKey> Default for DiagnosticContext<Pk> {
    fn default() -> Self {
        DiagnosticContext {
            available_keys: BTreeSet::new(),
            sha256_preimages: BTreeSet::new(),
            hash256_preimages: BTreeSet::new(),
            ripemd160_preimages: BTreeSet::new(),
            hash160_preimages: BTreeSet::new(),
            chain_height: None,
            chain_time: None,
            elapsed_blocks: None,
            elapsed_time: None,
        }
    }
}

impl<Pk: MiniscriptKey> DiagnosticContext<Pk> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_key(mut self, pk: Pk) -> Self {
        self.available_keys.insert(pk);
        self
    }

    pub fn with_sha256_preimage(mut self, hash: Pk::Sha256) -> Self {
        self.sha256_preimages.insert(hash);
        self
    }

    pub fn with_hash256_preimage(mut self, hash: Pk::Hash256) -> Self {
        self.hash256_preimages.insert(hash);
        self
    }

    pub fn with_ripemd160_preimage(mut self, hash: Pk::Ripemd160) -> Self {
        self.ripemd160_preimages.insert(hash);
        self
    }

    pub fn with_hash160_preimage(mut self, hash: Pk::Hash160) -> Self {
        self.hash160_preimages.insert(hash);
        self
    }

    pub fn with_chain_height(mut self, height: absolute::Height) -> Self {
        self.chain_height = Some(height);
        self
    }

    pub fn with_chain_time(mut self, time: absolute::Time) -> Self {
        self.chain_time = Some(time);
        self
    }

    pub fn with_elapsed_blocks(mut self, height: relative::Height) -> Self {
        self.elapsed_blocks = Some(height);
        self
    }

    pub fn with_elapsed_time(mut self, time: relative::Time) -> Self {
        self.elapsed_time = Some(time);
        self
    }

    pub fn is_key_available(&self, pk: &Pk) -> bool {
        self.available_keys.contains(pk)
    }

    pub fn has_sha256_preimage(&self, hash: &Pk::Sha256) -> bool {
        self.sha256_preimages.contains(hash)
    }

    pub fn has_hash256_preimage(&self, hash: &Pk::Hash256) -> bool {
        self.hash256_preimages.contains(hash)
    }

    pub fn has_ripemd160_preimage(&self, hash: &Pk::Ripemd160) -> bool {
        self.ripemd160_preimages.contains(hash)
    }

    pub fn has_hash160_preimage(&self, hash: &Pk::Hash160) -> bool {
        self.hash160_preimages.contains(hash)
    }

    pub fn check_older(&self, lock_time: relative::LockTime) -> bool {
        let elapsed_blocks = self.elapsed_blocks.unwrap_or(relative::Height::ZERO);
        let elapsed_time = self.elapsed_time.unwrap_or(relative::Time::ZERO);
        lock_time.is_satisfied_by(elapsed_blocks, elapsed_time)
    }

    pub fn check_after(&self, lock_time: absolute::LockTime) -> bool {
        let height = self.chain_height.unwrap_or(absolute::Height::MIN);
        let time = self.chain_time.unwrap_or(absolute::Time::MIN);
        lock_time.is_satisfied_by(height, time)
    }
}
