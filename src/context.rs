use crate::fnv;

pub trait Context<'ctx> {
	type Fields: AsRef<[Field<'ctx>]>;

	fn to_fields(&'ctx self) -> Self::Fields;
}

pub struct Field<'ctx> {
	pub(crate) hash: u64,
	pub(crate) value: &'ctx str,
}

impl<'ctx> Field<'ctx> {
	/// Hash should be a 64 bit variant of FNV-1a hash of the field name
	pub fn new(hash: u64, value: &'ctx str) -> Self {
		Field {
			hash,
			value,
		}
	}

	/// Create a field from name (note: this will perform the hashing)
	pub fn from_name(name: &str, value: &'ctx str) -> Self {
		let hash = fnv::hash(name);

		Field {
			hash,
			value,
		}
	}
}
