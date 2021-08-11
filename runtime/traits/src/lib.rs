pub trait AttestorAccounting {
	
	fn attestor_staking(&self) -> Result<u32, u32>;
}
