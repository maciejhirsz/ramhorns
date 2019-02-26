use cowvec::CowStr;
use crate::fnv;

pub trait Context<'ctx> {
	type Fields: AsRef<[Field<'ctx>]>;

	fn to_fields(&'ctx self) -> Self::Fields;
}

pub struct Field<'ctx> {
	pub(crate) hash: u64,
	pub(crate) value: CowStr<'ctx>,
}

impl<'ctx> Field<'ctx> {
	/// Hash should be a 64 bit variant of FNV-1a hash of the field name
	pub fn new<Value>(hash: u64, value: Value) -> Self
	where
		Value: Into<CowStr<'ctx>>,
	{
		Field {
			hash,
			value: value.into(),
		}
	}

	/// Create a field from name (note: this will perform the hashing)
	pub fn from_name<Value>(name: &str, value: Value) -> Self
	where
		Value: Into<CowStr<'ctx>>,
	{
		let hash = fnv::hash(name);

		Field {
			hash,
			value: value.into(),
		}
	}
}
